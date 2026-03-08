# bwa-rust

[![CI](https://github.com/LessUp/bwa-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/LessUp/bwa-rust/actions/workflows/ci.yml)
[![Docs](https://img.shields.io/badge/docs-GitHub%20Pages-blue)](https://lessup.github.io/bwa-rust/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

**[English](#english) | [中文](#中文)**

---

<a id="english"></a>

## English

A Rust-based sequence aligner inspired by [BWA](https://github.com/lh3/bwa). This project follows the overall architecture and algorithmic ideas of BWA/BWA-MEM, but **does not aim for 100% behavioral compatibility** with the C version of BWA (CLI options, index format, MAPQ details, etc. may differ).

### Implemented Features

#### Index Building (`index` subcommand)
- Read FASTA reference sequences (supports multiple contigs, various line endings, non-standard character filtering)
- Build suffix array (SA) using the doubling algorithm
- Construct BWT from SA
- Build FM-index (with C table, block-wise Occ sampling, sparse SA sampling support)
- Serialize index to `.fm` file (with magic number, version, build metadata)

#### Sequence Alignment (`align` subcommand)
- Load `.fm` index
- Read FASTQ reads
- SMEM seed finding (Super-Maximal Exact Matches) + multi-chain construction and filtering
- Banded affine-gap Smith-Waterman local alignment (supports mismatches, insertions, deletions)
- Forward / reverse complement bidirectional alignment
- Multi-chain candidate deduplication, primary / secondary alignment output (FLAG)
- Improved MAPQ estimation (based on primary-secondary score difference)
- SAM format output (with @HD/@SQ/@PG header, CIGAR, MAPQ, AS/XS/NM tags)
- Unmapped reads marked as unmapped (FLAG=4)
- **Multi-threaded parallelism**: via `--threads` parameter using rayon

#### Supported / Not Supported
- **Supported**: Single-end read alignment
- **Not supported**: Paired-end (PE) alignment (planned for future)

### Quick Start

```bash
# Build
cargo build --release

# Build index
cargo run --release -- index data/toy.fa -o data/toy

# Align reads
cargo run --release -- align -i data/toy.fm data/toy_reads.fq

# Output to file
cargo run --release -- align -i data/toy.fm data/toy_reads.fq -o output.sam

# Multi-threaded alignment
cargo run --release -- align -i data/toy.fm data/toy_reads.fq -t 4

# Custom alignment parameters
cargo run --release -- align -i data/toy.fm data/toy_reads.fq \
    --match 2 --mismatch 1 --gap-open 2 --gap-ext 1 --band-width 16
```

### Index Building & Format Overview

The `index` subcommand takes a FASTA file and performs the following steps:

1. **Read reference sequences**: Parse FASTA records, normalize bases to `{A,C,G,T,N}`.
2. **Encode to numeric alphabet**: `{0:$, 1:A, 2:C, 3:G, 4:T, 5:N}`, contigs separated by `$` (0).
3. **Build suffix array**: Sort all suffixes using the doubling algorithm (O(n log²n)).
4. **Build BWT**: Derive Burrows-Wheeler Transform directly from SA.
5. **Build FM-index**: Compute C table and block-wise Occ sampling table, store SA for position lookup (sparse sampling supported).
6. **Serialize**: Write the entire `FMIndex` structure to a `.fm` file using bincode.

The index file contains a magic number (`BWAFM_RS`) and version (v2) for format compatibility checks. Optional build metadata records the reference filename, command parameters, and timestamps.

### Project Structure

```
.
├── Cargo.toml           # Package config + dependencies + profile
├── .github/             # GitHub CI / Issue templates / PR template
├── src/
│   ├── main.rs          # CLI entry (clap)
│   ├── lib.rs           # Library entry
│   ├── error.rs         # Custom error types (BwaError / BwaResult)
│   ├── io/              # FASTA/FASTQ parsing, SAM output
│   ├── index/           # FM-index (SA, BWT, FM, Builder)
│   ├── align/           # Alignment algorithms (SMEM, Chain, SW, Pipeline)
│   └── util/            # DNA encode/decode/reverse complement
├── tests/               # Integration tests
├── benches/             # Benchmarks
├── examples/            # Example code
├── data/                # Test data (toy.fa / toy_reads.fq)
├── docs/                # Architecture docs, tutorials, full replication plan
├── scripts/             # Dev scripts
├── ROADMAP.md           # Roadmap (v0.1.0 completed)
├── CHANGELOG.md         # Changelog
├── CONTRIBUTING.md      # Contributing guide
└── README.md            # This file
```

### Documentation

| Document | Description |
|----------|-------------|
| [`docs/architecture.md`](docs/architecture.md) | Module architecture, index format, algorithm flow |
| [`docs/tutorial.md`](docs/tutorial.md) | Tutorial: build a BWA-style aligner from scratch |
| [`docs/plan.md`](docs/plan.md) | Full BWA replication roadmap (for future reference) |
| [`ROADMAP.md`](ROADMAP.md) | Development roadmap, future plans, versioning |
| [`CHANGELOG.md`](CHANGELOG.md) | Changelog |

### Test Report

All **133** tests pass (121 unit tests + 11 integration tests + 1 doc test), 0 failures.

```bash
cargo test
# test result: ok. 133 passed; 0 failed; 0 ignored
```

### Installation

#### Build from Source

```bash
git clone https://github.com/LessUp/bwa-rust.git
cd bwa-rust
cargo build --release
```

The compiled binary is at `target/release/bwa-rust`.

#### System Requirements

- **Rust** 1.70 or later
- Supports Linux, macOS, Windows

### Benchmarks

```bash
cargo bench
```

### Examples

```bash
cargo run --example simple_align
```

### Roadmap

The v0.1.0 roadmap is complete. See [ROADMAP.md](ROADMAP.md) for details and future plans (paired-end, BAM output, etc.).

### Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details.

### License

This project is released under the [MIT License](LICENSE).

---

<a id="中文"></a>

## 中文

受 [BWA](https://github.com/lh3/bwa) 启发的 Rust 版序列比对器。本项目在整体结构和算法思想上接近 BWA/BWA-MEM，但**不追求与 C 版 BWA 100% 行为兼容**（命令行选项、索引格式、MAPQ 细节等允许不同）。

### 已实现功能

#### 索引构建（`index` 子命令）
- 读取 FASTA 参考序列（支持多 contig、不同换行符、非标准字符过滤）
- 基于倍增法构建后缀数组（SA）
- 从 SA 构建 BWT
- 构建 FM 索引（含 C 表、分块 Occ 采样、稀疏 SA 采样支持）
- 序列化索引到 `.fm` 文件（含 magic number、版本号、构建元数据）

#### 序列比对（`align` 子命令）
- 加载 `.fm` 索引
- 读取 FASTQ reads
- SMEM 种子查找（超级最大精确匹配）+ 多链构建与过滤
- 带状仿射间隙 Smith-Waterman 局部对齐（支持错配、插入、缺失）
- 正向 / 反向互补双向比对
- 多链候选去重、主/次要比对输出（primary / secondary FLAG）
- 改进的 MAPQ 估算（基于主次候选得分差）
- 输出 SAM 格式（含 @HD/@SQ/@PG header，CIGAR、MAPQ、AS/XS/NM 标签）
- 未比对 reads 标记为 unmapped（FLAG=4）
- **多线程并行**：通过 `--threads` 参数使用 rayon 并行处理

#### 支持/不支持
- **支持**：单端 reads 对齐
- **不支持**：配对端（PE）对齐（未来可扩展）

### 快速开始

```bash
# 构建
cargo build --release

# 构建索引
cargo run --release -- index data/toy.fa -o data/toy

# 对齐 reads
cargo run --release -- align -i data/toy.fm data/toy_reads.fq

# 输出到文件
cargo run --release -- align -i data/toy.fm data/toy_reads.fq -o output.sam

# 多线程对齐
cargo run --release -- align -i data/toy.fm data/toy_reads.fq -t 4

# 自定义比对参数
cargo run --release -- align -i data/toy.fm data/toy_reads.fq \
    --match 2 --mismatch 1 --gap-open 2 --gap-ext 1 --band-width 16
```

### 索引构建与索引格式简介

`index` 子命令接受一个 FASTA 文件，执行以下步骤：

1. **读取参考序列**：逐条解析 FASTA 记录，将碱基归一化为 `{A,C,G,T,N}`。
2. **编码为数值字母表**：`{0:$, 1:A, 2:C, 3:G, 4:T, 5:N}`，contig 之间用 `$`（0）分隔。
3. **构建后缀数组**：使用倍增法（O(n log²n)）排序所有后缀。
4. **构建 BWT**：从 SA 直接推导 Burrows-Wheeler 变换。
5. **构建 FM 索引**：计算 C 表和分块 Occ 采样表，保存 SA 用于位置查询（支持稀疏采样）。
6. **序列化**：使用 bincode 将整个 `FMIndex` 结构写入 `.fm` 文件。

索引文件包含 magic number（`BWAFM_RS`）和版本号（v2），用于格式兼容性检查。可选的构建元数据记录参考文件名、命令参数和时间戳。

### 项目结构

```
.
├── Cargo.toml           # 包配置 + 依赖 + profile
├── .github/             # GitHub CI / Issue 模板 / PR 模板
├── src/
│   ├── main.rs          # CLI 入口（clap）
│   ├── lib.rs           # Library 入口
│   ├── error.rs         # 自定义错误类型（BwaError / BwaResult）
│   ├── io/              # FASTA/FASTQ 解析、SAM 输出
│   ├── index/           # FM 索引（SA、BWT、FM、Builder）
│   ├── align/           # 对齐算法（SMEM、Chain、SW、Pipeline）
│   └── util/            # DNA 编码/解码/反向互补
├── tests/               # 集成测试
├── benches/             # 基准测试
├── examples/            # 示例代码
├── data/                # 测试数据（toy.fa / toy_reads.fq）
├── docs/                # 架构文档、教程、全量复刻计划
├── scripts/             # 开发脚本
├── ROADMAP.md           # 开发路线图（v0.1.0 已完成）
├── CHANGELOG.md         # 版本变更日志
├── CONTRIBUTING.md      # 贡献指南
└── README.md            # 本文件
```

### 文档

| 文档 | 说明 |
|------|------|
| [`docs/architecture.md`](docs/architecture.md) | 模块架构、索引格式、算法流程 |
| [`docs/tutorial.md`](docs/tutorial.md) | 教程：从 0 实现 BWA 风格对齐器 |
| [`docs/plan.md`](docs/plan.md) | BWA 全量复刻远景规划（供未来扩展参考） |
| [`ROADMAP.md`](ROADMAP.md) | 开发路线图、未来展望、版本策略 |
| [`CHANGELOG.md`](CHANGELOG.md) | 版本变更日志 |

### 测试用例报告

全部 **133** 个测试通过（121 单元测试 + 11 集成测试 + 1 文档测试），0 失败。

```bash
cargo test
# test result: ok. 133 passed; 0 failed; 0 ignored
```

### 安装

#### 从源码构建

```bash
git clone https://github.com/LessUp/bwa-rust.git
cd bwa-rust
cargo build --release
```

编译后的二进制文件位于 `target/release/bwa-rust`。

#### 系统要求

- **Rust** 1.70 或更高版本
- 支持 Linux、macOS、Windows

### 基准测试

```bash
cargo bench
```

### 示例

```bash
cargo run --example simple_align
```

### 规划

v0.1.0 路线图已全部完成，详见 [ROADMAP.md](ROADMAP.md)。未来展望（配对端、BAM 输出等）见路线图末尾。

### 贡献

欢迎贡献！请阅读 [CONTRIBUTING.md](CONTRIBUTING.md) 了解详情。

### 许可证

本项目采用 [MIT 许可证](LICENSE) 发布。
