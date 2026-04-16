# bwa-rust

[![CI](https://github.com/LessUp/bwa-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/LessUp/bwa-rust/actions/workflows/ci.yml)
[![Version](https://img.shields.io/badge/version-0.2.0-blue)](https://github.com/LessUp/bwa-rust/releases)
[![Docs](https://img.shields.io/badge/docs-GitHub%20Pages-blue)](https://lessup.github.io/bwa-rust/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

[English](README.md) | 简体中文 | [📖 文档](docs/)

---

受 [BWA](https://github.com/lh3/bwa) 启发的 Rust 版序列比对器。

> **注意**：本项目在整体结构和算法思想上借鉴 BWA/BWA-MEM，但**不追求与 C 版 BWA 100% 兼容**（CLI 选项、索引格式、MAPQ 计算等可能不同）。

## ✨ 特性

| 特性 | 说明 |
|------|------|
| **FM 索引** | 后缀数组 + BWT + 稀疏 SA 采样；单一 `.fm` 文件 |
| **SMEM 种子** | 超级最大精确匹配，支持左向扩展 |
| **种子链构建** | 基于 DP 的链评分，贪心提取多链 |
| **Smith-Waterman** | 带状局部对齐，仿射间隙模型，CIGAR 输出 |
| **SAM 输出** | 标准格式，支持 @HD/@SQ/@PG、CIGAR、MAPQ、AS/XS/NM |
| **多线程** | 基于 rayon 的 reads 级并行 |
| **内存安全** | 零 `unsafe` 代码；非 Windows 使用 jemalloc |
| **可配置** | 支持 `max_occ`、`max_chains`、`max_alignments` 限制 |

## 📦 安装

### 从源码编译

```bash
git clone https://github.com/LessUp/bwa-rust.git
cd bwa-rust
cargo build --release
```

二进制文件：`target/release/bwa-rust`

### 系统要求

- Rust 1.70+
- 支持 Linux、macOS、Windows

## 🚀 快速开始

### 构建索引

```bash
bwa-rust index reference.fa -o ref
# 生成：ref.fm
```

### 比对 Reads

```bash
# 一步完成（BWA-MEM 风格）
bwa-rust mem reference.fa reads.fq -o output.sam

# 多线程
bwa-rust mem ref.fa reads.fq -t 4 -o output.sam
```

## 📚 文档

| 资源 | 说明 |
|------|------|
| [快速入门](docs/tutorial/getting-started.zh-CN.md) | 安装和基本使用指南 |
| [架构设计](docs/architecture/) | 模块设计和实现细节 |
| [算法教程](docs/tutorial/algorithms.zh-CN.md) | 核心算法详解 |
| [API 文档](docs/api/) | 库使用文档 |
| [更新日志](CHANGELOG.md) | 版本历史和发布说明 |
| [在线文档](https://lessup.github.io/bwa-rust/) | VitePress 文档站点 |

## 📊 与 BWA 对比

| 特性 | BWA (C) | bwa-rust |
|------|---------|----------|
| 索引格式 | 多文件 (`.bwt`, `.sa`, `.pac`) | 单一 `.fm` |
| SA 构建 | DC3/IS O(n) | 倍增法 O(n log²n) |
| MEM 查找 | 双向 BWT | 单向 |
| 并行 | pthread | rayon |
| 安全性 | unsafe C 代码 | 零 unsafe |
| 配对端 | ✅ 支持 | 🚧 计划中 (v0.3.0) |
| BAM 输出 | ✅ 支持 | 🚧 计划中 (v0.4.0) |

## 🧪 测试

```bash
cargo test                    # 运行所有测试
cargo test -- --nocapture     # 显示输出
cargo bench                   # 运行基准测试
```

## 🔧 作为库使用

```rust
use bwa_rust::index::{sa, bwt, fm};
use bwa_rust::util::dna;

let reference = b"ACGTACGT";
let norm = dna::normalize_seq(reference);
let mut text: Vec<u8> = norm.iter().map(|&b| dna::to_alphabet(b)).collect();
text.push(0);

let sa_arr = sa::build_sa(&text);
let bwt_arr = bwt::build_bwt(&text, &sa_arr);
let contigs = vec![fm::Contig { name: "ref".into(), len: 8, offset: 0 }];
let fm_idx = fm::FMIndex::build(text, bwt_arr, sa_arr, contigs, 6, 4);
```

更多示例请参见 [库使用指南](docs/api/library-usage.zh-CN.md)。

## 🤝 贡献

欢迎贡献！请参阅 [CONTRIBUTING.md](CONTRIBUTING.md)。

## 📄 许可证

[MIT 许可证](LICENSE)

## 🙏 致谢

本项目受 Heng Li 的 [BWA](https://github.com/lh3/bwa) 启发。
