# bwa-rust

[![CI](https://github.com/LessUp/bwa-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/LessUp/bwa-rust/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/LessUp/bwa-rust?color=blue)](https://github.com/LessUp/bwa-rust/releases)
[![Docs](https://img.shields.io/badge/docs-GitHub%20Pages-blue)](https://lessup.github.io/bwa-rust/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](rust-toolchain.toml)

用 Rust 实现的内存安全 BWA-MEM 风格单端 DNA 短读比对器。

[English](README.md) | 简体中文 | [项目站点](https://lessup.github.io/bwa-rust/)

## 已交付能力

| 领域 | 状态 | 说明 |
|------|------|------|
| FASTA 参考序列 | 已交付 | 支持多 contig，并归一化到项目字母表。 |
| FASTQ 单端 reads | 已交付 | 单端 reads 是当前稳定范围。 |
| FM-index | 已交付 | 后缀数组 + BWT + Occ 采样，序列化为单一 `.fm` 文件。 |
| BWA-MEM 风格比对 | 已交付 | SMEM 种子、种子链、带状 Smith-Waterman 延伸。 |
| SAM 输出 | 已交付 | header、CIGAR、MAPQ、AS/XS/NM、可用时输出 MD:Z 与 SA:Z。 |
| 并行比对 | 已交付 | 基于 rayon 的 read 级并行。 |
| 配对端比对 | 计划中 | 已有 reader/insert-size 基础，但 CLI 流水线仍是单端。 |
| BAM/CRAM 输出 | 计划中 | 当前仅交付 SAM。 |

## 项目价值

- 全仓库禁止 `unsafe`，由 lint 强制执行。
- 单文件 `.fm` 索引比 BWA 多文件索引更易移动和管理。
- 清晰呈现 seed-chain-extend 比对流水线，适合学习和实验。
- 同时提供 CLI 与 Rust library，方便嵌入 Rust 生物信息学流程。

如果需要生产级配对端流程、精确 BWA 行为兼容或成熟 BAM/CRAM 工作流，请优先使用原版 BWA。

## 安装

从 [GitHub Releases](https://github.com/LessUp/bwa-rust/releases) 下载对应平台产物：

| 平台 | 产物 |
|------|------|
| Linux 静态链接 | `bwa-rust-linux-amd64-static.tar.gz` |
| Linux glibc | `bwa-rust-linux-amd64.tar.gz` |
| macOS Intel | `bwa-rust-macos-amd64.tar.gz` |
| macOS Apple Silicon | `bwa-rust-macos-arm64.tar.gz` |
| Windows | `bwa-rust-windows-amd64.zip` |

从源码构建：

```bash
git clone https://github.com/LessUp/bwa-rust.git
cd bwa-rust
cargo build --release
```

## 快速开始

```bash
# 构建单文件 FM-index
bwa-rust index reference.fa -o ref

# 使用预构建索引比对单端 reads
bwa-rust align -i ref.fm reads.fq -o output.sam

# 内存中构建索引并一步比对
bwa-rust mem reference.fa reads.fq -t 4 -o output.sam
```

默认比对参数以 `src/align/mod.rs` 的 `AlignOpt::default()` 为准：

| 参数 | 默认值 |
|------|--------|
| match / mismatch | `2 / 1` |
| gap open / extend | `2 / 1` |
| band width | `16` |
| score threshold | `20` |
| min seed length | `19` |
| max seed occurrences | `500` |
| max chains / alignments | `5 / 5` |
| z-drop | `100` |

## 架构

```text
FASTA/FASTQ -> FM-index -> SMEM seeds -> chains -> Smith-Waterman -> SAM
```

关键模块：

- `src/index/`：后缀数组、BWT、FM-index 序列化。
- `src/align/`：种子、链、延伸、MAPQ、补充比对分类、流水线。
- `src/io/`：FASTA、FASTQ、SAM 解析与格式化。
- `src/util/dna.rs`：DNA 归一化、字母表映射、反向互补。

## 开发

OpenSpec 是行为和治理规范的唯一事实来源：见 `openspec/specs/`。

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
npm run docs:build
```

项目级 AI 工作流见 `AGENTS.md` 和 `docs/development/ai-workflow.md`。

## 文档

- 公共站点：<https://lessup.github.io/bwa-rust/>
- API 文档：<https://docs.rs/bwa-rust>
- 规范：`openspec/specs/`
- 路线图：`ROADMAP.md`
- 变更日志：`CHANGELOG.md`

## 许可证

MIT。本项目受 Heng Li 的 [BWA](https://github.com/lh3/bwa) 启发。
