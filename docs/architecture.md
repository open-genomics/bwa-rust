# bwa-rust 架构文档

> 本文档详细介绍 bwa-rust 的模块设计、数据流和关键实现。

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

| 文件 | 功能 | 关键函数 |
|------|------|----------|
| `fasta.rs` | FASTA 解析 | `parse_fasta()`, `normalize_seq()` |
| `fastq.rs` | FASTQ 解析 | `parse_fastq_record()` |
| `sam.rs` | SAM 输出 | `write_header()`, `format_record()`, `format_unmapped()` |

#### FASTA 解析特性

- ✅ 多 contig 支持
- ✅ 自动归一化（大写、过滤非标准字符）
- ✅ 支持不同换行符（LF/CRLF）
- ✅ 空序列检测
- ✅ 重复 contig 名称检测

#### SAM 输出格式

```
@HD	VN:1.6	SO:unsorted
@SQ	SN:chr1	LN:1000
@PG	ID:bwa-rust	VN:0.1.0	CL:bwa-rust mem ...
read1	0	chr1	100	60	50M	*	0	ACGT...	IIII...	AS:i:100	XS:i:0	NM:i:2
```

---

### `index/` — 索引构建

| 文件 | 功能 | 复杂度 |
|------|------|--------|
| `sa.rs` | 后缀数组构建 | O(n log²n) |
| `bwt.rs` | BWT 构建 | O(n) |
| `fm.rs` | FM 索引核心 | O(n) 空间 |
| `builder.rs` | 构建入口 | - |

#### 后缀数组算法

```
倍增法 (Doubling Algorithm):

第 k 轮：按后缀前 2^k 字符排序
最终：得到完整后缀数组

示例：text = "banana$"
SA = [6, 5, 3, 1, 0, 4, 2]
     $  a  a  a  b  n  n
     $  na na na an an
```

#### FM 索引结构

```
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

#### 索引文件格式

```
.fm 文件结构 (bincode 序列化):

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

---

### `align/` — 比对算法

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

#### 比对流程

```
FASTQ read
    │
    ├─ 正向归一化 ──────────────┐
    │                          │
    ├─ 反向互补 ──────────────┐│
    │                         ││
    ▼                         ▼▼
┌──────────────────────────────────────┐
│ SMEM 种子查找 (seed.rs)               │
│ • 对每个位置找最长精确匹配             │
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
│ • 带状仿射间隙局部对齐                 │
│ • semi-global refinement             │
│ • 生成 CIGAR + NM                    │
└──────────────────────────────────────┘
    │
    ▼
┌──────────────────────────────────────┐
│ 候选去重与排序 (candidate.rs)         │
│ • 位置/方向去重                       │
│ • clip penalty 排序                  │
└──────────────────────────────────────┘
    │
    ▼
┌──────────────────────────────────────┐
│ 主/次比对输出 (pipeline.rs)           │
│ • max_alignments_per_read 限制        │
│ • FLAG 设置 (0/16/256)               │
└──────────────────────────────────────┘
    │
    ▼
  SAM 行
```

#### AlignOpt 配置

```rust
pub struct AlignOpt {
    // 打分参数
    pub match_score: i32,           // 默认 2
    pub mismatch_penalty: i32,      // 默认 1
    pub gap_open: i32,              // 默认 2
    pub gap_extend: i32,            // 默认 1
    pub clip_penalty: i32,          // 默认 1

    // 对齐参数
    pub band_width: usize,          // 默认 16
    pub score_threshold: i32,       // 默认 20
    pub min_seed_len: usize,        // 默认 19

    // 并行参数
    pub threads: usize,             // 默认 1

    // 内存防护
    pub max_occ: usize,             // 默认 500
    pub max_chains_per_contig: usize,   // 默认 5
    pub max_alignments_per_read: usize, // 默认 5
}
```

#### 内存防护机制

```
┌────────────────────────────────────────────────────────────┐
│                     三层内存防护                            │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ 1. max_occ = 500                                     │  │
│  │    跳过 SA 区间 > 500 的种子                          │  │
│  │    防止 poly-A 等重复序列展开数万位置                  │  │
│  └──────────────────────────────────────────────────────┘  │
│                         ↓                                  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ 2. max_chains_per_contig = 5                         │  │
│  │    每个 contig 最多提取 5 条链                        │  │
│  │    防止重复区域产生过多候选                           │  │
│  └──────────────────────────────────────────────────────┘  │
│                         ↓                                  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ 3. max_alignments_per_read = 5                       │  │
│  │    每个 read 最多输出 5 个比对结果                     │  │
│  │    控制最终输出规模                                   │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

---

### `util/` — 工具函数

| 文件 | 功能 | 关键函数 |
|------|------|----------|
| `dna.rs` | DNA 编码 | `to_alphabet()`, `from_alphabet()`, `revcomp()` |

#### DNA 字母表编码

```
编码表:
  0 → $ (sentinel)
  1 → A
  2 → C
  3 → G
  4 → T
  5 → N (未知)

归一化:
  A/a → A
  C/c → C
  G/g → G
  T/t/U/u → T
  其他 → N
```

---

### `error.rs` — 错误处理

```rust
pub enum BwaError {
    Io(io::Error),           // I/O 错误
    IndexFormat(String),     // 索引格式错误
    IndexBuild(String),      // 索引构建错误
    Align(String),           // 比对错误
    Parse(String),           // 解析错误
}

pub type BwaResult<T> = Result<T, BwaError>;
```

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

## 安全性保证

```toml
[lints.rust]
unsafe_code = "forbid"  # 全项目禁止 unsafe
```

- **零 unsafe 代码**：所有内存安全由编译器保证
- **jemalloc 分配器**：非 Windows 平台使用 jemalloc，提升多线程性能

---

## 测试覆盖

| 类型 | 数量 | 位置 |
|------|------|------|
| 单元测试 | 151 | 各模块 `#[cfg(test)]` |
| 集成测试 | 11 | `tests/integration.rs` |
| 模块测试 | 5 | `src/align/mod.rs` 等 |
| 文档测试 | 1 | `src/lib.rs` |
| **总计** | **168** | - |

### 关键测试用例

```bash
# 端到端测试
cargo test e2e_build_index_and_exact_search
cargo test e2e_seed_chain_align_exact
cargo test e2e_seed_chain_align_with_mismatch

# 比对场景
cargo test align_single_read_mapped
cargo test align_single_read_revcomp
cargo test align_single_read_unmapped

# 参数验证
cargo test align_opt_default_is_valid
cargo test align_opt_rejects_zero_band_width
```

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

| 文档 | 说明 |
|------|------|
| [tutorial.md](tutorial.md) | 从零实现教程 |
| [plan.md](plan.md) | 详细功能规划 |
| [../README.md](../README.md) | 项目介绍 |
