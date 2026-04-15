# 教程：从 0 实现一个 BWA 风格的 Rust FM 索引和对齐器

## 概述

本教程介绍 bwa-rust 项目的核心算法和实现思路。通过阅读本文档，你将了解如何：

1. 构建 FM 索引（后缀数组 → BWT → C/Occ 表）
2. 使用 FM 索引进行精确匹配
3. 查找 SMEM 种子
4. 通过种子链 + Smith-Waterman 完成序列比对
5. 输出标准 SAM 格式

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

### 基本用法

```rust
use bwa_rust::align::find_smem_seeds;
use bwa_rust::util::dna;

// 将 read 编码为字母表格式
let read = b"ACGTACGT";
let read_alpha: Vec<u8> = dna::normalize_seq(read)
    .iter().map(|&b| dna::to_alphabet(b)).collect();

let seeds = find_smem_seeds(&fm_idx, &read_alpha, 5); // min_len=5

for seed in &seeds {
    println!("read[{}..{}] 匹配 ref[{}..{}]",
        seed.qb, seed.qe, seed.rb, seed.re);
}
```

### 内存防护：限制重复种子

对于高度重复序列（如 poly-A 尾巴），种子可能出现数万次。使用 `max_occ` 参数跳过这些种子：

```rust
use bwa_rust::align::find_smem_seeds_with_max_occ;

// 仅保留出现次数 <= 500 的种子
let seeds = find_smem_seeds_with_max_occ(&fm_idx, &read_alpha, 5, 500);
```

## 第四步：种子链构建

将多个种子组合成一条"链"，选择覆盖度最高、间距合理的种子组合。

### 基本用法

```rust
use bwa_rust::align::{build_chains, filter_chains};

let mut chains = build_chains(&seeds, read_len);
filter_chains(&mut chains, 0.3); // 过滤弱链

// chains[0] 是得分最高的链
```

### 限制链数量

使用 `max_chains_per_contig` 限制每个 contig 提取的链数：

```rust
use bwa_rust::align::build_chains_with_limit;

// 每个 contig 最多提取 5 条链
let chains = build_chains_with_limit(&seeds, read_len, 5);
```

## 第五步：Smith-Waterman 对齐

在种子链的间隙区域执行带状仿射间隙 Smith-Waterman 局部对齐，得到完整的 CIGAR 和对齐得分。

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

let result = banded_sw(query, reference, params);
println!("Score: {}, CIGAR: {}, NM: {}", result.score, result.cigar, result.nm);
```

### BWA-MEM 默认参数

`mem` 子命令使用 BWA-MEM 默认打分：

| 参数 | 值 | 说明 |
|------|-----|------|
| match | 1 | 匹配得分 |
| mismatch | 4 | 错配罚分 |
| gap_open | 6 | Gap 开启罚分 |
| gap_ext | 1 | Gap 扩展罚分 |
| band_width | 100 | 带宽 |

## 第六步：SAM 输出

最终将比对结果格式化为 SAM 行，包含 FLAG、RNAME、POS、MAPQ、CIGAR 等字段，以及 AS/XS/NM 可选标签。

```rust
use bwa_rust::io::sam;

// 写入 SAM header
let contigs = vec![("chr1", 1000u32), ("chr2", 2000u32)];
let mut out = Vec::new();
sam::write_header(&mut out, &contigs).unwrap();

// 生成 mapped 记录（FLAG=0 正向，FLAG=16 反向互补）
let line = sam::format_record(
    "read1",   // QNAME
    0,         // FLAG
    "chr1",    // RNAME
    100,       // POS (1-based)
    60,        // MAPQ
    "50M",     // CIGAR
    "ACGT...", // SEQ
    "IIII...", // QUAL
    100,       // AS (alignment score)
    0,         // XS (suboptimal score)
    2,         // NM (edit distance)
);
println!("{}", line);

// 生成 unmapped 记录（FLAG=4）
let unmapped = sam::format_unmapped("read2", "NNNN", "!!!!");
println!("{}", unmapped);
```

## 第七步：完整对齐流水线

使用 `AlignOpt` 配置完整的对齐参数：

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
    max_occ: 500,              // 跳过高度重复种子
    max_chains_per_contig: 5,  // 每个 contig 最大链数
    max_alignments_per_read: 5, // 每 read 最大输出数
};

// 验证参数
opt.validate().expect("Invalid AlignOpt");
```

## 完整示例

参见 `examples/simple_align.rs`，演示了从构建索引到对齐输出的完整流程。

```bash
cargo run --example simple_align
```

## CLI 使用

### 构建索引

```bash
# 从 FASTA 参考序列构建 FM 索引
bwa-rust index data/toy.fa -o data/toy
# 输出：data/toy.fm
```

### 比对 Reads

```bash
# 基本比对
bwa-rust align -i data/toy.fm data/toy_reads.fq

# 输出到文件
bwa-rust align -i data/toy.fm data/toy_reads.fq -o output.sam

# 多线程（4 线程）
bwa-rust align -i data/toy.fm data/toy_reads.fq -t 4

# 自定义打分参数
bwa-rust align -i data/toy.fm data/toy_reads.fq \
    --match 2 --mismatch 1 --gap-open 2 --gap-ext 1 --band-width 16
```

### 一步比对（BWA-MEM 风格）

```bash
# 构建索引并比对
bwa-rust mem data/toy.fa data/toy_reads.fq -t 4 -o output.sam

# 使用 BWA-MEM 默认打分
bwa-rust mem data/toy.fa data/toy_reads.fq \
    -A 1 -B 4 -O 6 -E 1 -w 100
```

## 进阶主题

### 多线程对齐

使用 `--threads N` 参数启用 rayon 并行处理：

```bash
bwa-rust mem ref.fa reads.fq -t 8 -o output.sam
```

内部使用自定义 rayon 线程池，避免全局线程池竞争。

### 内存优化

1. **稀疏 SA 采样**：`FMIndex::build_sparse()` 可减少内存占用
2. **max_occ 过滤**：跳过高度重复种子，防止内存爆炸
3. **SwBuffer 复用**：带状 DP 缓冲区复用，减少热路径分配

### 性能调优

```bash
# 小带宽：快速但对 indel 敏感度低
bwa-rust align -i ref.fm reads.fq --band-width 16

# 大带宽：慢但对 indel 容忍度高
bwa-rust align -i ref.fm reads.fq --band-width 64

# 降低阈值：输出更多比对
bwa-rust align -i ref.fm reads.fq --score-threshold 10
```
