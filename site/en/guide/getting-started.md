# Getting Started

## Requirements

- **Rust** 1.70 or later
- Supports Linux, macOS, Windows

## Installation

### Build from Source

```bash
git clone https://github.com/LessUp/bwa-rust.git
cd bwa-rust
cargo build --release
```

The compiled binary is located at `target/release/bwa-rust`.

## Basic Usage

### Build Index

```bash
cargo run --release -- index data/toy.fa -o data/toy
```

### Align Reads

```bash
# Basic alignment
cargo run --release -- align -i data/toy.fm data/toy_reads.fq

# Output to file
cargo run --release -- align -i data/toy.fm data/toy_reads.fq -o output.sam

# Multi-threaded alignment
cargo run --release -- align -i data/toy.fm data/toy_reads.fq -t 4

# Custom scoring parameters
cargo run --release -- align -i data/toy.fm data/toy_reads.fq \
    --match 2 --mismatch 1 --gap-open 2 --gap-ext 1 --band-width 16
```

## Index Format

The `index` subcommand accepts a FASTA file and performs the following steps:

1. **Read reference**: Parse FASTA records, normalize bases to `{A,C,G,T,N}`
2. **Encode to alphabet**: `{0:$, 1:A, 2:C, 3:G, 4:T, 5:N}`, contigs separated by `$`
3. **Build suffix array**: Doubling algorithm O(n log²n)
4. **Build BWT**: Derive Burrows-Wheeler Transform from SA
5. **Build FM index**: Compute C table and block Occ sampling
6. **Serialize**: Write to `.fm` file using bincode

The index file contains a magic number (`BWAFM_RS`) and version number (v2) for compatibility checking.

## Features

### Supported

- Single-end read alignment
- SMEM seed finding + seed chaining
- Banded affine-gap Smith-Waterman local alignment
- SAM output (CIGAR, MAPQ, AS/XS/NM tags)
- Multi-threaded parallel processing

### Not Yet Supported

- Paired-end (PE) alignment (planned for v0.2.0)
- BAM output format
- Reading BWA native index files

## Library Usage

```bash
cargo run --example simple_align
```

bwa-rust also provides a library API for direct use in Rust projects. See the [Algorithm Tutorial](./tutorial.md) for details.
