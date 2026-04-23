# 架构概览

> bwa-rust 采用模块化设计，清晰分离索引构建、比对算法和输入输出。

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

---

## 模块详解

### `io/` — 输入输出

| 文件 | 功能 |
|------|------|
| `fasta.rs` | FASTA 解析：多 contig、自动归一化、空序列检测 |
| `fastq.rs` | FASTQ 解析：4 行制记录、seq/qual 长度校验 |
| `sam.rs` | SAM 输出：header 写入、CIGAR/MAPQ/AS/XS/NM 标签 |

### `index/` — 索引构建

| 文件 | 功能 | 复杂度 |
|------|------|--------|
| `sa.rs` | 后缀数组：倍增法构建 | O(n log²n) |
| `bwt.rs` | BWT：从 SA 直接推导 | O(n) |
| `fm.rs` | FM 索引：C 表 + Occ 采样 + 稀疏 SA | O(n) 空间 |
| `builder.rs` | 从 FASTA 一键构建入口 | - |

### `align/` — 比对算法

| 文件 | 功能 |
|------|------|
| `mod.rs` | `AlignOpt` 配置：打分参数、内存限制 |
| `seed.rs` | SMEM 种子查找，`max_occ` 过滤 |
| `chain.rs` | 种子链构建与过滤，`max_chains` 限制 |
| `sw.rs` | 带状仿射间隙 Smith-Waterman |
| `extend.rs` | 链扩展 + semi-global refinement |
| `candidate.rs` | 候选收集与去重，clip penalty 排序 |
| `mapq.rs` | MAPQ 估算：得分差比例模型 |
| `pipeline.rs` | 批量并行比对 pipeline，`max_alignments` 限制 |

### `util/` — 工具函数

| 文件 | 功能 |
|------|------|
| `dna.rs` | DNA 编码：`{0:$, 1:A, 2:C, 3:G, 4:T, 5:N}`、归一化、反向互补 |

---

## 内存防护机制

为防止高度重复序列导致内存爆炸，引入三层限制：

```
┌────────────────────────────────────────────────────────────┐
│                     三层内存防护                            │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  1. max_occ = 500                                         │
│     跳过 SA 区间 > 500 的种子                              │
│     防止 poly-A 等重复序列展开数万位置                      │
│                         ↓                                  │
│  2. max_chains_per_contig = 5                             │
│     每个 contig 最多提取 5 条链                            │
│     防止重复区域产生过多候选                               │
│                         ↓                                  │
│  3. max_alignments_per_read = 5                           │
│     每个 read 最多输出 5 个比对结果                        │
│     控制最终输出规模                                       │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

| 参数 | 默认值 | 作用 |
|------|--------|------|
| `max_occ` | 500 | 跳过 SA 区间超过此值的种子 |
| `max_chains_per_contig` | 5 | 每个 contig 最多提取的链数 |
| `max_alignments_per_read` | 5 | 每个 read 最终输出的比对数量上限 |

---

## 索引文件格式

`.fm` 文件使用 bincode 序列化：

| 字段 | 类型 | 说明 |
|------|------|------|
| magic | `u64` | `0x424D_4146_4D5F5253` ("BWAFM_RS") |
| version | `u32` | 格式版本（当前为 2） |
| sigma | `u8` | 字母表大小（6） |
| block | `u32` | Occ 采样块大小 |
| c | `Vec<u32>` | C 表 |
| bwt | `Vec<u8>` | BWT 序列 |
| occ_samples | `Vec<u32>` | Occ 采样表 |
| sa | `Vec<u32>` | SA（完整或稀疏） |
| sa_sample_rate | `u32` | 稀疏采样间隔 |
| contigs | `Vec<Contig>` | contig 元信息 |
| text | `Vec<u8>` | 原始编码文本 |
| meta | `Option<IndexMeta>` | 构建元数据 |

---

## 与 BWA 的差异

| 方面 | BWA (C) | bwa-rust |
|------|---------|----------|
| 索引格式 | `.bwt/.sa/.pac` 多文件 | 单一 `.fm` 文件 |
| SA 构造 | DC3/IS O(n) | 倍增法 O(n log²n) |
| MEM 查找 | 双向 BWT | 单向 backward_search |
| 链构建 | 复杂贪心+DP | 简化 DP + 贪心剥离 |
| MAPQ | 复杂统计模型 | 简化得分差比例模型 |
| 并行 | pthread | rayon |
| 配对端 | ✅ 支持 | 📋 v0.2.0 |
| BAM 输出 | ✅ 支持 | 📋 v0.4.0 |

---

## 技术栈

| 依赖 | 用途 |
|------|------|
| Rust 2021 (MSRV 1.70) | 系统编程语言 |
| clap 4.5 | CLI 参数解析 |
| serde + bincode | 索引序列化 |
| rayon | 数据并行 |
| criterion | 基准测试 |
| tikv-jemallocator | 内存分配器 |

---

## 安全性保证

```toml
[lints.rust]
unsafe_code = "forbid"  # 全项目禁止 unsafe
```

- **零 unsafe 代码**：所有内存安全由编译器保证
- **jemalloc 分配器**：非 Windows 平台使用 jemalloc

---

## 测试覆盖

| 类型 | 数量 |
|------|------|
| 单元测试 | 188 |
| 集成测试 | 11 |
| 文档测试 | 2 |
| **总计** | **201** |
