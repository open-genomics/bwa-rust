# 算法教程

> 从零理解 bwa-rust 的核心算法和实现思路。

---

## 概述

通过本教程，你将了解：

| 步骤 | 算法 | 复杂度 |
|------|------|--------|
| FM 索引构建 | 后缀数组 → BWT → C/Occ 表 | O(n log²n) |
| 精确匹配 | Backward search | O(m) |
| 种子查找 | SMEM（Super-Maximal Exact Match） | O(m) |
| 链构建 | DP + 贪心剥离 | O(k²) |
| 局部对齐 | 带状 Smith-Waterman | O(n·w) |

---

## 第一步：理解 FM 索引

FM 索引是一种基于 BWT 的全文索引结构，支持高效的子串搜索。

### 构建流程

```
参考序列 "ACGT$"
    │
    ▼
┌─────────────────────────────────────────────────────┐
│ 后缀数组 (SA)                                        │
│ 所有后缀按字典序排列                                  │
│ "ACGT$" → [5, 0, 2, 4, 1, 3]                        │
└─────────────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────────────┐
│ BWT (Burrows-Wheeler Transform)                     │
│ SA 中每个位置的前一个字符                             │
│ "ACGT$" → "$ACGTA"                                  │
└─────────────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────────────┐
│ C 表 + Occ 采样                                      │
│ C[c] = 字符 c 之前所有字符的总数                       │
│ Occ(c, i) = BWT[0..i] 中 c 的出现次数                │
└─────────────────────────────────────────────────────┘
```

### Rust 代码示例

```rust
use bwa_rust::index::{sa, bwt, fm};
use bwa_rust::util::dna;

// 1. 编码参考序列
let reference = b"ACGTACGT";
let norm = dna::normalize_seq(reference);
let mut text: Vec<u8> = norm.iter().map(|&b| dna::to_alphabet(b)).collect();
text.push(0); // 添加 sentinel

// 2. 构建索引
let sa_arr = sa::build_sa(&text);
let bwt_arr = bwt::build_bwt(&text, &sa_arr);
let contigs = vec![fm::Contig { name: "ref".into(), len: 8, offset: 0 }];
let fm_idx = fm::FMIndex::build(text, bwt_arr, sa_arr, contigs, 6, 4);
```

---

## 第二步：精确匹配搜索

FM 索引的 backward search 可以在 O(m) 时间内查找长度为 m 的模式。

```rust
let pattern = b"CGT";
let pattern_alpha: Vec<u8> = dna::normalize_seq(pattern)
    .iter().map(|&b| dna::to_alphabet(b)).collect();

if let Some((l, r)) = fm_idx.backward_search(&pattern_alpha) {
    let count = r - l;                          // 出现次数
    let positions = fm_idx.sa_interval_positions(l, r); // 具体位置
    println!("'CGT' 出现 {} 次", count);
}
```

### Backward Search 原理

```
模式 "CGT" 从右向左匹配：

Step 1: 匹配 'T' → C['T'] ≤ i < C['T'+1]
Step 2: 匹配 'G' → C['G'] + Occ('G', l) ≤ i < C['G'] + Occ('G', r)
Step 3: 匹配 'C' → C['C'] + Occ('C', l) ≤ i < C['C'] + Occ('C', r)

最终 SA 区间 [l, r) 包含所有匹配位置
```

---

## 第三步：SMEM 种子查找

SMEM（Super-Maximal Exact Match）是 BWA-MEM 的核心概念。

> 对于 read 上的每个位置，找到覆盖该位置的**最长**精确匹配，且不被任何其他精确匹配包含。

### 基本用法

```rust
use bwa_rust::align::find_smem_seeds;

let read = b"ACGTACGT";
let read_alpha: Vec<u8> = dna::normalize_seq(read)
    .iter().map(|&b| dna::to_alphabet(b)).collect();

let seeds = find_smem_seeds(&fm_idx, &read_alpha, 5); // min_len=5

for seed in &seeds {
    println!("read[{}..{}] → ref[{}..{}], occ={}",
        seed.qb, seed.qe, seed.rb, seed.re, seed.occ);
}
```

### 内存防护：限制重复种子

```rust
use bwa_rust::align::find_smem_seeds_with_max_occ;

// 仅保留出现次数 <= 500 的种子
// 防止 poly-A 等重复序列导致内存爆炸
let seeds = find_smem_seeds_with_max_occ(&fm_idx, &read_alpha, 5, 500);
```

---

## 第四步：种子链构建

将多个种子组合成一条"链"，选择覆盖度最高、间距合理的种子组合。

```rust
use bwa_rust::align::{build_chains, filter_chains};

let mut chains = build_chains(&seeds, read_len);
filter_chains(&mut chains, 0.3); // 过滤弱链

// chains[0] 是得分最高的链
```

### 限制链数量

```rust
use bwa_rust::align::build_chains_with_limit;

// 每个 contig 最多提取 5 条链
let chains = build_chains_with_limit(&seeds, read_len, 5);
```

---

## 第五步：Smith-Waterman 对齐

在种子链的间隙区域执行带状仿射间隙 SW 局部对齐。

### 参数配置

```rust
use bwa_rust::align::{banded_sw, SwParams};

let params = SwParams {
    match_score: 2,
    mismatch_penalty: 1,
    gap_open: 2,
    gap_extend: 1,
    band_width: 16,
};

let result = banded_sw(query, ref_seq, params);
println!("Score: {}, CIGAR: {}, NM: {}", result.score, result.cigar, result.nm);
```

### BWA-MEM 默认参数

| 参数 | `align` 默认 | `mem` 默认 |
|------|-------------|-----------|
| match | 2 | 1 |
| mismatch | 1 | 4 |
| gap_open | 2 | 6 |
| gap_ext | 1 | 1 |
| band_width | 16 | 100 |

---

## 第六步：完整配置

使用 `AlignOpt` 配置完整参数：

```rust
use bwa_rust::align::AlignOpt;

let opt = AlignOpt {
    // 打分参数
    match_score: 2,
    mismatch_penalty: 1,
    gap_open: 2,
    gap_extend: 1,
    clip_penalty: 1,

    // 对齐参数
    band_width: 16,
    score_threshold: 20,
    min_seed_len: 19,

    // 并行参数
    threads: 4,

    // 内存防护
    max_occ: 500,               // 跳过高度重复种子
    max_chains_per_contig: 5,   // 每个 contig 最大链数
    max_alignments_per_read: 5, // 每 read 最大输出数
};

opt.validate().expect("Invalid AlignOpt");
```

---

## 完整流程

```
FASTA 参考序列
    │
    ▼
FM 索引构建（SA → BWT → C/Occ）
    │
    ▼
序列化为 .fm 文件

FASTQ reads
    │
    ▼
加载 FM 索引
    │
    ▼
对每条 read：
    ├─ SMEM 种子查找（正向 + 反向互补，max_occ 过滤）
    │
    ▼
    ├─ 种子链构建与过滤（max_chains 限制）
    │
    ▼
    ├─ 链→SW 扩展→完整对齐（semi-global refinement）
    │
    ▼
    └─ 候选去重、MAPQ 估算
    │
    ▼
输出 SAM（max_alignments 限制）
```

---

## 运行示例

```bash
cargo run --example simple_align
```

详细的源码实现见 [GitHub 仓库](https://github.com/LessUp/bwa-rust)。
