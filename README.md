# bwa-rust

<div align="center">

[![CI](https://github.com/LessUp/bwa-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/LessUp/bwa-rust/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/LessUp/bwa-rust?color=blue)](https://github.com/LessUp/bwa-rust/releases)
[![Docs](https://img.shields.io/badge/docs-GitHub%20Pages-blue)](https://lessup.github.io/bwa-rust/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Coverage](https://codecov.io/gh/LessUp/bwa-rust/branch/master/graph/badge.svg)](https://codecov.io/gh/LessUp/bwa-rust)
[![Crates.io](https://img.shields.io/crates/v/bwa-rust)](https://crates.io/crates/bwa-rust)

**A high-performance BWA-MEM style DNA sequence aligner in Rust**

*Zero unsafe code • Multi-threaded • Single-file index*

English | [简体中文](README.zh-CN.md) | [📖 Documentation](https://lessup.github.io/bwa-rust/)

</div>

---

## Overview

bwa-rust is a BWA-MEM style short-read aligner implemented in Rust from scratch. It provides a clean, memory-safe implementation of core BWA-MEM algorithms with a simplified index format and modern parallelism.

### Key Features

| Feature | Description |
|---------|-------------|
| 🔒 **Memory Safe** | Zero `unsafe` code, enforced by `forbid(unsafe_code)` lint |
| 🚀 **Multi-threaded** | Read-level parallelism via rayon, jemalloc allocator |
| 📦 **Simple Index** | Single `.fm` file vs. multiple BWA index files |
| 🎯 **Standard Output** | Full SAM format with CIGAR, MAPQ, AS/XS/NM tags |
| 🔧 **Memory Protected** | Configurable limits to handle repetitive sequences |
| 📖 **Readable Code** | Educational-grade implementation for learning alignment algorithms |

### Use Cases

- **Rust integration**: Library for building bioinformatics pipelines in Rust
- **Memory safety**: Processing reference genomes with strict safety requirements
- **Learning**: Clean architecture for understanding BWA-MEM algorithms
- **Single-end alignment**: Stable baseline aligner for SE reads

> ⚠️ **Current Scope**: This is a **single-end aligner**. Paired-end support is planned but not yet available. For production PE alignment, use the original [BWA](https://github.com/lh3/bwa).

---

## Installation

### Quick Install (Linux/macOS)

```bash
# Download latest release
curl -sL https://github.com/LessUp/bwa-rust/releases/latest/download/bwa-rust-linux-amd64-static.tar.gz | tar xz

# Move to PATH
sudo mv bwa-rust /usr/local/bin/

# Verify installation
bwa-rust --version
```

### Download Pre-built Binary

Download from [GitHub Releases](https://github.com/LessUp/bwa-rust/releases) for your platform:

| Platform | File | Size |
|----------|------|------|
| Linux (static, recommended) | `bwa-rust-linux-amd64-static.tar.gz` | ~3 MB |
| Linux (dynamic) | `bwa-rust-linux-amd64.tar.gz` | ~2 MB |
| macOS (Intel) | `bwa-rust-macos-amd64.tar.gz` | ~2 MB |
| macOS (Apple Silicon) | `bwa-rust-macos-arm64.tar.gz` | ~2 MB |
| Windows | `bwa-rust-windows-amd64.zip` | ~3 MB |

### Build from Source

**Requirements:** Rust 1.70+ on Linux, macOS, or Windows.

```bash
git clone https://github.com/LessUp/bwa-rust.git
cd bwa-rust
cargo build --release
```

Binary: `target/release/bwa-rust`

### Use as Library

Add to your `Cargo.toml`:

```toml
[dependencies]
bwa-rust = "0.2"
```

---

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

# Multi-threaded (4 threads)
bwa-rust mem ref.fa reads.fq -t 4 -o output.sam

# Use pre-built index
bwa-rust align -i ref.fm reads.fq -o output.sam
```

### Example Output

```
@HD	VN:1.6	SO:unsorted
@SQ	SN:chr1	LN:248956422
@PG	ID:bwa-rust	PN:bwa-rust	VN:0.2.0
read_001	0	chr1	10001	60	100M	*	0	0	ACGT...	!!!!...	AS:i:100	XS:i:0	NM:i:0
```

---

## Performance

Small-scale benchmarks on E. coli reference (~4.6Mbp) with 100bp reads show bwa-rust achieves ~70% of BWA-MEM throughput for single-end alignment:

| Metric | BWA-MEM | bwa-rust | Notes |
|--------|---------|----------|-------|
| Index Build | ~2s | ~3s | O(n log²n) doubling algorithm vs O(n) |
| Index Size | ~5MB (multiple files) | ~5MB (single `.fm`) | Comparable total size |
| Align (1 thread) | ~10K reads/s | ~7K reads/s | Single-end only |
| Align (8 threads) | ~35K reads/s | ~25K reads/s | rayon parallelism |
| Memory Usage | ~150MB | ~150MB | Comparable for small genomes |

> **Note**: These are indicative results from small-scale tests. Comprehensive benchmarking on human-scale genomes is planned. See `cargo bench` for reproducible micro-benchmarks.

---

## Features

### Core Components

| Component | Implementation |
|-----------|---------------|
| **FM-Index** | Suffix array + BWT + sparse SA sampling |
| **SMEM Seeding** | Super-Maximal Exact Match finding with left-extension |
| **Seed Chaining** | DP-based chain scoring with greedy multi-chain extraction |
| **Smith-Waterman** | Banded local alignment with affine gap penalty |
| **SAM Output** | Standard format with @HD/@SQ/@PG headers |

## Comparison with BWA

| Feature | BWA (C) | bwa-rust |
|---------|---------|----------|
| Index format | Multiple files (`.bwt`, `.sa`, `.pac`) | Single `.fm` |
| SA construction | DC3/IS O(n) | Doubling O(n log²n) |
| MEM finding | Bidirectional BWT | Backward search |
| Parallelism | pthread | rayon |
| Memory safety | Manual memory management | Zero `unsafe` (compiler-verified) |
| Paired-end | ✅ Supported | 🚧 Planned |
| BAM output | ✅ Supported | 🚧 Planned |

**Compatibility**: bwa-rust follows BWA-MEM algorithms but does not aim for 100% behavioral compatibility. Index formats, MAPQ calculations, and some heuristics differ intentionally.

---

## Documentation

| Resource | Link |
|----------|------|
| **Documentation Site** | [lessup.github.io/bwa-rust](https://lessup.github.io/bwa-rust/) |
| Architecture Guide | [Architecture Overview](https://lessup.github.io/bwa-rust/architecture/) |
| Installation & Usage | [User Guide](https://lessup.github.io/bwa-rust/guide/) |
| Performance Data | [Benchmarks](https://lessup.github.io/bwa-rust/benchmarks) |
| FAQ | [Common Questions](https://lessup.github.io/bwa-rust/faq) |
| Specifications | [openspec/specs/](openspec/specs/) (OpenSpec workflow) |
| Changelog | [CHANGELOG.md](CHANGELOG.md) |

---

## Library Usage

### Basic Example

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

// Use the FM index for exact matching
let pattern: Vec<u8> = b"ACGT".iter().map(|&b| dna::to_alphabet(b)).collect();
if let Some((l, r)) = fm_idx.backward_search(&pattern) {
    println!("Found matches in range [{}, {}]", l, r);
}
```

### Alignment Example

```rust
use bwa_rust::index::fm;
use bwa_rust::align::{find_smem_seeds, AlignOpt};
use bwa_rust::util::dna;

// Load FM index
let fm_idx = fm::FMIndex::load_from_file("reference.fm")?;

// Configure alignment options
let opt = AlignOpt {
    min_seed_len: 19,
    max_occ: 1000,
    ..AlignOpt::default()
};

// Find SMEM seeds for a read
let read = b"ACGTACGTACGTACGTACGT";
let norm = dna::normalize_seq(read);
let alpha: Vec<u8> = norm.iter().map(|&b| dna::to_alphabet(b)).collect();

let seeds = find_smem_seeds(&fm_idx, &alpha, opt.min_seed_len);
for seed in seeds {
    println!("Seed: len={}, contig={}, query=[{}..{}], ref=[{}..{}]",
        seed.qe - seed.qb, seed.contig, seed.qb, seed.qe, seed.rb, seed.re);
}
```

See [Library Usage Guide](docs/api/library-usage.md) for more examples.

---

## Development

```bash
# Run tests
cargo test

# Run specific test
cargo test test_name -- --nocapture

# Run benchmarks
cargo bench

# Check code quality
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings

# Build documentation
cargo doc --open
```

---

## FAQ

**Q: Is bwa-rust compatible with BWA indices?**  
A: No, bwa-rust uses a different single-file `.fm` index format. You need to rebuild the index using `bwa-rust index`.

**Q: Can I use bwa-rust for production workflows?**  
A: bwa-rust is suitable for single-end alignment and Rust library integration. For production paired-end workflows, use the original BWA until PE support ships in bwa-rust.

**Q: What file formats are supported?**  
A: **Input**: FASTA (reference), FASTQ (single-end reads). **Output**: SAM. BAM output is planned for future versions.

**Q: How do I report issues or request features?**  
A: Please use [GitHub Issues](https://github.com/LessUp/bwa-rust/issues) and check existing issues first.

---

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

[MIT License](LICENSE)

## Acknowledgments

Inspired by [BWA](https://github.com/lh3/bwa) by Heng Li.

---

<div align="center">

[![Star History](https://api.star-history.com/svg?repos=LessUp/bwa-rust&type=Date)](https://star-history.com/#LessUp/bwa-rust&Date)

</div>
