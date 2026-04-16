# 教程：从零实现 BWA 风格的序列比对器

> 本教程介绍 bwa-rust 的核心算法和实现思路。

---

## 概述

通过阅读本文档，你将了解：

```
参考序列 → FM 索引 → SMEM 种子 → 种子链 → Smith-Waterman → SAM 输出
```

| 步骤 | 算法 | 复杂度 |
|------|------|--------|
| FM 索引构建 | 后缀数组 → BWT → C/Occ 表 | O(n log²n) |
| 精确匹配 | Backward search | O(m) |
| 种子查找 | SMEM（Super-Maximal Exact Match）| O(m) |
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
│   SA[0]=5 → "$"                                     │
│   SA[1]=0 → "ACGT$"                                 │
│   SA[2]=2 → "CGT$"                                  │
│   ...                                               │
└─────────────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────────────┐
│ BWT (Burrows-Wheeler Transform)                     │
│ SA 中每个位置的前一个字符                             │
│ BWT[i] = text[(SA[i] - 1) mod n]                    │
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
let sa_arr = sa::build_sa(&text);           // O(n log²n)
let bwt_arr = bwt::build_bwt(&text, &sa_arr);
let contigs = vec![fm::Contig { name: "ref".into(), len: 8, offset: 0 }];
let fm_idx = fm::FMIndex::build(text, bwt_arr, sa_arr, contigs, 6, 4);
```

---

## 第二步：精确匹配搜索

FM 索引的 backward search 可以在 O(m) 时间内查找长度为 m 的模式。

```rust
// 搜索模式 "CGT"
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
use bwa_rust::util::dna;

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

// 仅保留出现次数 ≤ 500 的种子
// 防止 poly-A 等重复序列导致内存爆炸
let seeds = find_smem_seeds_with_max_occ(&fm_idx, &read_alpha, 5, 500);
```

### SMEM 查找流程

```
Read: ACGTACGT
      ^^^^^
      种子1: read[0:5] → ref[10:15], occ=1

        ^^^^^
        种子2: read[3:8] → ref[20:25], occ=1

种子之间可能有重叠，但每个种子在其位置上是最长的精确匹配
```

---

## 第四步：种子链构建

将多个种子组合成一条"链"，选择覆盖度最高、间距合理的种子组合。

### 链评分

```
链得分 = Σ(seed.length) - gap_penalty

gap_penalty 基于种子之间的距离和方向一致性
```

### 基本用法

```rust
use bwa_rust::align::{build_chains, filter_chains};

let mut chains = build_chains(&seeds, read_len);
filter_chains(&mut chains, 0.3); // 过滤得分 < 最佳 * 0.3 的弱链

// chains[0] 是得分最高的链
```

### 限制链数量

```rust
use bwa_rust::align::build_chains_with_limit;

// 每个 contig 最多提取 5 条链
// 防止重复区域产生过多候选
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

let result = banded_sw(query, reference, params);
println!("Score: {}, CIGAR: {}, NM: {}", result.score, result.cigar, result.nm);
```

### BWA-MEM 默认参数

| 参数 | `align` 默认 | `mem` 默认 | 说明 |
|------|-------------|-----------|------|
| match | 2 | 1 | 匹配得分 |
| mismatch | 1 | 4 | 错配罚分 |
| gap_open | 2 | 6 | Gap 开启罚分 |
| gap_ext | 1 | 1 | Gap 扩展罚分 |
| band_width | 16 | 100 | 带宽 |

### 仿射间隙模型

```
Score = match × M - mismatch × X - gap_open × G - gap_ext × E

其中：
- M: 匹配数
- X: 错配数
- G: gap 开启次数
- E: gap 扩展总长度
```

---

## 第六步：SAM 输出

将比对结果格式化为标准 SAM 格式。

```rust
use bwa_rust::io::sam;

// 写入 SAM header
let contigs = vec![("chr1", 1000u32), ("chr2", 2000u32)];
let mut out = Vec::new();
sam::write_header(&mut out, &contigs).unwrap();

// 生成 mapped 记录
let line = sam::format_record(
    "read1",   // QNAME
    0,         // FLAG (0=正向, 16=反向)
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

// 生成 unmapped 记录（FLAG=4）
let unmapped = sam::format_unmapped("read2", "NNNN", "!!!!");
```

### SAM 字段说明

| 字段 | 说明 |
|------|------|
| FLAG | 比对标志（0=正向，4=unmapped，16=反向） |
| POS | 1-based 参考序列起始位置 |
| MAPQ | 映射质量（0-60） |
| CIGAR | 比对操作序列（M/I/D/S） |
| AS | 比对得分 |
| XS | 次优比对得分 |
| NM | 编辑距离 |

---

## 第七步：完整对齐流水线

### AlignOpt 配置

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
    max_chains_per_contig: 5,   // 每 contig 最大链数
    max_alignments_per_read: 5, // 每 read 最大输出数
};

opt.validate().expect("Invalid AlignOpt");
```

### 内存防护机制

```
┌────────────────────────────────────────────────────────┐
│ 三层内存防护                                            │
├────────────────────────────────────────────────────────┤
│ 1. max_occ = 500                                       │
│    跳过 SA 区间 > 500 的种子                            │
│    防止 poly-A 等重复序列展开数万位置                    │
│                                                        │
│ 2. max_chains_per_contig = 5                           │
│    每个 contig 最多提取 5 条链                          │
│    防止重复区域产生过多候选                             │
│                                                        │
│ 3. max_alignments_per_read = 5                         │
│    每个 read 最多输出 5 个比对结果                       │
│    控制最终输出规模                                     │
└────────────────────────────────────────────────────────┘
```

---

## CLI 使用

### 构建索引

```bash
# 从 FASTA 构建 FM 索引
bwa-rust index data/toy.fa -o data/toy
# 输出：data/toy.fm
```

### 比对 Reads

```bash
# 基本比对
bwa-rust align -i data/toy.fm data/toy_reads.fq

# 多线程
bwa-rust align -i data/toy.fm data/toy_reads.fq -t 4

# 自定义参数
bwa-rust align -i ref.fm reads.fq \
    --match 2 --mismatch 1 --gap-open 2 --gap-ext 1
```

### 一步比对（BWA-MEM 风格）

```bash
# 构建索引并比对
bwa-rust mem data/toy.fa data/toy_reads.fq -t 4 -o output.sam

# 使用 BWA-MEM 默认打分
bwa-rust mem ref.fa reads.fq -A 1 -B 4 -O 6 -E 1 -w 100
```

---

## 进阶主题

### 多线程对齐

```bash
bwa-rust mem ref.fa reads.fq -t 8 -o output.sam
```

内部使用自定义 rayon 线程池，避免全局线程池竞争。

### 内存优化

| 优化 | 方法 |
|------|------|
| 稀疏 SA 采样 | `FMIndex::build_sparse()` |
| max_occ 过滤 | 跳过高度重复种子 |
| SwBuffer 复用 | 带状 DP 缓冲区复用 |

### 性能调优

```bash
# 小带宽：快速但对 indel 敏感度低
bwa-rust align -i ref.fm reads.fq --band-width 16

# 大带宽：慢但对 indel 容忍度高
bwa-rust align -i ref.fm reads.fq --band-width 64

# 降低阈值：输出更多比对
bwa-rust align -i ref.fm reads.fq --score-threshold 10
```

---

## 完整示例

```bash
# 运行示例代码
cargo run --example simple_align
```

示例演示了从构建索引到对齐输出的完整流程。
