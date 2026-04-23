# CLAUDE.md

> 本文件为 Claude Code (claude.ai/code) 提供项目上下文和开发指导。

---

## Project Philosophy: OpenSpec-Driven Development

本项目采用 **OpenSpec** 框架进行规范驱动开发。所有的代码实现必须以 `openspec/specs/` 目录下的规范文档为唯一事实来源（Single Source of Truth）。

### OpenSpec Workflow

使用 OpenSpec CLI 和 `/opsx` 命令管理变更：

| 命令 | 说明 |
|------|------|
| `/opsx:propose <name>` | 创建新变更提案，生成 proposal.md, design.md, tasks.md |
| `/opsx:apply` | 执行 tasks.md 中的实施任务 |
| `/opsx:archive` | 归档已完成的变更到 `openspec/changes/archive/` |
| `/opsx:explore` | 探索代码库，理解问题后再提案 |

### AI Agent Workflow

1. **审查 Spec**: 在编写代码前，先阅读 `openspec/specs/` 下的相关文档
2. **Propose First**: 新功能或接口变更必须先用 `/opsx:propose` 创建提案
3. **遵守 Spec**: 代码实现必须 100% 遵守 Spec 定义
4. **Archive**: 完成后用 `/opsx:archive` 归档变更

> **必读**: [docs/development/ai-workflow.md](docs/development/ai-workflow.md) — 完整工作流程规范

---

## 项目概览

**bwa-rust** 是一个用 Rust 从零实现的 BWA-MEM 风格 DNA 短序列比对器。

### 核心特性

- **FM 索引构建**：后缀数组 + BWT + 稀疏 SA 采样，序列化为单一 `.fm` 文件
- **BWA-MEM 风格比对**：SMEM 种子查找 → 种子链构建 → 带状 Smith-Waterman
- **标准 SAM 输出**：完整的 header、CIGAR、MAPQ、AS/XS/NM 标签
- **多线程并行**：基于 rayon 的 reads 级并行处理
- **内存安全**：零 unsafe 代码，jemalloc 分配器
- **可配置限制**：`max_occ`、`max_chains_per_contig`、`max_alignments_per_read` 防止内存爆炸

### 项目状态

| 指标 | 状态 |
|------|------|
| 版本 | v0.2.0 |
| 测试 | 188 单元 + 11 集成 + 2 文档测试 ✅ |
| CI | GitHub Actions (fmt → clippy → test → release) |
| 文档 | 架构文档、教程、VitePress 站点 |
| 规范 | `openspec/specs/` 目录 (OpenSpec 工作流) |

---

## 常用命令

```bash
# === 构建 ===
cargo build --release              # 生产构建

# === 测试 ===
cargo test                          # 运行所有测试
cargo test --lib                    # 仅库测试
cargo test --test integration       # 仅集成测试
cargo test <test_name>              # 运行单个测试
cargo test <test_name> -- --nocapture  # 显示输出

# === 代码质量 ===
cargo fmt --all -- --check          # 检查格式
cargo clippy --all-targets --all-features -- -D warnings  # Lint

# === 性能分析 ===
cargo bench                         # 基准测试

# === CLI 使用 ===
cargo run -- index <ref.fa> -o <prefix>              # 构建索引
cargo run -- align -i <index.fm> <reads.fq>          # 预构建索引比对
cargo run -- mem <ref.fa> <reads.fq>                 # 一步比对
cargo run -- mem <ref.fa> <reads.fq> -t 4 -o out.sam # 多线程输出到文件
cargo run --example simple_align                     # 运行示例
```

---

## 架构设计

### 三层模块架构

```
┌─────────────────────────────────────────────────────────────┐
│                    CLI Layer (main.rs)                       │
│                  clap 命令解析 + 调度                        │
├─────────────┬─────────────┬──────────────┬──────────────────┤
│    io/      │   index/    │    align/    │      util/       │
│  FASTA/FASTQ│   SA/BWT    │  Seed/Chain  │   DNA 编码       │
│  SAM 输出   │   FM Index  │  SW/Pipeline │   反向互补       │
└─────────────┴─────────────┴──────────────┴──────────────────┘
```

### 数据流

```
FASTA/FASTQ → FM 索引构建 → SMEM 种子 → 种子链 → SW 对齐 → SAM 输出
```

### 模块职责

| 模块 | 文件 | 职责 |
|------|------|------|
| **io/** | `fasta.rs` | FASTA 解析（多 contig、归一化） |
| | `fastq.rs` | FASTQ 解析（seq/qual 校验） |
| | `sam.rs` | SAM 格式输出（header/record/unmapped） |
| **index/** | `sa.rs` | 后缀数组（倍增法 O(n log²n)） |
| | `bwt.rs` | BWT 构建 |
| | `fm.rs` | FM 索引（C 表 + Occ 采样 + 序列化） |
| | `builder.rs` | 索引构建入口 |
| **align/** | `mod.rs` | `AlignOpt` 配置、常量定义 |
| | `seed.rs` | SMEM 种子查找（支持 `max_occ` 过滤） |
| | `chain.rs` | 种子链构建与过滤（DP + 贪心剥离） |
| | `sw.rs` | 带状 Smith-Waterman（仿射间隙，`SwBuffer` 复用） |
| | `extend.rs` | 链到完整对齐扩展 |
| | `candidate.rs` | 候选管理、semi-global refinement、去重 |
| | `mapq.rs` | MAPQ 估算（BWA 风格） |
| | `pipeline.rs` | 完整比对流水线（rayon 并行） |
| **util/** | `dna.rs` | DNA 编码/解码/反向互补 |
| **error** | `error.rs` | `BwaError` 枚举 + `BwaResult<T>` |

---

## 关键设计决策

### 安全性

```toml
[lints.rust]
unsafe_code = "forbid"  # 全项目禁止 unsafe
```

### 性能优化

| 优化点 | 实现方式 |
|--------|----------|
| 内存分配 | `SwBuffer` 缓冲区复用，减少热路径分配 |
| 多线程 | rayon 并行处理 reads，自定义线程池 |
| 内存分配器 | 非 Windows 使用 jemalloc（提升 rayon 场景性能） |
| SA 存储 | 稀疏采样（`sa_sample_rate`）平衡内存与查询速度 |
| 重复序列 | `max_occ` 过滤高度重复种子，防止内存爆炸 |

### 索引格式

```
.fm 文件结构（bincode 序列化）：
┌──────────────────────────────────────┐
│ magic: u64 = 0x424D4146_4D5F5253     │ "BWAFM_RS"
│ version: u32 = 2                     │ 格式版本
│ sigma: u8 = 6                        │ 字母表大小
│ block: u32                           │ Occ 采样块大小
│ c: Vec<u32>                          │ C 表
│ bwt: Vec<u8>                         │ BWT 序列
│ occ_samples: Vec<u32>                │ Occ 采样表
│ sa: Vec<u32>                         │ SA（完整或稀疏）
│ sa_sample_rate: u32                  │ 稀疏采样间隔
│ contigs: Vec<Contig>                 │ contig 元信息
│ text: Vec<u8>                        │ 原始编码文本
│ meta: Option<IndexMeta>              │ 构建元数据
└──────────────────────────────────────┘
```

---

## AlignOpt 配置参数

```rust
pub struct AlignOpt {
    pub match_score: i32,               // 匹配得分 (默认: 2)
    pub mismatch_penalty: i32,          // 错配罚分 (默认: 1)
    pub gap_open: i32,                  // Gap 开启罚分 (默认: 2)
    pub gap_extend: i32,                // Gap 扩展罚分 (默认: 1)
    pub clip_penalty: i32,              // 软剪切惩罚（候选排序）(默认: 1)
    pub band_width: usize,              // 带状 SW 带宽 (默认: 16)
    pub score_threshold: i32,           // 最低输出得分 (默认: 20)
    pub min_seed_len: usize,            // 最小种子长度 (默认: 19)
    pub threads: usize,                 // 线程数 (默认: 1)
    pub max_chains_per_contig: usize,   // 每 contig 最大链数 (默认: 5)
    pub max_alignments_per_read: usize, // 每 read 最大输出数 (默认: 5)
    pub max_occ: usize,                 // 最大种子出现次数 (默认: 500)
    pub zdrop: i32,                     // Z-drop 阈值 (默认: 100)
}
```

**真值来源**: `src/align/mod.rs` 中的 `AlignOpt::default()` 实现。

`validate()` 方法校验所有参数的有效性。

---

## 错误处理

```rust
// 库模式：结构化错误
pub enum BwaError {
    Io(io::Error),
    IndexFormat(String),
    IndexBuild(String),
    Align(String),
    Parse(String),
}

// CLI 模式：anyhow 传播
use anyhow::Result;
```

---

## 代码质量标准

### Lint 配置 (`clippy.toml`)

```toml
cognitive-complexity-threshold = 30
max-fn-lines = 200
max-fn-params = 8
msrv = "1.70"
```

### CI 流程

```yaml
jobs:
  ci:
    steps:
      - cargo fmt --all -- --check
      - cargo clippy --all-targets --all-features -- -D warnings
      - cargo test
      - cargo build --release
```

### 提交规范

遵循 [Conventional Commits](https://www.conventionalcommits.org/)：

- `feat:` 新功能
- `fix:` Bug 修复
- `docs:` 文档变更
- `refactor:` 代码重构
- `test:` 测试相关
- `perf:` 性能优化

---

## 测试覆盖

### 测试类型

| 类型 | 数量 | 位置 |
|------|------|------|
| 单元测试 | 188 | 各模块 `#[cfg(test)]` |
| 集成测试 | 11 | `tests/integration.rs` |
| 文档测试 | 2 | `src/lib.rs` 等 |

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

## 扩展开发

### 添加新的比对参数

1. 在 `src/align/mod.rs` 的 `AlignOpt` 结构体中添加字段
2. 在 `validate()` 方法中添加校验逻辑
3. 在 `src/main.rs` 的 CLI 参数中添加对应选项
4. 添加单元测试验证

### 添加新的输出格式

1. 在 `src/io/` 下创建新模块
2. 实现 `write_header` 和 `write_record` 函数
3. 在 `src/align/pipeline.rs` 中集成

### 添加配对端支持（未来）

参见 `docs/plan.md` 中的 PE 对齐设计。

---

## 相关文档

| 文档 | 说明 |
|------|------|
| [AGENTS.md](AGENTS.md) | AI 编程助手完整指南 |
| [README.md](README.md) | 项目介绍（英文） |
| [README.zh-CN.md](README.zh-CN.md) | 项目介绍（中文） |
| [openspec/specs/](openspec/specs/) | **规范文档 (Single Source of Truth)** |
| [openspec/config.yaml](openspec/config.yaml) | OpenSpec 项目配置 |
| [docs/](docs/) | 用户教程与架构文档 |
| [ROADMAP.md](ROADMAP.md) | 开发路线图 |
| [CHANGELOG.md](CHANGELOG.md) | 变更日志 |
| [CONTRIBUTING.md](CONTRIBUTING.md) | 贡献指南 |
