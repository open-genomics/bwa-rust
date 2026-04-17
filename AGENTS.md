# AGENTS.md

> AI 编程助手（Claude Code, Cursor, Windsurf, etc.）项目指南

---

## Project Philosophy: Spec-Driven Development (SDD)

本项目严格遵循**规范驱动开发（Spec-Driven Development）**范式。所有的代码实现必须以 `/specs` 目录下的规范文档为唯一事实来源（Single Source of Truth）。

---

## Directory Context (目录说明)

```
specs/                    # 核心：规范文档存放地
├── product/              # 产品功能定义与验收标准 (PRD)
├── rfc/                  # 技术设计文档与架构方案 (RFCs)
├── api/                  # 接口规范 (CLI、库 API)
└── testing/              # 测试策略与 BDD 测试用例规范

docs/                     # 用户文档与开发文档
├── architecture/         # 架构说明
├── tutorial/             # 用户教程
├── api/                  # API 使用指南
├── development/          # 开发指南
└── assets/               # 图片、UML 图等静态资源
```

| 目录 | 说明 |
|------|------|
| `/specs/product/` | 产品功能定义与验收标准 |
| `/specs/rfc/` | 技术设计文档 (RFCs)、架构方案 |
| `/specs/api/` | API 接口定义（CLI interface、library API） |
| `/specs/testing/` | BDD 测试用例规范与测试策略 |

---

## AI Agent Workflow Instructions (AI 工作流指令)

当你（AI）被要求开发一个新功能、修改现有功能或修复 Bug 时，**必须严格按照以下工作流执行，不可跳过任何步骤**：

### Step 1: 审查与分析 (Review Specs)

- 在编写任何代码之前，首先阅读 `/specs` 目录下相关的产品文档、RFC 和 API 定义。
- 如果用户指令与现有 Spec 冲突，**请立即停止编码**，并指出冲突点，询问用户是否需要先更新 Spec。

### Step 2: 规范优先 (Spec-First Update)

- 如果这是一个新功能，或者需要改变现有的接口/行为，**必须首先提议修改或创建相应的 Spec 文档**（如 RFC 或 API 文档）。
- 等待用户确认 Spec 的修改后，才能进入代码编写阶段。

### Step 3: 代码实现 (Implementation)

- 编写代码时，必须 100% 遵守 Spec 中的定义（包括变量命名、API 路径、数据类型、行为规范等）。
- 不要在代码中擅自添加 Spec 中未定义的功能（No Gold-Plating）。

### Step 4: 测试验证 (Test against Spec)

- 根据 `/specs` 中的验收标准（Acceptance Criteria）编写单元测试和集成测试。
- 确保测试用例覆盖了 Spec 中描述的所有边界情况。

---

## Code Generation Rules

- 任何对外部暴露的 API 变更，必须同步修改 `/specs/api/` 下的相关文档。
- 如果遇到不确定的技术细节，请查阅 `/specs/rfc/` 下的架构约定，不要自行捏造设计模式。
- 修改索引格式或序列化协议时，必须递增版本号并保持向后兼容。

---

## 项目范围

**bwa-rust** 是一个 BWA-MEM 风格的 DNA 短序列比对器的 Rust 实现。

**核心流水线**：

```
FASTA/FASTQ → FM 索引构建 → SMEM 种子 → 种子链 → SW 对齐 → SAM 输出
```

**开发原则**：

- 正确性和可重复性优先于巧妙的重构
- 偏好小范围、符合现有模式的本地修改
- 保持格式化、lint 和测试全部通过

---

## 工具链与版本

| 工具 | 版本要求 |
|------|----------|
| Rust | MSRV 1.70 |
| Cargo | 随 Rust 安装 |
| clap | 4.5 (CLI) |
| serde + bincode | 序列化 |
| rayon | 并行处理 |
| criterion | 基准测试 |

**支持平台**：Linux、macOS、Windows

---

## CI 流程

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo build --release
```

> ⚠️ 修改代码后，确保上述流程全部通过。

---

## 常用命令

### 构建

```bash
cargo build                    # Debug 构建
cargo build --release          # Release 构建
```

### 测试

```bash
cargo test                     # 运行所有测试
cargo test --lib               # 仅库测试
cargo test --test integration  # 仅集成测试
cargo test <test_name>         # 运行单个测试
cargo test <test_name> -- --nocapture  # 显示输出
```

### 代码质量

```bash
cargo fmt --all -- --check     # 检查格式
cargo fmt --all                # 应用格式
cargo clippy --all-targets --all-features -- -D warnings  # Lint
```

### 基准测试

```bash
cargo bench                    # 运行基准测试
```

### CLI 使用

```bash
# 构建 FM 索引
cargo run -- index <ref.fa> -o <prefix>

# 使用已有索引比对
cargo run -- align -i <prefix>.fm <reads.fq>

# 一步 BWA-MEM 风格运行
cargo run -- mem <ref.fa> <reads.fq>

# 多线程输出到文件
cargo run -- mem <ref.fa> <reads.fq> -t 4 -o out.sam

# 运行示例
cargo run --example simple_align
```

---

## 仓库结构

```
src/
├── main.rs          # CLI 入口（clap）
├── lib.rs           # 库入口 + 测试辅助函数
├── error.rs         # 结构化错误（BwaError / BwaResult<T>）
│
├── index/           # 索引构建
│   ├── sa.rs        # 后缀数组（倍增法）
│   ├── bwt.rs       # BWT 构建
│   ├── fm.rs        # FM 索引（C 表 + Occ 采样 + 序列化）
│   └── builder.rs   # 索引构建入口
│
├── align/           # 比对算法
│   ├── mod.rs       # AlignOpt 配置、常量定义
│   ├── seed.rs      # SMEM 种子查找
│   ├── chain.rs     # 种子链构建与过滤
│   ├── sw.rs        # 带状 Smith-Waterman
│   ├── extend.rs    # 链到完整对齐扩展
│   ├── candidate.rs # 候选管理与去重
│   ├── mapq.rs      # MAPQ 估算
│   └── pipeline.rs  # 完整比对流水线
│
├── io/              # 输入输出
│   ├── fasta.rs     # FASTA 解析
│   ├── fastq.rs     # FASTQ 解析
│   └── sam.rs       # SAM 格式输出
│
└── util/            # 工具函数
    └── dna.rs       # DNA 编码/解码/反向互补

tests/
└── integration.rs   # 端到端和跨模块测试

benches/
└── benchmarks.rs    # Criterion 基准测试
```

---

## 代码风格

### 通用规则

- Rust 2021 Edition 风格
- 最大行宽：120 字符（见 `rustfmt.toml`）
- 缩进：4 空格（禁用 Tab）
- 让 `cargo fmt` 处理格式化，避免手动对齐

### 导入顺序

```rust
// 1. 标准库
use std::io::BufRead;

// 2. 第三方 crate
use anyhow::Result;
use rayon::prelude::*;

// 3. crate 内部模块
use crate::index::fm::FMIndex;
use crate::util::dna;

// 4. 父模块
use super::{AlignOpt, SwParams};
```

### 命名约定

| 类型 | 风格 | 示例 |
|------|------|------|
| 类型/特质 | `UpperCamelCase` | `FMIndex`, `SwParams` |
| 函数/方法/变量 | `snake_case` | `build_sa`, `backward_search` |
| 常量 | `UPPER_SNAKE_CASE` | `FM_MAGIC`, `SIGMA` |
| 领域术语 | 简短精确 | `fm`, `sa`, `bwt`, `mapq`, `revcomp` |

### 类型设计

- 使用显式字段的纯结构体
- 仅派生需要的 trait：`Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Serialize`, `Deserialize`, `Default`
- 内存索引用 `usize`，持久化坐标用 `u32`
- `usize`/`u32` 边界使用 `try_from` 检查溢出
- 纯辅助函数标记 `#[must_use]`

---

## 实现指南

### 控制流

- 无效输入和空路径使用早期返回
- 逻辑保持本地化，仅在明显提高可读性时提取辅助函数
- 热路径复用缓冲区

### 禁止事项

```rust
// ❌ 禁止 unsafe（crate 配置强制）
unsafe { ... }

// ❌ 热路径中不必要的分配
let v: Vec<_> = iter.collect();  // 仅在必要时使用

// ❌ 更改性能敏感的默认行为
// SA 采样率、SW 缓冲区复用、rayon 批处理等
```

### 错误处理

```rust
// 库模式：src/error.rs
pub enum BwaError {
    Io(io::Error),
    IndexFormat(String),
    IndexBuild(String),
    Align(String),
    Parse(String),
}
pub type BwaResult<T> = Result<T, BwaError>;

// CLI/内部：anyhow
use anyhow::{anyhow, bail, Result};

// 错误信息包含具体上下文
bail!("invalid FM index file: BWT symbol out of range at {}", i);
```

### 输入验证

早期验证以下内容：

- 空 FASTA 条目
- 重复 contig 名
- 零块大小
- 格式错误的 FASTQ 记录
- 无效线程数

---

## 注释与文档

### 注释原则

- 仅在算法意图或不变量不明显时添加
- 解释"为什么"而非逐行叙述
- 公共 API 和数据结构使用简洁的文档注释

### 现有风格

- 代码包含中英文注释/文档
- 编辑附近代码时保持本地风格

---

## 测试要求

### 测试类型

| 变更范围 | 测试位置 |
|----------|----------|
| 本模块行为 | 模块内 `#[cfg(test)] mod tests` |
| 跨模块/端到端 | `tests/integration.rs` |

### 测试覆盖

- 边界情况：空输入、无效格式、边界坐标、反向互补行为、重复名称、得分阈值
- 使用精确断言而非松散的真值判断
- 测试中 `unwrap()` 可用于设置和预期成功路径

---

## 性能与并发

### 关键优化

| 优化点 | 说明 |
|--------|------|
| jemalloc | 非 Windows 构建使用，勿轻易移除 |
| 多线程 | 支持 rayon 后台多线程执行 |
| 批处理 | 保持现有批处理行为 |
| 缓冲区复用 | SW 热路径中的 `SwBuffer` |

### 性能测量

```bash
cargo bench  # 使用 Criterion 基准测试
```

---

## 领域知识

- **字母表编码**: `{0:$, 1:A, 2:C, 3:G, 4:T, 5:N}`, SIGMA=6
- **FM 索引格式**: 单一 `.fm` 文件 (bincode 序列化, magic=0x424D_4146_4D5F5253)
- **对齐流程**: FASTQ → 正向+反向互补 → SMEM 种子 → 链构建 → SW 扩展 → 候选去重 → SAM 输出
- **默认参数 (BWA-MEM 风格)**: match=1, mismatch=4, gap_open=6, gap_ext=1, band_width=100, min_seed_len=19

---

## 提交规范

遵循 [Conventional Commits](https://www.conventionalcommits.org/)：

- `feat:` 新功能
- `fix:` Bug 修复
- `docs:` 文档变更
- `refactor:` 代码重构
- `test:` 测试相关
- `perf:` 性能优化
- `ci:` CI 配置变更

---

## 模块速查表

| 功能 | 文件/目录 |
|------|----------|
| **规范文档** | `specs/` |
| 产品功能定义 | `specs/product/` |
| 技术设计 (RFC) | `specs/rfc/` |
| API 接口定义 | `specs/api/` |
| 测试规范 | `specs/testing/` |
| FM 索引核心 | `src/index/fm.rs` |
| 索引构建 | `src/index/builder.rs`, `sa.rs`, `bwt.rs` |
| 种子查找 | `src/align/seed.rs` |
| 链构建 | `src/align/chain.rs` |
| SW 扩展 | `src/align/extend.rs`, `sw.rs` |
| 候选管理 | `src/align/candidate.rs` |
| 比对流水线 | `src/align/pipeline.rs` |
| FASTA/FASTQ | `src/io/fasta.rs`, `fastq.rs` |
| SAM 格式化 | `src/io/sam.rs` |
| DNA 工具 | `src/util/dna.rs` |

---

## 代理工作流程

1. **阅读**目标模块及其测试
2. **修改**最小化正确的代码变更
3. **测试**运行最窄的相关测试
4. **格式化** `cargo fmt --all`
5. **检查**运行目标 clippy/测试，大变更运行完整检查
6. **总结**变更文件、行为变化、未运行的检查

---

## 不要做的事

- 不要使用 unsafe 代码
- 不要在热路径上使用 `String` / `format!` 进行不必要的分配
- 不要修改 `.fm` 文件的 magic number 或破坏索引格式兼容性
- 不要在 library 层 (lib.rs) 使用 `println!`，应使用 `eprintln!` 或返回错误
- 不要删除或弱化现有测试
