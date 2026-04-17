# bwa-rust

[![CI](https://github.com/LessUp/bwa-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/LessUp/bwa-rust/actions/workflows/ci.yml)
[![Version](https://img.shields.io/badge/version-0.2.0-blue)](https://github.com/LessUp/bwa-rust/releases)
[![Docs](https://img.shields.io/badge/docs-GitHub%20Pages-blue)](https://lessup.github.io/bwa-rust/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

English | [简体中文](README.zh-CN.md) | [📖 Documentation](docs/)

---

A BWA-MEM style short-read aligner implemented in Rust from scratch.

> **Note**: This project follows the architecture and algorithms of BWA/BWA-MEM, but does not aim for 100% behavioral compatibility with the C version (CLI options, index format, MAPQ details may differ).

## ✨ Features

| Feature | Description |
|---------|-------------|
| **FM-Index** | Suffix array + BWT + sparse SA sampling; single `.fm` file |
| **SMEM Seeding** | Super-Maximal Exact Match finding with left-extension |
| **Seed Chaining** | DP-based chain scoring with greedy multi-chain extraction |
| **Smith-Waterman** | Banded local alignment with affine gap and CIGAR |
| **SAM Output** | Standard format with @HD/@SQ/@PG, CIGAR, MAPQ, AS/XS/NM |
| **Multi-threaded** | Read-level parallelism via rayon |
| **Memory Safe** | Zero `unsafe` code; jemalloc on non-Windows |
| **Configurable** | `max_occ`, `max_chains`, `max_alignments` limits |

## 📦 Installation

### From Source

```bash
git clone https://github.com/LessUp/bwa-rust.git
cd bwa-rust
cargo build --release
```

Binary: `target/release/bwa-rust`

### Requirements

- Rust 1.70+
- Linux, macOS, Windows

## 🚀 Quick Start

### Build Index

```bash
bwa-rust index reference.fa -o ref
# Creates: ref.fm
```

### Align Reads

```bash
# One-step (BWA-MEM style)
bwa-rust mem reference.fa reads.fq -o output.sam

# Multi-threaded
bwa-rust mem ref.fa reads.fq -t 4 -o output.sam
```

## 📚 Documentation

| Resource | Description |
|----------|-------------|
| [Specifications](specs/) | **Single Source of Truth** (SDD workflow) |
| [Getting Started](docs/tutorial/getting-started.md) | Installation and basic usage guide |
| [Architecture](docs/architecture/) | Module design and implementation details |
| [Algorithms](docs/tutorial/algorithms.md) | Core algorithm tutorial |
| [API Reference](docs/api/) | Library usage documentation |
| [Changelog](CHANGELOG.md) | Version history and release notes |
| [Online Docs](https://lessup.github.io/bwa-rust/) | VitePress-powered documentation site |

## 📊 Comparison with BWA

| Feature | BWA (C) | bwa-rust |
|---------|---------|----------|
| Index format | Multiple files (`.bwt`, `.sa`, `.pac`) | Single `.fm` |
| SA construction | DC3/IS O(n) | Doubling O(n log²n) |
| MEM finding | Bidirectional BWT | Unidirectional |
| Parallel | pthread | rayon |
| Safety | unsafe C code | Zero unsafe |
| Paired-end | ✅ Supported | 🚧 Planned (v0.3.0) |
| BAM output | ✅ Supported | 🚧 Planned (v0.4.0) |

## 🧪 Testing

```bash
cargo test                    # Run all tests
cargo test -- --nocapture     # Show output
cargo bench                   # Run benchmarks
```

## 🔧 Library Usage

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

## 🤝 Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## 📄 License

[MIT License](LICENSE)

## 🙏 Acknowledgments

Inspired by [BWA](https://github.com/lh3/bwa) by Heng Li.
