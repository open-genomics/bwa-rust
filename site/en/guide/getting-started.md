# Getting Started

## Requirements

| Requirement | Version |
|-------------|---------|
| Rust | 1.70+ (MSRV) |
| Platform | Linux / macOS / Windows |

---

## Installation

### Build from Source

```bash
git clone https://github.com/LessUp/bwa-rust.git
cd bwa-rust
cargo build --release
```

The compiled binary is located at `target/release/bwa-rust`.

### Download Pre-built Binary

Download the binary for your platform from [GitHub Releases](https://github.com/LessUp/bwa-rust/releases).

---

## Basic Usage

### Build Index

```bash
bwa-rust index data/toy.fa -o data/toy
# Output: data/toy.fm
```

### Align Reads

```bash
# Basic alignment
bwa-rust align -i data/toy.fm data/toy_reads.fq

# Output to file
bwa-rust align -i data/toy.fm data/toy_reads.fq -o output.sam

# Multi-threaded alignment
bwa-rust align -i data/toy.fm data/toy_reads.fq -t 4

# Custom scoring parameters
bwa-rust align -i data/toy.fm data/toy_reads.fq \
    --match 2 --mismatch 1 --gap-open 2 --gap-ext 1
```

### One-Step Alignment (BWA-MEM Style)

```bash
# Build index and align
bwa-rust mem data/toy.fa data/toy_reads.fq -t 4 -o output.sam

# Use BWA-MEM default scoring
bwa-rust mem data/toy.fa data/toy_reads.fq -A 1 -B 4 -O 6 -E 1
```

---

## CLI Reference

### `index` — Build Index

```bash
bwa-rust index <reference.fa> -o <prefix>
```

| Argument | Default | Description |
|----------|---------|-------------|
| `<reference.fa>` | required | FASTA reference file |
| `-o, --output` | `ref` | Output prefix for `.fm` index |

### `align` — Align Against Index

```bash
bwa-rust align -i <index.fm> <reads.fq> [options]
```

| Option | Default | Description |
|--------|---------|-------------|
| `-i, --index` | required | Path to `.fm` index |
| `<reads.fq>` | required | FASTQ file |
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

---

## Features

### ✅ Supported

| Feature | Description |
|---------|-------------|
| Single-end alignment | Forward + reverse complement bidirectional |
| SMEM seed finding | Super-Maximal Exact Match, `max_occ` filtering |
| Seed chaining | DP + greedy peeling, `max_chains` limit |
| Banded SW alignment | Affine gap, semi-global refinement |
| SAM output | Full header, CIGAR, MAPQ, AS/XS/NM |
| Multi-threading | Rayon data parallelism |

### 📋 Planned

| Feature | Version |
|---------|---------|
| Paired-end (PE) alignment | v0.2.0 |
| BWA native index reading | v0.3.0 |
| BAM output format | v0.4.0 |
| SIMD acceleration | v0.5.0 |

---

## Library Usage

```bash
cargo run --example simple_align
```

bwa-rust provides a library API for direct use in Rust projects. See the [Algorithm Tutorial](./tutorial.md) for details.

---

## FAQ

### Why do results differ from BWA?

bwa-rust **does not aim for 100% behavioral compatibility**:

- Different index format (single `.fm` vs multi-file)
- Simplified MAPQ calculation
- Some edge case handling may differ

### How to handle memory issues?

Use memory protection parameters:

```bash
# Reduce repetitive seeds
bwa-rust align -i ref.fm reads.fq --max-occ 100

# Reduce candidate chains
bwa-rust mem ref.fa reads.fq --max-chains 3

# Reduce output count
bwa-rust mem ref.fa reads.fq --max-alignments 3
```

### How to improve performance?

1. **Increase threads**: `-t 8`
2. **Adjust bandwidth**: smaller is faster but less sensitive to indels
3. **Raise threshold**: fewer secondary alignments
