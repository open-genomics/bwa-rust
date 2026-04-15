# bwa-rust

[![CI](https://github.com/LessUp/bwa-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/LessUp/bwa-rust/actions/workflows/ci.yml)
[![Docs](https://img.shields.io/badge/docs-GitHub%20Pages-blue)](https://lessup.github.io/bwa-rust/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

[English](README.md) | 简体中文

受 [BWA](https://github.com/lh3/bwa) 启发的 Rust 版序列比对器。本项目在整体结构和算法思想上接近 BWA/BWA-MEM，但**不追求与 C 版 BWA 100% 行为兼容**（命令行选项、索引格式、MAPQ 细节等允许不同）。

## 特性

- **FM 索引构建** — 后缀数组、BWT、Occ 采样表，支持可配置的 SA 稀疏采样
- **SMEM 种子查找** — 超级最大精确匹配，增量式左扩展
- **种子链构建** — 基于 DP 的链评分，贪心剥离提取多链
- **Smith-Waterman** — 带状仿射间隙局部对齐，生成 CIGAR
- **完整流水线** — 单一二进制文件完成索引构建 + reads 比对
- **多线程并行** — 基于 rayon 的 reads 级并行处理
- **内存安全** — 零 unsafe 代码，jemalloc 分配器提升多线程性能
- **可配置限制** — `max_occ`、`max_chains_per_contig`、`max_alignments_per_read` 防止内存爆炸

## 快速开始

### 安装

```bash
git clone https://github.com/LessUp/bwa-rust.git
cd bwa-rust
cargo build --release
```

编译后的二进制文件位于 `target/release/bwa-rust`。

### 系统要求

- Rust 1.70+
- 支持 Linux、macOS、Windows

### 构建索引

```bash
# 从 FASTA 参考序列构建 FM 索引
bwa-rust index data/toy.fa -o data/toy
# 输出：data/toy.fm
```

### 比对 Reads

```bash
# 基本比对
bwa-rust align -i data/toy.fm data/toy_reads.fq

# 输出到文件
bwa-rust align -i data/toy.fm data/toy_reads.fq -o output.sam

# 多线程（4 线程）
bwa-rust align -i data/toy.fm data/toy_reads.fq -t 4

# 自定义打分参数
bwa-rust align -i data/toy.fm data/toy_reads.fq \
    --match 2 --mismatch 1 --gap-open 2 --gap-ext 1 --band-width 16
```

### 一步比对（BWA-MEM 风格）

```bash
# 构建索引并比对
bwa-rust mem data/toy.fa data/toy_reads.fq -t 4 -o output.sam
```

## CLI 参考

### `index` — 构建 FM 索引

```
bwa-rust index <reference.fa> -o <prefix>
```

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `reference` | （必填） | FASTA 参考文件 |
| `-o, --output` | `ref` | 输出前缀，生成 `.fm` 索引 |

### `align` — 使用已有索引比对

```
bwa-rust align -i <index.fm> <reads.fq> [options]
```

| 选项 | 默认值 | 说明 |
|------|--------|------|
| `-i, --index` | （必填） | `.fm` 索引路径 |
| `reads` | （必填） | FASTQ reads 文件 |
| `-o, --out` | stdout | 输出 SAM 路径 |
| `--match` | 2 | 匹配得分 |
| `--mismatch` | 1 | 错配罚分 |
| `--gap-open` | 2 | Gap 开启罚分 |
| `--gap-ext` | 1 | Gap 扩展罚分 |
| `--clip-penalty` | 1 | 软剪切惩罚（候选排序用） |
| `--band-width` | 16 | 带状 SW 带宽 |
| `--score-threshold` | 20 | 最低输出得分 |
| `-t, --threads` | 1 | 线程数 |

### `mem` — 一步比对

```
bwa-rust mem <reference.fa> <reads.fq> [options]
```

使用 BWA-MEM 默认打分：match=1, mismatch=4, gap-open=6, gap-ext=1。

| 选项 | 默认值 | 说明 |
|------|--------|------|
| `-o, --out` | stdout | 输出 SAM 路径 |
| `-A, --match` | 1 | 匹配得分 |
| `-B, --mismatch` | 4 | 错配罚分 |
| `-O, --gap-open` | 6 | Gap 开启罚分 |
| `-E, --gap-ext` | 1 | Gap 扩展罚分 |
| `--clip-penalty` | 1 | 软剪切惩罚 |
| `-w, --band-width` | 100 | 带宽 |
| `-T, --score-threshold` | 10 | 最低得分 |
| `-t, --threads` | 1 | 线程数 |

## 项目结构

```
├── src/
│   ├── main.rs          # CLI 入口（clap）
│   ├── lib.rs           # Library 入口 + 测试工具
│   ├── error.rs         # BwaError 枚举 + BwaResult<T>
│   ├── io/              # FASTA/FASTQ 解析、SAM 输出
│   ├── index/           # FM 索引（SA、BWT、FM、Builder）
│   ├── align/           # 比对算法（SMEM、Chain、SW、Pipeline）
│   └── util/            # DNA 编码/解码/反向互补
├── tests/               # 集成测试
├── benches/             # Criterion 基准测试
├── examples/            # 示例代码
├── data/                # 测试数据
└── docs/                # 架构、教程
```

## 测试

```bash
# 运行所有测试
cargo test

# 运行单个测试
cargo test <test_name>

# 显示输出
cargo test -- --nocapture

# test result: ok. 167 passed; 0 failed; 0 ignored
```

## 基准测试

```bash
cargo bench
```

## 作为库使用

```rust
use bwa_rust::index::{sa, bwt, fm};
use bwa_rust::util::dna;

// 构建 FM 索引
let reference = b"ACGTACGT";
let norm = dna::normalize_seq(reference);
let mut text: Vec<u8> = norm.iter().map(|&b| dna::to_alphabet(b)).collect();
text.push(0);

let sa_arr = sa::build_sa(&text);
let bwt_arr = bwt::build_bwt(&text, &sa_arr);
let contigs = vec![fm::Contig { name: "ref".into(), len: 8, offset: 0 }];
let fm_idx = fm::FMIndex::build(text, bwt_arr, sa_arr, contigs, 6, 4);

// 搜索模式
let pattern: Vec<u8> = b"CGT".iter().map(|&b| dna::to_alphabet(b)).collect();
if let Some((l, r)) = fm_idx.backward_search(&pattern) {
    println!("找到 {} 次出现", r - l);
}
```

运行示例：

```bash
cargo run --example simple_align
```

## 文档

| 文档 | 说明 |
|------|------|
| [docs/architecture.md](docs/architecture.md) | 模块架构、索引格式、算法流程 |
| [docs/tutorial.md](docs/tutorial.md) | 教程：从 0 实现 BWA 风格对齐器 |
| [ROADMAP.md](ROADMAP.md) | 开发路线图、版本策略 |
| [CHANGELOG.md](CHANGELOG.md) | 版本变更日志 |
| [GitHub Pages](https://lessup.github.io/bwa-rust/) | 在线文档 |

## 与 BWA 对比

| 特性 | BWA (C) | bwa-rust |
|------|---------|----------|
| 索引格式 | 多文件（`.bwt`、`.sa`、`.pac`） | 单一 `.fm` 文件 |
| SA 构造 | DC3/IS O(n) | 倍增法 O(n log²n) |
| MEM 查找 | 双向 BWT | 单向扩展 |
| 多线程 | pthread | rayon |
| 配对端 | ✓ | 计划中（v0.2.0） |
| BAM 输出 | ✓ | 计划中（v0.4.0） |

## 贡献

欢迎贡献！请阅读 [CONTRIBUTING.md](CONTRIBUTING.md)。

## 许可证

[MIT 许可证](LICENSE)
