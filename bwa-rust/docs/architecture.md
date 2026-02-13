# bwa-rust 架构文档

## 整体架构

bwa-rust 是一个受 BWA/BWA-MEM 启发的 Rust 版序列比对器。项目采用模块化设计，分为四个主要模块：

```
┌─────────────────────────────────────────────────┐
│                   main.rs (CLI)                 │
│            clap 命令行解析 + 调度               │
├──────────┬──────────┬──────────┬────────────────┤
│   io/    │  index/  │  align/  │    util/       │
│  FASTA   │   SA     │  SMEM    │   DNA 编码     │
│  FASTQ   │   BWT    │  Chain   │   反向互补     │
│          │   FM     │  SW      │                │
│          │          │  SAM输出  │                │
└──────────┴──────────┴──────────┴────────────────┘
```

## 模块划分

### `io/` — 输入输出

- **`fasta.rs`** — FASTA 格式解析器
  - 支持多 contig、描述行、不同换行符
  - 过滤非标准字符，大写归一化
- **`fastq.rs`** — FASTQ 格式解析器
  - 解析 4 行制 FASTQ 记录（header/seq/+/qual）
  - 校验 seq/qual 长度一致性

### `index/` — 索引构建

- **`sa.rs`** — 后缀数组（Suffix Array）
  - 倍增法构建，O(n log²n)
  - 支持含多个 sentinel（$）的文本
- **`bwt.rs`** — BWT（Burrows-Wheeler Transform）
  - 从 SA 直接推导 BWT
- **`fm.rs`** — FM 索引
  - C 表（累计频率）+ 分块 Occ 采样
  - 反向精确搜索（backward_search）
  - SA 区间到文本位置映射（支持稀疏 SA 采样）
  - 序列化/反序列化（bincode，含 magic + 版本号）
  - 可选构建元数据（参考文件名、命令参数、时间戳）

### `align/` — 序列比对

- **`seed.rs`** — SMEM 种子查找
  - 对 read 的每个位置查找最长精确匹配（MEM）
  - 过滤被包含的 MEM，保留超级最大精确匹配（SMEM）
  - `AlnReg` 结构体：描述一个对齐区域
- **`chain.rs`** — 种子链构建与过滤
  - DP 方法构建最佳链
  - 贪心剥离提取多条链
  - 链过滤：按得分比例和 read 覆盖重叠去除弱链/冗余链
- **`sw.rs`** — Smith-Waterman 局部对齐
  - 带状仿射间隙 DP（match/mismatch/gap_open/gap_extend）
  - 可复用工作缓冲区（SwBuffer），减少内存分配
  - 回溯生成 CIGAR 和 edit distance (NM)
- **`mod.rs`** — 对齐主流程
  - 批量读取 FASTQ，rayon 并行处理
  - 正向 + 反向互补双向比对
  - 多链候选生成 → 去重 → 排序
  - 主/次要比对 FLAG 设置（0x10 反向、0x100 secondary）
  - MAPQ 估算（基于主次候选得分差）
  - SAM 输出（含 @HD/@SQ/@PG header，AS/XS/NM 标签）

### `util/` — 工具函数

- **`dna.rs`** — DNA 碱基处理
  - 字母表编码：`{0:$, 1:A, 2:C, 3:G, 4:T, 5:N}`
  - 归一化：A/C/G/T/U/N，其他→N
  - 反向互补（revcomp）

## 索引格式

`.fm` 文件使用 bincode 序列化，包含：

| 字段 | 类型 | 说明 |
|------|------|------|
| magic | u64 | `0x424D_4146_4D5F5253`（"BWAFM_RS"） |
| version | u32 | 格式版本（当前为 2） |
| sigma | u8 | 字母表大小（6） |
| block | u32 | Occ 采样块大小 |
| c | Vec\<u32\> | C 表 |
| bwt | Vec\<u8\> | BWT 序列 |
| occ_samples | Vec\<u32\> | Occ 采样表 |
| sa | Vec\<u32\> | SA（完整或稀疏） |
| sa_sample_rate | u32 | 稀疏采样间隔 |
| contigs | Vec\<Contig\> | contig 元信息 |
| text | Vec\<u8\> | 原始编码文本 |
| meta | Option\<IndexMeta\> | 构建元数据 |

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
| 索引格式 | .bwt/.sa/.pac 等多文件 | 单一 .fm 文件（bincode） |
| SA 构造 | DC3/IS 算法 | 倍增法 |
| MEM 查找 | bwt_smem1（双向 BWT） | 单向 backward_search 逐位延伸 |
| 链构建 | 复杂贪心+DP | 简化 DP |
| MAPQ | 复杂统计模型 | 简化得分差比例模型 |
| 并行 | pthread | rayon |
| 配对端 | 支持 | 目前仅单端 |

## 技术栈

- **Rust 2021 Edition**
- **clap 4.5** — CLI 参数解析
- **serde + bincode** — 索引序列化
- **rayon** — 数据并行
- **chrono** — 时间戳
- **criterion** — 基准测试
- **anyhow** — 错误处理
