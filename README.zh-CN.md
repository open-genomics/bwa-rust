# bwa-rust

<div align="center">

[![CI](https://github.com/LessUp/bwa-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/LessUp/bwa-rust/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/LessUp/bwa-rust?color=blue)](https://github.com/LessUp/bwa-rust/releases)
[![Docs](https://img.shields.io/badge/docs-GitHub%20Pages-blue)](https://lessup.github.io/bwa-rust/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Coverage](https://codecov.io/gh/LessUp/bwa-rust/branch/master/graph/badge.svg)](https://codecov.io/gh/LessUp/bwa-rust)
[![Crates.io](https://img.shields.io/crates/v/bwa-rust)](https://crates.io/crates/bwa-rust)

**高性能 BWA-MEM 风格 DNA 序列比对器**

*零 unsafe 代码 • 多线程 • 单文件索引*

[English](README.md) | 简体中文 | [📖 文档](https://lessup.github.io/bwa-rust/)

</div>

---

## 概述

bwa-rust 是一个用 Rust 从零实现的 BWA-MEM 风格短序列比对器，提供核心 BWA-MEM 算法的清晰、内存安全实现，采用简化索引格式和现代并行机制。

### 核心特性

| 特性 | 说明 |
|------|------|
| 🔒 **内存安全** | 零 `unsafe` 代码，通过 `forbid(unsafe_code)` lint 验证 |
| 🚀 **多线程** | 基于 rayon 的 reads 级并行，jemalloc 分配器 |
| 📦 **简洁索引** | 单一 `.fm` 文件，对比 BWA 的多文件索引 |
| 🎯 **标准输出** | 完整 SAM 格式，支持 CIGAR、MAPQ、AS/XS/NM 标签 |
| 🔧 **内存保护** | 可配置限制以处理重复序列 |
| 📖 **可读代码** | 教育级实现，便于学习比对算法 |

### 适用场景

- **Rust 集成**：在 Rust 项目中构建生物信息学流程库
- **内存安全**：对安全性有严格要求的参考基因组处理
- **学习研究**：清晰架构，便于理解 BWA-MEM 算法
- **单端比对**：稳定的 SE reads 比对基线

> ⚠️ **当前范围**：本项目为**单端比对器**。配对端支持已规划但尚未实现。对于生产级 PE 比对，建议使用原版 [BWA](https://github.com/lh3/bwa)。

---

## 安装

### 快速安装（Linux/macOS）

```bash
# 下载最新版本
curl -sL https://github.com/LessUp/bwa-rust/releases/latest/download/bwa-rust-linux-amd64-static.tar.gz | tar xz

# 移动到 PATH
sudo mv bwa-rust /usr/local/bin/

# 验证安装
bwa-rust --version
```

### 下载预编译二进制

从 [GitHub Releases](https://github.com/LessUp/bwa-rust/releases) 下载对应平台：

| 平台 | 文件 | 大小 |
|------|------|------|
| Linux（静态链接，推荐） | `bwa-rust-linux-amd64-static.tar.gz` | ~3 MB |
| Linux（动态链接） | `bwa-rust-linux-amd64.tar.gz` | ~2 MB |
| macOS (Intel) | `bwa-rust-macos-amd64.tar.gz` | ~2 MB |
| macOS (Apple Silicon) | `bwa-rust-macos-arm64.tar.gz` | ~2 MB |
| Windows | `bwa-rust-windows-amd64.zip` | ~3 MB |

### 从源码编译

**系统要求：** Rust 1.70+，支持 Linux、macOS、Windows。

```bash
git clone https://github.com/LessUp/bwa-rust.git
cd bwa-rust
cargo build --release
```

二进制文件：`target/release/bwa-rust`

### 作为库使用

添加到 `Cargo.toml`：

```toml
[dependencies]
bwa-rust = "0.2"
```

---

## 快速开始

### 构建索引

```bash
bwa-rust index reference.fa -o ref
# 生成：ref.fm
```

### 比对 Reads

```bash
# 一步完成（BWA-MEM 风格）
bwa-rust mem reference.fa reads.fq -o output.sam

# 多线程（4 线程）
bwa-rust mem ref.fa reads.fq -t 4 -o output.sam

# 使用预构建索引
bwa-rust align -i ref.fm reads.fq -o output.sam
```

### 输出示例

```
@HD	VN:1.6	SO:unsorted
@SQ	SN:chr1	LN:248956422
@PG	ID:bwa-rust	PN:bwa-rust	VN:0.2.0
read_001	0	chr1	10001	60	100M	*	0	0	ACGT...	!!!!...	AS:i:100	XS:i:0	NM:i:0
```

---

## 性能

基于 E. coli 参考基因组（约 4.6Mbp）和 100bp reads 的小规模测试显示，bwa-rust 单端比对达到 BWA-MEM 约 70% 的吞吐量：

| 指标 | BWA-MEM | bwa-rust | 说明 |
|------|---------|----------|------|
| 索引构建 | ~2s | ~3s | O(n log²n) 倍增法 vs O(n) |
| 索引大小 | ~5MB（多文件） | ~5MB（单 `.fm`） | 总大小相当 |
| 比对速度（单线程） | ~10K reads/s | ~7K reads/s | 仅单端 |
| 比对速度（8线程） | ~35K reads/s | ~25K reads/s | rayon 并行 |
| 内存占用 | ~150MB | ~150MB | 小基因组相当 |

> **注意**：以上为小规模测试的示意数据。针对人类基因组规模的全面基准测试计划中。详见 `cargo bench` 获取可复现的微基准测试。

---

## 功能特性

### 核心组件

| 组件 | 实现 |
|------|------|
| **FM 索引** | 后缀数组 + BWT + 稀疏 SA 采样 |
| **SMEM 种子** | 超级最大精确匹配查找，支持左向扩展 |
| **种子链构建** | 基于 DP 的链评分，贪心多链提取 |
| **Smith-Waterman** | 带状局部对齐，仿射间隙模型 |
| **SAM 输出** | 标准格式，支持 @HD/@SQ/@PG 头部 |

## 与 BWA 对比

| 特性 | BWA (C) | bwa-rust |
|------|---------|----------|
| 索引格式 | 多文件 (`.bwt`, `.sa`, `.pac`) | 单一 `.fm` |
| SA 构建 | DC3/IS O(n) | 倍增法 O(n log²n) |
| MEM 查找 | 双向 BWT | Backward search |
| 并行 | pthread | rayon |
| 安全性 | 手动内存管理 | 零 `unsafe`（编译器验证） |
| 配对端 | ✅ 支持 | 🚧 计划中 |
| BAM 输出 | ✅ 支持 | 🚧 计划中 |

**兼容性说明**：bwa-rust 遵循 BWA-MEM 算法思想，但不追求 100% 行为兼容。索引格式、MAPQ 计算、部分启发式规则有意设计不同。

---

## 文档

| 资源 | 链接 |
|------|------|
| **文档站点** | [lessup.github.io/bwa-rust](https://lessup.github.io/bwa-rust/) |
| 架构指南 | [架构概览](https://lessup.github.io/bwa-rust/architecture/) |
| 安装与使用 | [使用指南](https://lessup.github.io/bwa-rust/guide/) |
| 性能数据 | [基准测试](https://lessup.github.io/bwa-rust/benchmarks) |
| 常见问题 | [FAQ](https://lessup.github.io/bwa-rust/faq) |
| 规范文档 | [openspec/specs/](openspec/specs/)（OpenSpec 工作流） |
| 更新日志 | [CHANGELOG.md](CHANGELOG.md) |

---

## 作为库使用

### 基础示例

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

// 使用 FM 索引进行精确匹配
let pattern: Vec<u8> = b"ACGT".iter().map(|&b| dna::to_alphabet(b)).collect();
if let Some((l, r)) = fm_idx.backward_search(&pattern) {
    println!("匹配范围: [{}, {}]", l, r);
}
```

### 比对示例

```rust
use bwa_rust::index::fm;
use bwa_rust::align::{find_smem_seeds, AlignOpt};
use bwa_rust::util::dna;

// 加载 FM 索引
let fm_idx = fm::FMIndex::load_from_file("reference.fm")?;

// 配置比对选项
let opt = AlignOpt {
    min_seed_len: 19,
    max_occ: 1000,
    ..AlignOpt::default()
};

// 查找 SMEM 种子
let read = b"ACGTACGTACGTACGTACGT";
let norm = dna::normalize_seq(read);
let alpha: Vec<u8> = norm.iter().map(|&b| dna::to_alphabet(b)).collect();

let seeds = find_smem_seeds(&fm_idx, &alpha, opt.min_seed_len);
for seed in seeds {
    println!("种子: 长度={}, contig={}, query=[{}..{}], ref=[{}..{}]",
        seed.qe - seed.qb, seed.contig, seed.qb, seed.qe, seed.rb, seed.re);
}
```

更多示例请参见 [库使用指南](docs/api/library-usage.zh-CN.md)。

---

## 开发

```bash
# 运行测试
cargo test

# 运行指定测试
cargo test test_name -- --nocapture

# 运行基准测试
cargo bench

# 代码质量检查
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings

# 构建文档
cargo doc --open
```

---

## 常见问题

**Q: bwa-rust 与 BWA 索引兼容吗？**  
A: 不兼容。bwa-rust 使用不同的单文件 `.fm` 索引格式，需要使用 `bwa-rust index` 重新构建索引。

**Q: 我可以在生产流程中使用 bwa-rust 吗？**  
A: bwa-rust 适用于单端比对和 Rust 库集成。对于生产级双端工作流，建议在 bwa-rust 的 PE 支持发布前使用原版 BWA。

**Q: 支持哪些文件格式？**  
A: **输入**：FASTA（参考基因组）、FASTQ（单端 reads）。**输出**：SAM。BAM 输出计划在未来版本中支持。

**Q: 如何报告问题或请求功能？**  
A: 请使用 [GitHub Issues](https://github.com/LessUp/bwa-rust/issues)，并先检查是否已有相关问题。

---

## 贡献

欢迎贡献！请参阅 [CONTRIBUTING.md](CONTRIBUTING.md)。

## 许可证

[MIT 许可证](LICENSE)

## 致谢

本项目受 Heng Li 的 [BWA](https://github.com/lh3/bwa) 启发。

---

<div align="center">

[![Star History](https://api.star-history.com/svg?repos=LessUp/bwa-rust&type=Date)](https://star-history.com/#LessUp/bwa-rust&Date)

</div>
