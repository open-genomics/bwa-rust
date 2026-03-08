# 架构设计

## 整体架构

bwa-rust 采用模块化设计，分为五个主要模块：

```
┌─────────────────────────────────────────────────────────┐
│                     main.rs (CLI)                       │
│              clap 命令行解析 + 调度                     │
├──────────┬──────────┬──────────────┬──────────┬─────────┤
│   io/    │  index/  │    align/    │  util/   │ error/  │
│  FASTA   │   SA     │  Seed        │ DNA 编码 │ BwaError│
│  FASTQ   │   BWT    │  Chain       │ 反向互补 │ BwaResult│
│  SAM     │   FM     │  SW/Extend   │          │         │
│          │  Builder │  Candidate   │          │         │
│          │          │  MAPQ        │          │         │
│          │          │  Pipeline    │          │         │
└──────────┴──────────┴──────────────┴──────────┴─────────┘
```

## 模块划分

### `io/` — 输入输出

- **`fasta.rs`** — FASTA 格式解析器：支持多 contig、描述行、不同换行符，过滤非标准字符
- **`fastq.rs`** — FASTQ 格式解析器：解析 4 行制记录，校验 seq/qual 长度一致性
- **`sam.rs`** — SAM 格式输出：header 写入、mapped/unmapped 记录生成（含 AS/XS/NM 标签）

### `index/` — 索引构建

- **`sa.rs`** — 后缀数组：倍增法构建 O(n log²n)，支持多 sentinel
- **`bwt.rs`** — BWT：从 SA 直接推导
- **`fm.rs`** — FM 索引：C 表 + 分块 Occ 采样、backward search、稀疏 SA 采样、bincode 序列化
- **`builder.rs`** — 从 FASTA 一键构建 FM 索引的入口

### `align/` — 序列比对

- **`seed.rs`** — SMEM 种子查找：逐位延伸查找最长精确匹配
- **`chain.rs`** — 种子链构建（DP）与过滤（贪心剥离 + 覆盖重叠去除）
- **`sw.rs`** — 带状仿射间隙 Smith-Waterman：可复用缓冲区，生成 CIGAR 和 NM
- **`extend.rs`** — 链→完整对齐：左右 SW 扩展 + CIGAR 合并
- **`candidate.rs`** — 对齐候选收集与去重
- **`mapq.rs`** — MAPQ 估算：基于主次候选得分差
- **`pipeline.rs`** — 批量并行对齐 pipeline（rayon）+ SAM 输出

### `error/` — 错误类型

- `BwaError` 枚举（Io / IndexFormat / IndexBuild / Align / Parse）+ `BwaResult<T>` 类型别名

### `util/` — 工具函数

- **`dna.rs`** — 字母表编码 `{0:$, 1:A, 2:C, 3:G, 4:T, 5:N}`、归一化、反向互补

## 索引文件格式

`.fm` 文件使用 bincode 序列化：

| 字段 | 类型 | 说明 |
|------|------|------|
| magic | `u64` | `0x424D_4146_4D5F5253`（"BWAFM_RS"） |
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

## 对齐算法流程

```
FASTQ read
    │
    ├─ 正向归一化 ──────────────┐
    │                           │
    ├─ 反向互补归一化 ─────────┐│
    │                          ││
    ▼                          ▼▼
  SMEM 种子查找 ──→ 多链构建 ──→ 链过滤
                                   │
                                   ▼
                            链→对齐（SW 扩展）
                                   │
                                   ▼
                        候选去重 + 排序
                                   │
                                   ▼
                    主比对 + secondary 输出
                                   │
                                   ▼
                              SAM 行
```

## 与 BWA/BWA-MEM 的主要差异

| 方面 | BWA (C) | bwa-rust |
|------|---------|----------|
| 索引格式 | `.bwt/.sa/.pac` 等多文件 | 单一 `.fm` 文件（bincode） |
| SA 构造 | DC3/IS 算法 | 倍增法 |
| MEM 查找 | `bwt_smem1`（双向 BWT） | 单向 backward_search 逐位延伸 |
| 链构建 | 复杂贪心+DP | 简化 DP |
| MAPQ | 复杂统计模型 | 简化得分差比例模型 |
| 并行 | pthread | rayon |
| 配对端 | 支持 | 目前仅单端 |

## 技术栈

- **Rust 2021 Edition** — 系统编程语言
- **clap 4.5** — CLI 参数解析
- **serde + bincode** — 索引序列化
- **rayon** — 数据并行
- **chrono** — 时间戳
- **criterion** — 基准测试
- **anyhow** — 错误处理
