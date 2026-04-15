# bwa-rust

[![CI](https://github.com/LessUp/bwa-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/LessUp/bwa-rust/actions/workflows/ci.yml)
[![Docs](https://img.shields.io/badge/docs-GitHub%20Pages-blue)](https://lessup.github.io/bwa-rust/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

English | [简体中文](README.zh-CN.md)

---

A BWA-MEM style short-read aligner implemented in Rust from scratch.

> **Note**: This project follows the architecture and algorithms of BWA/BWA-MEM, but does not aim for 100% behavioral compatibility with the C version (CLI options, index format, MAPQ details may differ).

## ✨ Features

| Feature | Description |
|---------|-------------|
| **FM-Index** | Suffix array + BWT + sparse SA sampling, serialized to a single `.fm` file |
| **SMEM Seeding** | Super-Maximal Exact Match seed finding with incremental left-extension |
| **Seed Chaining** | DP-based chain scoring with greedy peeling and filtering |
| **Smith-Waterman** | Banded local alignment with affine gap penalties and CIGAR generation |
| **SAM Output** | Standard format with @HD/@SQ/@PG, CIGAR, MAPQ, AS/XS/NM tags |
| **Multi-threaded** | Parallel read processing via rayon with configurable thread count |
| **Memory Safe** | Zero `unsafe` code, jemalloc allocator on non-Windows |
| **Configurable** | `max_occ`, `max_chains_per_contig`, `max_alignments_per_read` to prevent memory explosion |

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
- Linux, macOS, or Windows

## 🚀 Quick Start

### Build Index

```bash
bwa-rust index reference.fa -o ref
# Creates: ref.fm
```

### Align Reads

```bash
# Using pre-built index
bwa-rust align -i ref.fm reads.fq > output.sam

# One-step BWA-MEM style
bwa-rust mem reference.fa reads.fq > output.sam

# Multi-threaded
bwa-rust mem reference.fa reads.fq -t 4 -o output.sam

# Custom scoring
bwa-rust mem reference.fa reads.fq \
    --match 1 --mismatch 4 --gap-open 6 --gap-ext 1
```

## 📖 CLI Reference

### `index` — Build FM Index

```bash
bwa-rust index <reference.fa> -o <prefix>
```

| Argument | Default | Description |
|----------|---------|-------------|
| `reference` | required | FASTA reference file |
| `-o, --output` | `ref` | Output prefix for `.fm` index |

### `align` — Align Against Index

```bash
bwa-rust align -i <index.fm> <reads.fq> [options]
```

| Option | Default | Description |
|--------|---------|-------------|
| `-i, --index` | required | Path to `.fm` index |
| `reads` | required | FASTQ file |
| `-o, --out` | stdout | Output SAM file |
| `-t, --threads` | 1 | Number of threads |
| `--match` | 2 | Match score |
| `--mismatch` | 1 | Mismatch penalty |
| `--gap-open` | 2 | Gap open penalty |
| `--gap-ext` | 1 | Gap extension penalty |
| `--band-width` | 16 | SW band width |
| `--score-threshold` | 20 | Minimum alignment score |

### `mem` — One-Step Alignment

```bash
bwa-rust mem <reference.fa> <reads.fq> [options]
```

Uses BWA-MEM defaults: match=1, mismatch=4, gap-open=6, gap-ext=1.

| Option | Default | Description |
|--------|---------|-------------|
| `-o, --out` | stdout | Output SAM file |
| `-t, --threads` | 1 | Number of threads |
| `-A, --match` | 1 | Match score |
| `-B, --mismatch` | 4 | Mismatch penalty |
| `-O, --gap-open` | 6 | Gap open penalty |
| `-E, --gap-ext` | 1 | Gap extension penalty |
| `-w, --band-width` | 100 | SW band width |
| `-T, --score-threshold` | 10 | Minimum score |

## 📁 Project Structure

```
bwa-rust/
├── src/
│   ├── main.rs          # CLI entry (clap)
│   ├── lib.rs           # Library entry
│   ├── error.rs         # BwaError / BwaResult<T>
│   ├── io/              # FASTA/FASTQ parsing, SAM output
│   ├── index/           # FM-index (SA, BWT, FM, Builder)
│   ├── align/           # Alignment (SMEM, Chain, SW, Pipeline)
│   └── util/            # DNA encoding/decoding/revcomp
├── tests/               # Integration tests
├── benches/             # Criterion benchmarks
├── examples/            # Example code
├── data/                # Test data
└── docs/                # Architecture, tutorial
```

## 🧪 Testing

```bash
cargo test                    # Run all tests
cargo test <test_name>        # Run specific test
cargo test -- --nocapture     # Show output
cargo bench                   # Run benchmarks
cargo run --example simple_align
```

**Test Coverage**: 151 unit + 11 integration + 1 doc test ✅

## 📚 Documentation

| Document | Description |
|----------|-------------|
| [Architecture](docs/architecture.md) | Module design, index format, algorithm flow |
| [Tutorial](docs/tutorial.md) | Build a BWA-style aligner from scratch |
| [Roadmap](ROADMAP.md) | Development roadmap & version strategy |
| [Changelog](CHANGELOG.md) | Version changelog |
| [GitHub Pages](https://lessup.github.io/bwa-rust/) | Online documentation |

## 📊 Comparison with BWA

| Feature | BWA (C) | bwa-rust |
|---------|---------|----------|
| Index format | Multiple files | Single `.fm` file |
| SA construction | DC3/IS O(n) | Doubling O(n log²n) |
| MEM finding | Bidirectional BWT | Unidirectional extension |
| Multi-threading | pthread | rayon |
| Paired-end | ✓ | Planned (v0.2.0) |
| BAM output | ✓ | Planned (v0.4.0) |

## 🔧 Library Usage

```rust
use bwa_rust::index::{sa, bwt, fm};
use bwa_rust::util::dna;

// Build FM index
let reference = b"ACGTACGT";
let norm = dna::normalize_seq(reference);
let mut text: Vec<u8> = norm.iter().map(|&b| dna::to_alphabet(b)).collect();
text.push(0);

let sa_arr = sa::build_sa(&text);
let bwt_arr = bwt::build_bwt(&text, &sa_arr);
let contigs = vec![fm::Contig { name: "ref".into(), len: 8, offset: 0 }];
let fm_idx = fm::FMIndex::build(text, bwt_arr, sa_arr, contigs, 6, 4);

// Search for pattern
let pattern: Vec<u8> = b"CGT".iter().map(|&b| dna::to_alphabet(b)).collect();
if let Some((l, r)) = fm_idx.backward_search(&pattern) {
    println!("Found {} occurrences", r - l);
}
```

## 🤝 Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md).

## 📄 License

[MIT License](LICENSE)

## 🙏 Acknowledgments

Inspired by [BWA](https://github.com/lh3/bwa) by Heng Li.
