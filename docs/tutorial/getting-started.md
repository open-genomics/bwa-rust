# Getting Started with bwa-rust

> A quick start guide for using bwa-rust as a command-line tool and library.

---

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [CLI Commands](#cli-commands)
- [Library Usage](#library-usage)
- [Configuration](#configuration)
- [Performance Tuning](#performance-tuning)
- [Troubleshooting](#troubleshooting)

---

## Installation

### From Source

```bash
# Clone repository
git clone https://github.com/LessUp/bwa-rust.git
cd bwa-rust

# Build release binary
cargo build --release

# Binary location
target/release/bwa-rust
```

### Requirements

- Rust 1.70 or higher
- Linux, macOS, or Windows

### Verify Installation

```bash
bwa-rust --version
```

---

## Quick Start

### 1. Build FM Index

```bash
# From FASTA reference
bwa-rust index reference.fa -o ref

# Creates: ref.fm
```

### 2. Align Reads (Two-Step)

```bash
# Step 1: Build index (if not exists)
bwa-rust index reference.fa -o ref

# Step 2: Align
bwa-rust align -i ref.fm reads.fq -o output.sam
```

### 3. One-Step Alignment (BWA-MEM Style)

```bash
# Build index and align in one command
bwa-rust mem reference.fa reads.fq -o output.sam

# With multiple threads
bwa-rust mem reference.fa reads.fq -t 4 -o output.sam
```

### 4. Example with Test Data

```bash
# Use provided test data
cd data/

# Build index
bwa-rust index toy.fa -o toy

# Align
bwa-rust align -i toy.fm toy_reads.fq

# Or one-step
bwa-rust mem toy.fa toy_reads.fq
```

---

## CLI Commands

### `index` — Build FM Index

```bash
bwa-rust index <reference.fa> -o <prefix>
```

| Argument | Default | Description |
|----------|---------|-------------|
| `reference` | required | FASTA reference file |
| `-o, --output` | `ref` | Output prefix for `.fm` index |

**Example:**
```bash
bwa-rust index hg38.fa -o hg38   # Creates hg38.fm
```

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
| `--clip-penalty` | 1 | Soft clip penalty (for ranking) |
| `--band-width` | 16 | SW band width |
| `--score-threshold` | 20 | Minimum score to output |
| `--max-occ` | 500 | Skip seeds with >500 occurrences |
| `--max-chains` | 5 | Max chains per contig |
| `--max-alignments` | 5 | Max alignments per read |

**Examples:**
```bash
# Basic
bwa-rust align -i ref.fm reads.fq

# Multi-threaded
bwa-rust align -i ref.fm reads.fq -t 8 -o out.sam

# Custom scoring
bwa-rust align -i ref.fm reads.fq \
    --match 2 --mismatch 1 --gap-open 2 --gap-ext 1
```

### `mem` — One-Step Alignment

```bash
bwa-rust mem <reference.fa> <reads.fq> [options]
```

Uses BWA-MEM default scoring: match=1, mismatch=4, gap-open=6, gap-ext=1.

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

**Examples:**
```bash
# One-step with defaults
bwa-rust mem ref.fa reads.fq -o out.sam

# Multi-threaded
bwa-rust mem ref.fa reads.fq -t 4 -o out.sam

# Custom parameters
bwa-rust mem ref.fa reads.fq \
    -A 1 -B 4 -O 6 -E 1 \
    -w 100 -T 10 \
    -t 8 \
    -o out.sam
```

---

## Library Usage

### Basic Example

```rust
use bwa_rust::index::{sa, bwt, fm};
use bwa_rust::util::dna;

fn main() {
    // 1. Load or build reference
    let reference = b"ACGTACGTACGT";
    
    // 2. Normalize and encode
    let norm = dna::normalize_seq(reference);
    let mut text: Vec<u8> = norm.iter().map(|&b| dna::to_alphabet(b)).collect();
    text.push(0);  // sentinel
    
    // 3. Build index components
    let sa_arr = sa::build_sa(&text);
    let bwt_arr = bwt::build_bwt(&text, &sa_arr);
    
    // 4. Build FM index
    let contigs = vec![fm::Contig {
        name: "chr1".into(),
        len: 12,
        offset: 0,
    }];
    
    let fm_idx = fm::FMIndex::build(
        text,
        bwt_arr,
        sa_arr,
        contigs,
        6,  // sigma
        4,  // SA sample rate
    );
    
    // 5. Search pattern
    let pattern: Vec<u8> = b"CGT".iter().map(|&b| dna::to_alphabet(b)).collect();
    
    if let Some((l, r)) = fm_idx.backward_search(&pattern) {
        println!("Found {} occurrences", r - l);
        
        // Get positions
        for pos in fm_idx.sa_interval_positions(l, r) {
            println!("  At position {}", pos);
        }
    }
}
```

### Run Example

```bash
cargo run --example simple_align
```

---

## Configuration

### Memory Protection Limits

bwa-rust provides three configurable limits to prevent memory explosion on repetitive sequences:

```bash
# Limit repetitive seeds (default: 500)
bwa-rust align -i ref.fm reads.fq --max-occ 200

# Limit chains per contig (default: 5)
bwa-rust mem ref.fa reads.fq --max-chains 3

# Limit output alignments per read (default: 5)
bwa-rust mem ref.fa reads.fq --max-alignments 10
```

### Band Width Tuning

```bash
# Small bandwidth: fast but less tolerant to indels
bwa-rust align -i ref.fm reads.fq --band-width 16

# Large bandwidth: slower but more tolerant to indels
bwa-rust align -i ref.fm reads.fq --band-width 64
```

---

## Performance Tuning

### Multi-threading

```bash
# Use all available cores
bwa-rust mem ref.fa reads.fq -t $(nproc)

# Or specify manually
bwa-rust mem ref.fa reads.fq -t 8
```

### Memory Optimization

| Setting | Impact | Recommendation |
|---------|--------|----------------|
| `--max-occ 500` | Skip repetitive seeds | Keep at 500 for most cases |
| `--max-chains 5` | Limit chains per contig | Reduce to 3 for speed |
| `--max-alignments 5` | Limit output | Increase to 10 for more mappings |

---

## Troubleshooting

### Index Not Found

```
Error: Index file not found: ref.fm
```

**Solution:** Build the index first
```bash
bwa-rust index reference.fa -o ref
```

### Thread Count Errors

```
Error: --threads must be >= 1
```

**Solution:** Use `--threads 1` or higher. For single-thread, omit the flag.

### Low Mapping Rate

**Possible causes:**
1. `--score-threshold` too high
2. `--max-occ` too low (skipping valid seeds)
3. Band width too small for indels

**Solutions:**
```bash
# Lower threshold
bwa-rust align -i ref.fm reads.fq --score-threshold 10

# Increase max_occ
bwa-rust align -i ref.fm reads.fq --max-occ 1000

# Increase bandwidth
bwa-rust align -i ref.fm reads.fq --band-width 32
```

---

## Next Steps

- [Algorithm Tutorial](./algorithms.md) — Deep dive into FM-index and alignment algorithms
- [Architecture Details](../architecture/) — Module design and implementation
- [API Documentation](../api/) — Library API reference
