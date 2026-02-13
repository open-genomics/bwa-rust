# 教程：从 0 实现一个 BWA 风格的 Rust FM 索引和对齐器

## 概述

本教程介绍 bwa-rust 项目的核心算法和实现思路。通过阅读本文档，你将了解如何：

1. 构建 FM 索引（后缀数组 → BWT → C/Occ 表）
2. 使用 FM 索引进行精确匹配
3. 查找 SMEM 种子
4. 通过种子链 + Smith-Waterman 完成序列比对

## 第一步：理解 FM 索引

FM 索引是一种基于 BWT 的全文索引结构，支持高效的子串搜索。

### 构建流程

```
参考序列 "ACGT$"
    │
    ▼
后缀数组 (SA)    → 所有后缀按字典序排列
    │
    ▼
BWT              → SA 中每个位置的前一个字符
    │
    ▼
C 表             → 字母表中每个字符的累计频率
Occ 采样         → BWT 中每个字符到某位置的出现次数
```

### Rust 代码示例

```rust
use bwa_rust::index::{sa, bwt, fm};
use bwa_rust::util::dna;

// 编码参考序列
let reference = b"ACGTACGT";
let norm = dna::normalize_seq(reference);
let mut text: Vec<u8> = norm.iter().map(|&b| dna::to_alphabet(b)).collect();
text.push(0); // 添加 sentinel

// 构建索引
let sa_arr = sa::build_sa(&text);
let bwt_arr = bwt::build_bwt(&text, &sa_arr);
let contigs = vec![fm::Contig { name: "ref".into(), len: 8, offset: 0 }];
let fm_idx = fm::FMIndex::build(text, bwt_arr, sa_arr, contigs, 6, 4);
```

## 第二步：精确匹配搜索

FM 索引的 backward search 可以在 O(m) 时间内查找长度为 m 的模式。

```rust
let pattern = b"CGT";
let pattern_alpha: Vec<u8> = dna::normalize_seq(pattern)
    .iter().map(|&b| dna::to_alphabet(b)).collect();

if let Some((l, r)) = fm_idx.backward_search(&pattern_alpha) {
    let count = r - l; // 出现次数
    let positions = fm_idx.sa_interval_positions(l, r); // 具体位置
    println!("'CGT' 出现 {} 次", count);
}
```

## 第三步：SMEM 种子查找

SMEM（Super-Maximal Exact Match）是 BWA-MEM 的核心概念。对于 read 上的每个位置，找到覆盖该位置的最长精确匹配。

```rust
use bwa_rust::align::find_smem_seeds;

let read_alpha: Vec<u8> = /* 编码后的 read */;
let seeds = find_smem_seeds(&fm_idx, &read_alpha, 10); // min_len=10

for seed in &seeds {
    println!("read[{}..{}] 匹配 ref[{}..{}]",
        seed.qb, seed.qe, seed.rb, seed.re);
}
```

## 第四步：种子链构建

将多个种子组合成一条"链"，选择覆盖度最高、间距合理的种子组合。

```rust
use bwa_rust::align::{build_chains, filter_chains};

let mut chains = build_chains(&seeds, read_len);
filter_chains(&mut chains, 0.3); // 过滤弱链

// chains[0] 是得分最高的链
```

## 第五步：Smith-Waterman 对齐

在种子链的间隙区域执行带状仿射间隙 Smith-Waterman 局部对齐，得到完整的 CIGAR 和对齐得分。

```rust
use bwa_rust::align::{banded_sw, SwParams};

let params = SwParams {
    match_score: 2,
    mismatch_penalty: 1,
    gap_open: 2,
    gap_extend: 1,
    band_width: 16,
};

let result = banded_sw(query, reference, params);
println!("Score: {}, CIGAR: {}, NM: {}", result.score, result.cigar, result.nm);
```

## 第六步：SAM 输出

最终将比对结果格式化为 SAM 行，包含 FLAG、RNAME、POS、MAPQ、CIGAR 等字段，以及 AS/XS/NM 可选标签。

## 完整示例

参见 `examples/simple_align.rs`，演示了从构建索引到对齐输出的完整流程。

```bash
cargo run --example simple_align
```

## 进阶主题

- **多线程对齐**：使用 `--threads N` 参数启用 rayon 并行处理
- **稀疏 SA 采样**：`FMIndex::build_sparse()` 可减少内存占用
- **自定义打分参数**：通过 `--match`/`--mismatch`/`--gap-open`/`--gap-ext` 调整
