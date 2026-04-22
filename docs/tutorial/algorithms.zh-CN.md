# 核心算法教程

> 本教程深入讲解 bwa-rust 的核心算法：FM 索引、SMEM 种子查找和 Smith-Waterman 对齐。

---

## 目录

- [FM 索引原理](#fm-索引原理)
- [SMEM 种子查找](#smem-种子查找)
- [种子链构建](#种子链构建)
- [Smith-Waterman 对齐](#smith-waterman-对齐)
- [完整比对流水线](#完整比对流水线)

---

## FM 索引原理

### 什么是 FM 索引？

FM 索引（Ferragina-Manzini Index）是一种基于 BWT（Burrows-Wheeler Transform）的全文索引数据结构，支持：

- **高效子串搜索**：O(m) 时间查找长度为 m 的模式
- **内存高效**：通过采样大幅减少存储需求
- **可逆操作**：支持从 BWT 恢复原始序列

### 构建流程

```
原始序列: ACGTACGT$
    │
    ▼
┌─────────────────────────────┐
│ 1. 后缀数组 (Suffix Array)   │
│    所有后缀按字典序排序        │
│    [8, 0, 4, 1, 5, 2, 6, 3, 7]│
└─────────────────────────────┘
    │
    ▼
┌─────────────────────────────┐
│ 2. BWT 变换                  │
│    取每个后缀的前一个字符      │
│    BWT = "T$TCAGAGC"         │
└─────────────────────────────┘
    │
    ▼
┌─────────────────────────────┐
│ 3. C 表 + Occ 表             │
│    C: 每个字符的起始位置       │
│    Occ: 累计出现次数          │
└─────────────────────────────┘
```

### Backward Search

FM 索引的核心操作，从右向左搜索模式：

```rust
// 搜索模式 "CGT"
// Step 1: 从 'T' 开始
SA_range = [C['T'], C['T']+1) = [6, 8)

// Step 2: 扩展 'G'
new_l = C['G'] + Occ('G', l) = 3 + 1 = 4
new_r = C['G'] + Occ('G', r) = 3 + 2 = 5
SA_range = [4, 5)

// Step 3: 扩展 'C'
final_l = C['C'] + Occ('C', 4) = 1 + 1 = 2
final_r = C['C'] + Occ('C', 5) = 1 + 2 = 3
SA_range = [2, 3)  // 找到 1 次匹配

// 匹配位置 = SA[2] = 4
// 即 "CGT" 在原始序列的第 4 位
```

**时间复杂度：O(m)** — 与序列长度无关！

### 代码示例

```rust
use bwa_rust::index::{sa, bwt, fm};
use bwa_rust::util::dna;

fn main() {
    let reference = b"ACGTACGT";

    // 编码
    let norm = dna::normalize_seq(reference);
    let mut text: Vec<u8> = norm.iter().map(|&b| dna::to_alphabet(b)).collect();
    text.push(0);  // sentinel

    // 构建索引
    let sa_arr = sa::build_sa(&text);
    let bwt_arr = bwt::build_bwt(&text, &sa_arr);
    let contigs = vec![fm::Contig { name: "ref".into(), len: 8, offset: 0 }];
    let fm_idx = fm::FMIndex::build(text, bwt_arr, sa_arr, contigs, 6, 4);

    // 搜索
    let pattern: Vec<u8> = b"CGT".iter().map(|&b| dna::to_alphabet(b)).collect();
    if let Some((l, r)) = fm_idx.backward_search(&pattern) {
        println!("'CGT' 出现 {} 次", r - l);

        for pos in fm_idx.sa_interval_positions(l, r) {
            println!("  位置: {}", pos);
        }
    }
}
```

---

## SMEM 种子查找

### 什么是 SMEM？

**SMEM** (Super-Maximal Exact Match) 是 BWA-MEM 的核心概念：

> 对于 read 上的每个位置，找到覆盖该位置的**最长**精确匹配，且不被任何其他精确匹配包含。

### 算法步骤

```
Read: ACGTACGTACGT

1. 从位置 0 开始，向右扩展，找到最长匹配
   "ACGTACGT" 在参考序列匹配到位置 1000
   → 尝试向左扩展，无法再扩展

   Seed 1: read[0:8] → ref[1000:1008]

2. 跳到位置 4，重复上述过程
   "ACGTACGT" 在参考序列匹配到位置 2000

   Seed 2: read[4:12] → ref[2000:2008]

注意：两个种子重叠，但各自在其位置上是最长的。
```

### 内存防护

对于重复序列（如 poly-A），可能出现数万次匹配，使用 `max_occ` 限制：

```rust
// 仅保留出现次数 ≤ 500 的种子
let seeds = find_smem_seeds_with_max_occ(&fm_idx, &read, min_len, 500);
```

---

## 种子链构建

### 目标

将多个种子组合成"链"，表示 read 可能比对到的参考区域。

### 评分公式

```
链得分 = Σ(种子长度) - Gap 罚分

Gap 罚分规则：
- 共线种子（距离合理）：线性罚分
- 非共线种子：重罚
```

### DP 算法

```rust
// 种子按参考位置排序
seeds.sort_by_key(|s| s.rb);

// 动态规划找最佳链
for i in 0..seeds.len() {
    best_chain[i] = seeds[i].len;  // 单独一个种子

    for j in 0..i {
        if collinear(seeds[j], seeds[i]) {
            let candidate = best_chain[j] + gap_score(seeds[j], seeds[i]);
            if candidate > best_chain[i] {
                best_chain[i] = candidate;
                prev[i] = j;
            }
        }
    }
}

// 回溯找链
reconstruct_chain(prev, argmax(best_chain));
```

---

## Smith-Waterman 对齐

### 问题

在种子链的间隙区域执行局部对齐，处理：
- 错配 (mismatch)
- 插入/删除 (indel)
- 软剪切 (soft clipping)

### 带状优化

标准 SW 需要 O(m×n) 时间和空间。使用带状优化：

```
标准矩阵 (m×n):
┌────────────┐
│ ██████████ │  时间/空间: O(m×n)
│ ██████████ │
│ ██████████ │
└────────────┘

带状矩阵 (m×w):
┌────────────┐
│     ██     │  时间/空间: O(m×w)
│    ████    │  w = 带宽 (默认 16)
│     ██     │
└────────────┘
```

### DP 公式

```
M[i][j] = 匹配得分
X[i][j] = 纵向 gap 得分（插入）
Y[i][j] = 横向 gap 得分（删除）

M[i][j] = max(
    M[i-1][j-1] + match_score(query[i], ref[j]),
    X[i-1][j-1] + match_score(query[i], ref[j]),
    Y[i-1][j-1] + match_score(query[i], ref[j]),
    0  // SW 允许局部对齐
)

X[i][j] = max(
    M[i-1][j] - gap_open,
    X[i-1][j] - gap_extend
)

Y[i][j] = max(
    M[i][j-1] - gap_open,
    Y[i][j-1] - gap_extend
)
```

### 复杂度对比

| 实现 | 时间 | 空间 | 100bp × 1000bp |
|-----|------|------|----------------|
| 标准 SW | O(m×n) | O(m×n) | 100K cells |
| 带状 SW | O(m×w) | O(m×w) | 1.6K cells (w=16) |

**节省 98%+**

---

## 完整比对流水线

```
┌──────────────────────────────────────────────────────────┐
│                    Input: FASTQ Read                      │
└──────────────────────────────────────────────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        ▼                   ▼                   ▼
   ┌─────────┐        ┌─────────┐         ┌─────────┐
   │ Forward │        │ Forward │         │ Reverse │
   │ Sequence│        │  Seeds  │         │ Seeds   │
   └────┬────┘        └────┬────┘         └────┬────┘
        │                   │                   │
        │              ┌────┴────┐         ┌────┴────┐
        │              │ Chains  │         │ Chains  │
        │              └────┬────┘         └────┬────┘
        │                   │                   │
        │              ┌────┴────┐         ┌────┴────┐
        │              │   SW    │         │   SW    │
        │              └────┬────┘         └────┬────┘
        │                   │                   │
        └───────────────────┼───────────────────┘
                            ▼
              ┌─────────────────────┐
              │   Merge & Dedup     │
              └──────────┬──────────┘
                         ▼
              ┌─────────────────────┐
              │ Sort by Quality     │
              └──────────┬──────────┘
                         ▼
              ┌─────────────────────┐
              │ Select Primary/Sec  │
              └──────────┬──────────┘
                         ▼
              ┌─────────────────────┐
              │   Output: SAM      │
              └─────────────────────┘
```

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
    max_occ: 500,
    max_chains_per_contig: 5,
    max_alignments_per_read: 5,
};

opt.validate().expect("Invalid AlignOpt");
```

---

## 相关文档

- [快速入门](./getting-started.zh-CN.md) — 安装和基本使用
- [架构总览](../architecture/overview.zh-CN.md) — 模块设计
- [索引构建](../architecture/index-building.zh-CN.md) — 索引构建详解
- [比对算法](../architecture/alignment.zh-CN.md) — 完整比对流程
