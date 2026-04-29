# bwa-rust

[![CI](https://github.com/LessUp/bwa-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/LessUp/bwa-rust/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/LessUp/bwa-rust?color=blue)](https://github.com/LessUp/bwa-rust/releases)
[![Docs](https://img.shields.io/badge/docs-GitHub%20Pages-blue)](https://lessup.github.io/bwa-rust/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](rust-toolchain.toml)

Memory-safe BWA-MEM-style single-end DNA short-read aligner in Rust.

English | [简体中文](README.zh-CN.md) | [Documentation](https://lessup.github.io/bwa-rust/)

## What It Ships

| Area | Status | Notes |
|------|--------|-------|
| FASTA reference input | Shipped | Multi-contig references are normalized to the project alphabet. |
| FASTQ single-end reads | Shipped | Single-end reads are the supported production scope. |
| FM-index | Shipped | Suffix array + BWT + sampled Occ table in one `.fm` file. |
| BWA-MEM-style alignment | Shipped | SMEM seeds, seed chaining, banded Smith-Waterman extension. |
| SAM output | Shipped | Header, CIGAR, MAPQ, AS/XS/NM, MD:Z, and SA:Z where available. |
| Parallel alignment | Shipped | Read-level parallelism with rayon. |
| Paired-end alignment | Planned | Reader/statistics groundwork exists; CLI pipeline is still single-end. |
| BAM/CRAM output | Planned | SAM is the only shipped output format. |

## Why Use It

- Zero `unsafe` code, enforced by repository lints.
- A compact single-file index that is easier to move than BWA's multi-file index set.
- A readable Rust implementation of the classic seed-chain-extend alignment pipeline.
- A library + CLI surface suitable for Rust bioinformatics experiments and learning.

Use original BWA for production paired-end pipelines, exact BWA compatibility, or mature BAM/CRAM workflows.

## Install

Download a release asset from [GitHub Releases](https://github.com/LessUp/bwa-rust/releases):

| Platform | Asset |
|----------|-------|
| Linux static | `bwa-rust-linux-amd64-static.tar.gz` |
| Linux glibc | `bwa-rust-linux-amd64.tar.gz` |
| macOS Intel | `bwa-rust-macos-amd64.tar.gz` |
| macOS Apple Silicon | `bwa-rust-macos-arm64.tar.gz` |
| Windows | `bwa-rust-windows-amd64.zip` |

Build from source:

```bash
git clone https://github.com/LessUp/bwa-rust.git
cd bwa-rust
cargo build --release
```

## Quick Start

```bash
# Build a single-file FM index
bwa-rust index reference.fa -o ref

# Align single-end reads with a prebuilt index
bwa-rust align -i ref.fm reads.fq -o output.sam

# Build the index in memory and align in one command
bwa-rust mem reference.fa reads.fq -t 4 -o output.sam
```

Default alignment parameters come from `src/align/mod.rs` `AlignOpt::default()`:

| Parameter | Default |
|-----------|---------|
| match / mismatch | `2 / 1` |
| gap open / extend | `2 / 1` |
| band width | `16` |
| score threshold | `20` |
| min seed length | `19` |
| max seed occurrences | `500` |
| max chains / alignments | `5 / 5` |
| z-drop | `100` |

## Architecture

```text
FASTA/FASTQ -> FM-index -> SMEM seeds -> chains -> Smith-Waterman -> SAM
```

Key modules:

- `src/index/`: suffix array, BWT, FM-index serialization.
- `src/align/`: seeding, chaining, extension, MAPQ, supplementary classification, pipeline.
- `src/io/`: FASTA, FASTQ, SAM parsing/formatting.
- `src/util/dna.rs`: DNA normalization, alphabet mapping, reverse complement.

## Development

OpenSpec is the source of truth for behavior and repository governance: see `openspec/specs/`.

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
npm run docs:build
```

For the project-specific AI workflow, see `AGENTS.md` and `docs/development/ai-workflow.md`.

## Documentation

- Public portal: <https://lessup.github.io/bwa-rust/>
- API docs: <https://docs.rs/bwa-rust>
- Specs: `openspec/specs/`
- Roadmap: `ROADMAP.md`
- Changelog: `CHANGELOG.md`

## License

MIT. Inspired by Heng Li's [BWA](https://github.com/lh3/bwa).
