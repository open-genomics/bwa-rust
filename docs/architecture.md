# bwa-rust 架构文档

## 整体架构

bwa-rust 是一个受 BWA/BWA-MEM 启发的 Rust 版序列比对器。项目采用模块化设计，分为五个主要模块：

```
┌─────────────────────────────────────────────────────────────┐
│                     main.rs (CLI)                           │
│              clap 命令行解析 + 调度                          │
├──────────┬──────────┬──────────────┬──────────┬─────────────┤
│   io/    │  index/  │    align/    │  util/   │   error/    │
│  FASTA   │   SA     │  Seed        │ DNA 编码 │  BwaError   │
│  FASTQ   │   BWT    │  Chain       │ 反向互补 │  BwaResult  │
│  SAM     │   FM     │  SW/Extend   │          │             │
│          │  Builder │  Candidate   │          │             │
│          │          │  MAPQ        │          │             │
│          │          │  Pipeline    │          │             │
└──────────┴──────────┴──────────────┴──────────┴─────────────┘
```

## 模块划分

### `io/` — 输入输出

| 文件 | 职责 |
|------|------|
| `fasta.rs` | FASTA 格式解析器：支持多 contig、描述行、不同换行符，过滤非标准字符，大写归一化 |
| `fastq.rs` | FASTQ 格式解析器：解析 4 行制记录（header/seq/+/qual），校验 seq/qual 长度一致性 |
| `sam.rs` | SAM 格式输出：`write_header`（@HD/@SQ/@PG）、`format_unmapped`（FLAG=4）、`format_record`（含 AS/XS/NM 标签）|

### `index/` — 索引构建

| 文件 | 职责 |
|------|------|
| `sa.rs` | 后缀数组（Suffix Array）：倍增法构建，O(n log²n)，支持含多个 sentinel（$）的文本 |
| `bwt.rs` | BWT（Burrows-Wheeler Transform）：从 SA 直接推导 |
| `fm.rs` | FM 索引：C 表（累计频率）+ 分块 Occ 采样、反向精确搜索（backward_search）、SA 区间到文本位置映射（支持稀疏 SA 采样）、序列化/反序列化（bincode，含 magic + 版本号）、可选构建元数据 |
| `builder.rs` | 索引构建入口：从 FASTA 文件或 BufRead 一键构建 FM 索引，自动完成序列归一化 → 编码 → SA → BWT → FMIndex 全流程，空输入/空序列检测与错误上报 |

### `align/` — 序列比对

| 文件 | 职责 |
|------|------|
| `mod.rs` | 模块声明与核心类型：`AlignOpt` 配置结构（打分参数、带宽、种子长度、线程数、内存限制）、统一 re-export 各子模块公开 API |
| `seed.rs` | SMEM 种子查找：对 read 的每个位置查找最长精确匹配（MEM）、过滤被包含的 MEM 保留超级最大精确匹配（SMEM）、`max_occ` 过滤高度重复种子 |
| `chain.rs` | 种子链构建与过滤：DP 方法构建最佳链、贪心剥离提取多条链、链过滤（按得分比例和 read 覆盖重叠去除弱链/冗余链）、`max_chains_per_contig` 限制 |
| `sw.rs` | Smith-Waterman 局部对齐：带状仿射间隙 DP（match/mismatch/gap_open/gap_extend）、可复用工作缓冲区（SwBuffer）减少内存分配、回溯生成 CIGAR 和 edit distance (NM) |
| `extend.rs` | 链→完整对齐：`chain_to_alignment` 从种子链生成完整 CIGAR（左右扩展 + 间隙 SW）、CIGAR 操作合并（`push_run`）与 NM 计算、semi-global refinement 优化 indel 检测 |
| `candidate.rs` | 对齐候选管理：`AlignCandidate` 结构体（内部候选表示）、`collect_candidates` 从种子链收集候选对齐、`dedup_candidates` 位置/方向相同的候选去重、clip penalty 参与排序 |
| `mapq.rs` | MAPQ 估算：BWA 风格 `compute_mapq`，基于主次候选得分差的映射质量估算 |
| `pipeline.rs` | 对齐 pipeline 与 SAM 输出：批量读取 FASTQ、rayon 并行处理、正向 + 反向互补双向比对、主/次要比对 FLAG 设置（0x10 反向、0x100 secondary）、`max_alignments_per_read` 限制输出数量 |

### `error/` — 错误类型

| 文件 | 职责 |
|------|------|
| `error.rs` | 自定义错误：`BwaError` 枚举（Io / IndexFormat / IndexBuild / Align / Parse）、`BwaResult<T>` 类型别名，为库模式调用提供结构化错误信息 |

### `util/` — 工具函数

| 文件 | 职责 |
|------|------|
| `dna.rs` | DNA 碱基处理：字母表编码 `{0:$, 1:A, 2:C, 3:G, 4:T, 5:N}`、归一化（A/C/G/T/U/N，其他→N）、反向互补（revcomp）|

## AlignOpt 配置参数

```rust
pub struct AlignOpt {
    // 打分参数
    pub match_score: i32,           // 匹配得分（默认 2）
    pub mismatch_penalty: i32,      // 错配罚分（默认 1）
    pub gap_open: i32,              // Gap 开启罚分（默认 2）
    pub gap_extend: i32,            // Gap 扩展罚分（默认 1）
    pub clip_penalty: i32,          // 软剪切惩罚，用于候选排序（默认 1）

    // 对齐参数
    pub band_width: usize,          // 带状 SW 带宽（默认 16）
    pub score_threshold: i32,       // 最低输出得分（默认 20）
    pub min_seed_len: usize,        // 最小种子长度（默认 19）

    // 并行参数
    pub threads: usize,             // 线程数（默认 1）

    // 内存防护限制
    pub max_chains_per_contig: usize,   // 每 contig 最大链数（默认 5）
    pub max_alignments_per_read: usize, // 每 read 最大输出数（默认 5）
    pub max_occ: usize,                 // 最大种子出现次数（默认 500）
}
```

### 内存防护机制

为防止高度重复序列导致内存爆炸，引入三层限制：

| 限制 | 默认值 | 作用 |
|------|--------|------|
| `max_occ` | 500 | 跳过 SA 区间超过此值的种子，避免展开数万个位置 |
| `max_chains_per_contig` | 5 | 每个 contig 最多提取的链数，防止重复区域产生过多候选 |
| `max_alignments_per_read` | 5 | 每个 read 最终输出的比对数量上限 |

### 参数验证

`AlignOpt::validate()` 方法校验所有参数的有效性：

- `band_width > 0`
- 所有打分参数 `>= 0`
- `threads > 0`
- `max_chains_per_contig > 0`
- `max_alignments_per_read > 0`

## 索引格式

`.fm` 文件使用 bincode 序列化，包含：

| 字段 | 类型 | 说明 |
|------|------|------|
| magic | `u64` | `0x424D_4146_4D5F5253`（"BWAFM_RS"）|
| version | `u32` | 格式版本（当前为 2）|
| sigma | `u8` | 字母表大小（6）|
| block | `u32` | Occ 采样块大小 |
| c | `Vec<u32>` | C 表 |
| bwt | `Vec<u8>` | BWT 序列 |
| occ_samples | `Vec<u32>` | Occ 采样表 |
| sa | `Vec<u32>` | SA（完整或稀疏）|
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
   (max_occ 过滤)    (贪心剥离)    (max_chains)
                                   │
                                   ▼
                            链→对齐（SW 扩展）
                           (semi-global refinement)
                                   │
                                   ▼
                        候选去重 + 排序
                        (clip penalty)
                                   │
                                   ▼
                    主比对 + secondary 输出
                    (max_alignments)
                                   │
                                   ▼
                              SAM 行
```

## 与 BWA/BWA-MEM 的主要差异

| 方面 | BWA (C) | bwa-rust |
|------|---------|----------|
| 索引格式 | `.bwt/.sa/.pac` 等多文件 | 单一 `.fm` 文件（bincode）|
| SA 构造 | DC3/IS 算法 O(n) | 倍增法 O(n log²n) |
| MEM 查找 | `bwt_smem1`（双向 BWT）| 单向 backward_search 逐位延伸 |
| 链构建 | 复杂贪心+DP | 简化 DP + 贪心剥离 |
| MAPQ | 复杂统计模型 | 简化得分差比例模型 |
| 并行 | pthread | rayon |
| 配对端 | 支持 | 计划中（v0.2.0）|
| BAM 输出 | 支持 | 计划中（v0.4.0）|

## 技术栈

| 依赖 | 版本 | 用途 |
|------|------|------|
| Rust | 2021 Edition | 系统编程语言（最低 1.70）|
| clap | 4.5 | CLI 参数解析 |
| serde + bincode | - | 索引序列化 |
| rayon | - | 数据并行 |
| chrono | - | 时间戳 |
| criterion | - | 基准测试 |
| anyhow | - | 错误处理（CLI）|
| tikv-jemallocator | - | 内存分配器（非 Windows）|

## 安全性保证

```toml
[lints.rust]
unsafe_code = "forbid"  # 全项目禁止 unsafe
```

- **零 unsafe 代码**：所有内存安全由编译器保证
- **jemalloc 分配器**：非 Windows 平台使用 jemalloc，提升多线程场景性能

## 测试覆盖

| 类型 | 数量 | 位置 |
|------|------|------|
| 单元测试 | 151 | 各模块 `#[cfg(test)]` |
| 集成测试 | 11 | `tests/integration.rs` |
| 模块测试 | 5 | `src/align/mod.rs` 等 |
| 文档测试 | 1 | `src/lib.rs` |
| **总计** | **168** | - |
