# bwa-rust 架构概述

> 本文档提供 bwa-rust 的模块化架构设计概览、数据流和技术栈。

---

## 目录

- [整体架构](#整体架构)
- [模块详解](#模块详解)
- [数据流](#数据流)
- [技术栈](#技术栈)
- [与 BWA/BWA-MEM 的差异](#与-bwabwa-mem-的差异)
- [安全性保证](#安全性保证)
- [性能优化](#性能优化)

---

## 整体架构

```
┌─────────────────────────────────────────────────────────────┐
│                     CLI Layer (main.rs)                      │
│                  clap 命令解析 + 调度                         │
├─────────────┬─────────────┬──────────────┬──────────────────┤
│    io/      │   index/    │    align/    │      util/       │
│  输入输出    │   索引构建   │   比对算法    │     工具函数     │
├─────────────┼─────────────┼──────────────┼──────────────────┤
│  FASTA      │   SA        │  Seed        │  DNA 编码        │
│  FASTQ      │   BWT       │  Chain       │  反向互补        │
│  SAM        │   FM        │  SW/Extend   │                  │
│             │  Builder    │  Candidate   │                  │
│             │             │  MAPQ        │                  │
│             │             │  Pipeline    │                  │
└─────────────┴─────────────┴──────────────┴──────────────────┘
```

### 设计原则

1. **模块化** —— 每个模块职责单一，高内聚低耦合
2. **内存安全** —— 零 `unsafe` 代码，编译时安全保证
3. **性能优先** —— 关键路径优化（jemalloc、缓冲区复用）

---

## 模块详解

### 1. `io/` — 输入输出层

| 文件 | 功能 | 关键函数 |
|------|------|----------|
| `fasta.rs` | FASTA 解析 | `parse_fasta()`, `normalize_seq()` |
| `fastq.rs` | FASTQ 解析 | `parse_fastq_record()` |
| `sam.rs` | SAM 输出 | `write_header()`, `format_record()` |

**FASTA 解析特性：**
- ✅ 多 contig 支持
- ✅ 自动归一化（大写、过滤非标准字符）
- ✅ 支持不同换行符（LF/CRLF）
- ✅ 空序列检测
- ✅ 重复 contig 名称检测

**SAM 输出格式：**
```
@HD	VN:1.6	SO:unsorted
@SQ	SN:chr1	LN:1000
@PG	ID:bwa-rust	VN:0.2.0	CL:bwa-rust mem ...
read1	0	chr1	100	60	50M	*	0	ACGT...	IIII...	AS:i:100	XS:i:0	NM:i:2
```

### 2. `index/` — 索引构建

| 文件 | 功能 | 复杂度 |
|------|------|--------|
| `sa.rs` | 后缀数组构建 | O(n log²n) |
| `bwt.rs` | BWT 构建 | O(n) |
| `fm.rs` | FM 索引核心 | O(n) 空间 |
| `builder.rs` | 构建入口 | - |

**后缀数组算法：**

倍增法 (Doubling Algorithm):
- 第 k 轮：按后缀前 2^k 字符排序
- 最终：得到完整后缀数组

示例：text = "banana$"
```
SA = [6, 5, 3, 1, 0, 4, 2]
     $  a  a  a  b  n  n
     $  na na na an an
```

**FM 索引结构：**
```rust
FMIndex {
    sigma: 6,           // 字母表大小 {$, A, C, G, T, N}
    block: 64,          // Occ 采样块大小

    c: [0, 1, 2, 3, 4, 5, 6],  // C 表：累计频率

    bwt: Vec<u8>,       // BWT 序列
    occ_samples: Vec<u32>, // Occ 采样表

    sa: Vec<u32>,       // 后缀数组（完整或稀疏）
    sa_sample_rate: 4,  // SA 采样间隔

    contigs: Vec<Contig>, // contig 元信息
    text: Vec<u8>,       // 原始编码文本
}
```

**索引文件格式 (.fm):**

```
┌─────────────────────────────────────────────┐
│ magic: u64 = 0x424D4146_4D5F5253 ("BWAFM_RS")│
│ version: u32 = 2                            │
│ sigma: u8 = 6                               │
│ block: u32                                  │
│ c: Vec<u32>                                 │
│ bwt: Vec<u8>                                │
│ occ_samples: Vec<u32>                       │
│ sa: Vec<u32>                                │
│ sa_sample_rate: u32                         │
│ contigs: Vec<Contig>                        │
│ text: Vec<u8>                               │
│ meta: Option<IndexMeta>                     │
└─────────────────────────────────────────────┘
```

### 3. `align/` — 比对算法

| 文件 | 功能 | 关键函数 |
|------|------|----------|
| `mod.rs` | 配置定义 | `AlignOpt`, `validate()` |
| `seed.rs` | SMEM 种子 | `find_smem_seeds()` |
| `chain.rs` | 链构建 | `build_chains()`, `filter_chains()` |
| `sw.rs` | Smith-Waterman | `banded_sw()` |
| `extend.rs` | 链扩展 | `chain_to_alignment()` |
| `candidate.rs` | 候选管理 | `collect_candidates()`, `dedup_candidates()` |
| `mapq.rs` | MAPQ 估算 | `compute_mapq()` |
| `pipeline.rs` | 完整流水线 | `align_reads()` |

---

## 数据流

```
FASTQ读取
    │
    ├─ 正向归一化 ──────────────┐
    │                          │
    ├─ 反向互补归一化 ──────────┤
    │                         │
    ▼                         ▼
┌──────────────────────────────────────┐
│ SMEM 种子查找 (seed.rs)               │
│ • 对每个位置找最长精确匹配            │
│ • max_occ 过滤重复种子                │
└──────────────────────────────────────┘
    │
    ▼
┌──────────────────────────────────────┐
│ 种子链构建 (chain.rs)                 │
│ • DP 评分 + 贪心剥离                  │
│ • max_chains_per_contig 限制          │
└──────────────────────────────────────┘
    │
    ▼
┌──────────────────────────────────────┐
│ SW 扩展 (extend.rs + sw.rs)          │
│ • 带状仿射间隙局部对齐                │
│ • semi-global refinement              │
│ • 生成 CIGAR + NM                     │
└──────────────────────────────────────┘
    │
    ▼
┌──────────────────────────────────────┐
│ 候选去重与排序 (candidate.rs)         │
│ • 位置/方向去重                       │
│ • clip penalty 排序                   │
└──────────────────────────────────────┘
    │
    ▼
┌──────────────────────────────────────┐
│ 主/次比对输出 (pipeline.rs)           │
│ • max_alignments_per_read 限制        │
│ • FLAG 设置 (0/16/256)                │
└──────────────────────────────────────┘
    │
    ▼
  SAM 输出
```

---

## 技术栈

| 依赖 | 版本 | 用途 |
|------|------|------|
| **Rust** | 2021 Edition (MSRV 1.70) | 系统编程语言 |
| **clap** | 4.5 | CLI 参数解析 |
| **serde + bincode** | - | 索引序列化 |
| **rayon** | - | 数据并行 |
| **chrono** | - | 时间戳 |
| **criterion** | - | 基准测试 |
| **anyhow** | - | 错误处理 (CLI) |
| **tikv-jemallocator** | - | 内存分配器 (非 Windows) |

---

## 与 BWA/BWA-MEM 的差异

| 方面 | BWA (C) | bwa-rust |
|------|---------|----------|
| **索引格式** | `.bwt/.sa/.pac` 多文件 | 单一 `.fm` 文件 |
| **SA 构造** | DC3/IS O(n) | 倍增法 O(n log²n) |
| **MEM 查找** | 双向 BWT | 单向 backward_search |
| **链构建** | 复杂贪心+DP | 简化 DP + 贪心剥离 |
| **MAPQ** | 复杂统计模型 | 简化得分差比例模型 |
| **并行** | pthread | rayon |
| **配对端** | ✅ 支持 | 📋 计划中 (v0.2.0) |
| **BAM 输出** | ✅ 支持 | 📋 计划中 (v0.4.0) |

---

## 安全性保证

```toml
[lints.rust]
unsafe_code = "forbid"  # 全项目禁止 unsafe
```

- **零 unsafe 代码**：所有内存安全由编译器保证
- **jemalloc 分配器**：非 Windows 平台使用 jemalloc，提升多线程性能

---

## 性能优化

| 优化点 | 方法 | 效果 |
|--------|------|------|
| SA 存储 | 稀疏采样 (rate=4) | 内存减少 75% |
| SW 缓冲区 | `SwBuffer` 复用 | 减少热路径分配 |
| 多线程 | rayon 并行 | 多核线性加速 |
| 内存分配器 | jemalloc | 多线程吞吐提升 |

---

## 相关文档

- [索引构建详情](./index-building.zh-CN.md) — FM 索引构建流程
- [比对算法详情](./alignment.zh-CN.md) — 完整比对算法流程
- [快速入门教程](../tutorial/getting-started.zh-CN.md) — 快速上手
- [算法教程](../tutorial/algorithms.zh-CN.md) — 算法详解
