# bwa-rust

[![CI](https://github.com/LessUp/bwa-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/LessUp/bwa-rust/actions/workflows/ci.yml)
[![Docs](https://img.shields.io/badge/docs-GitHub%20Pages-blue)](https://lessup.github.io/bwa-rust/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

English | [简体中文](README.zh-CN.md)

A BWA-MEM style short-read aligner implemented in Rust from scratch. Supports FASTA/FASTQ input and SAM output.

## Features

- **FM-Index Construction** — Suffix array, BWT, occurrence table, with configurable SA sampling rate
- **SMEM Seeding** — Super-Maximal Exact Match seed finding with bidirectional extension
- **Seed Chaining** — Dynamic programming-based seed chain scoring
- **Smith-Waterman** — Banded local alignment with affine gap penalties and CIGAR generation
- **Full Pipeline** — Index building + read mapping in a single binary
- **FASTA/FASTQ I/O** — Multi-line FASTA and standard FASTQ parsing
- **SAM Output** — Standard SAM format with proper flags, MAPQ, and timestamps

## Project Structure

```
├── src/
│   ├── main.rs          # CLI entry (clap)
│   ├── lib.rs           # Library entry
│   ├── error.rs         # Custom error types (BwaError / BwaResult)
│   ├── io/              # FASTA/FASTQ parsing, SAM output
│   ├── index/           # FM index (SA, BWT, FM, Builder)
│   ├── align/           # Alignment (SMEM, Chain, SW, Pipeline)
│   └── util/            # DNA encoding/decoding/reverse complement
├── tests/               # Integration tests
├── benches/             # Benchmarks
├── examples/            # Examples
├── data/                # Test data (toy.fa / toy_reads.fq)
└── docs/                # Architecture, tutorial, full replication plan
```

## Quick Start

### Build from Source

```bash
git clone https://github.com/LessUp/bwa-rust.git
cd bwa-rust
cargo build --release
```

Binary at `target/release/bwa-rust`.

### Requirements

- Rust 1.70+
- Linux, macOS, Windows

### Run Tests

```bash
cargo test
# test result: ok. 133 passed; 0 failed; 0 ignored
```

### Benchmark

```bash
cargo bench
```

### Example

```bash
cargo run --example simple_align
```

## Documentation

| Document | Description |
|----------|-------------|
| [`docs/architecture.md`](docs/architecture.md) | Module architecture, index format, algorithm flow |
| [`docs/tutorial.md`](docs/tutorial.md) | Tutorial: implement a BWA-style aligner from scratch |
| [`docs/plan.md`](docs/plan.md) | Full BWA replication roadmap |
| [`ROADMAP.md`](ROADMAP.md) | Development roadmap & version strategy |
| [`CHANGELOG.md`](CHANGELOG.md) | Version changelog |

## Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[MIT License](LICENSE)
