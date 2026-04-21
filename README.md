# bwa-rust

<div align="center">

[![CI](https://github.com/LessUp/bwa-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/LessUp/bwa-rust/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/LessUp/bwa-rust?color=blue)](https://github.com/LessUp/bwa-rust/releases)
[![Docs](https://img.shields.io/badge/docs-GitHub%20Pages-blue)](https://lessup.github.io/bwa-rust/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Coverage](https://codecov.io/gh/LessUp/bwa-rust/branch/main/graph/badge.svg)](https://codecov.io/gh/LessUp/bwa-rust)

**A high-performance BWA-MEM style DNA sequence aligner in Rust**

*Zero unsafe code • Multi-threaded • Single-file index*

English | [简体中文](README.zh-CN.md) | [📖 Documentation](docs/)

</div>

---

## Overview

bwa-rust is a BWA-MEM style short-read aligner implemented in Rust from scratch. It follows the architecture and algorithms of BWA/BWA-MEM, but does not aim for 100% behavioral compatibility with the C version (CLI options, index format, MAPQ details may differ).

### Key Highlights

| Feature | Description |
|---------|-------------|
| 🔒 **Memory Safe** | Zero `unsafe` code, verified by `forbid(unsafe_code)` lint |
| 🚀 **Fast** | Read-level parallelism via rayon, jemalloc allocator |
| 📦 **Simple** | Single `.fm` index file vs. multiple files in original BWA |
| 🎯 **Standard** | Full SAM output with CIGAR, MAPQ, AS/XS/NM tags |
| 🔧 **Configurable** | Memory protection limits for repetitive sequences |

## Features

| Component | Implementation |
|-----------|---------------|
| **FM-Index** | Suffix array + BWT + sparse SA sampling |
| **SMEM Seeding** | Super-Maximal Exact Match finding with left-extension |
| **Seed Chaining** | DP-based chain scoring with greedy multi-chain extraction |
| **Smith-Waterman** | Banded local alignment with affine gap penalty |
| **SAM Output** | Standard format with @HD/@SQ/@PG headers |

## Installation

### Download Pre-built Binary

Download from [GitHub Releases](https://github.com/LessUp/bwa-rust/releases) for your platform:

| Platform | File |
|----------|------|
| Linux (static, recommended) | `bwa-rust-linux-amd64-static.tar.gz` |
| Linux (dynamic) | `bwa-rust-linux-amd64.tar.gz` |
| macOS (Intel) | `bwa-rust-macos-amd64.tar.gz` |
| macOS (Apple Silicon) | `bwa-rust-macos-arm64.tar.gz` |
| Windows | `bwa-rust-windows-amd64.zip` |

### Build from Source

```bash
git clone https://github.com/LessUp/bwa-rust.git
cd bwa-rust
cargo build --release
```

Binary: `target/release/bwa-rust`

**Requirements:** Rust 1.70+ on Linux, macOS, or Windows.

## Quick Start

### Build Index

```bash
bwa-rust index reference.fa -o ref
# Creates: ref.fm
```

### Align Reads

```bash
# One-step alignment (BWA-MEM style)
bwa-rust mem reference.fa reads.fq -o output.sam

# Multi-threaded
bwa-rust mem ref.fa reads.fq -t 4 -o output.sam

# With pre-built index
bwa-rust align -i ref.fm reads.fq -o output.sam
```

## Comparison with BWA

| Feature | BWA (C) | bwa-rust |
|---------|---------|----------|
| Index format | Multiple files (`.bwt`, `.sa`, `.pac`) | Single `.fm` |
| SA construction | DC3/IS O(n) | Doubling O(n log²n) |
| MEM finding | Bidirectional BWT | Unidirectional |
| Parallelism | pthread | rayon |
| Memory safety | unsafe C code | Zero unsafe |
| Paired-end | ✅ Supported | 🚧 Planned (v0.3.0) |
| BAM output | ✅ Supported | 🚧 Planned (v0.4.0) |

## Documentation

| Resource | Description |
|----------|-------------|
| [Specifications](specs/) | Single Source of Truth (SDD workflow) |
| [Getting Started](docs/tutorial/getting-started.md) | Installation and basic usage guide |
| [Architecture](docs/architecture/) | Module design and implementation details |
| [Algorithms](docs/tutorial/algorithms.md) | Core algorithm tutorial |
| [API Reference](docs/api/) | Library usage documentation |
| [Changelog](CHANGELOG.md) | Version history and release notes |
| [Online Docs](https://lessup.github.io/bwa-rust/) | VitePress documentation site |

## Library Usage

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

See [Library Usage Guide](docs/api/library-usage.md) for more examples.

## Development

```bash
# Run tests
cargo test

# Run benchmarks
cargo bench

# Check code quality
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

[MIT License](LICENSE)

## Acknowledgments

Inspired by [BWA](https://github.com/lh3/bwa) by Heng Li.
