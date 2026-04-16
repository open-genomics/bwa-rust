# Algorithm Tutorial

Understand the core algorithms and implementation of bwa-rust from scratch.

## Overview

In this tutorial, you will learn:

1. Building an FM index (Suffix Array → BWT → C/Occ tables)
2. Exact matching with FM index
3. SMEM seed finding
4. Sequence alignment via seed chaining + Smith-Waterman
5. Standard SAM format output

## Step 1: Understanding the FM Index

The FM index is a full-text index based on BWT that supports efficient substring search.

### Construction Flow

```
Reference "ACGT$"
    │
    ▼
Suffix Array (SA)  → All suffixes sorted lexicographically
    │
    ▼
BWT                → Character preceding each SA position
    │
    ▼
C table            → Cumulative character frequencies
Occ sampling       → Character occurrence counts up to each position
```

### Rust Code Example

```rust
use bwa_rust::index::{sa, bwt, fm};
use bwa_rust::util::dna;

// Encode reference sequence
let reference = b"ACGTACGT";
let norm = dna::normalize_seq(reference);
let mut text: Vec<u8> = norm.iter().map(|&b| dna::to_alphabet(b)).collect();
text.push(0); // Add sentinel

// Build index
let sa_arr = sa::build_sa(&text);
let bwt_arr = bwt::build_bwt(&text, &sa_arr);
let contigs = vec![fm::Contig { name: "ref".into(), len: 8, offset: 0 }];
let fm_idx = fm::FMIndex::build(text, bwt_arr, sa_arr, contigs, 6, 4);
```

## Step 2: Exact Matching

The FM index backward search finds a pattern of length m in O(m) time.

```rust
let pattern = b"CGT";
let pattern_alpha: Vec<u8> = dna::normalize_seq(pattern)
    .iter().map(|&b| dna::to_alphabet(b)).collect();

if let Some((l, r)) = fm_idx.backward_search(&pattern_alpha) {
    let count = r - l; // Number of occurrences
    let positions = fm_idx.sa_interval_positions(l, r);
    println!("'CGT' occurs {} times", count);
}
```

## Step 3: SMEM Seed Finding

SMEM (Super-Maximal Exact Match) is a core concept in BWA-MEM. For each position on the read, find the longest exact match covering that position.

### Basic Usage

```rust
use bwa_rust::align::find_smem_seeds;

let read = b"ACGTACGT";
let read_alpha: Vec<u8> = dna::normalize_seq(read)
    .iter().map(|&b| dna::to_alphabet(b)).collect();

let seeds = find_smem_seeds(&fm_idx, &read_alpha, 5); // min_len=5

for seed in &seeds {
    println!("read[{}..{}] matches ref[{}..{}]",
        seed.qb, seed.qe, seed.rb, seed.re);
}
```

### Memory Protection: Filter Repetitive Seeds

For highly repetitive sequences, use `max_occ` to skip:

```rust
use bwa_rust::align::find_smem_seeds_with_max_occ;

// Only keep seeds with <= 500 occurrences
let seeds = find_smem_seeds_with_max_occ(&fm_idx, &read_alpha, 5, 500);
```

## Step 4: Seed Chaining

Combine multiple seeds into a "chain", selecting seed combinations with the best coverage and reasonable gaps.

```rust
use bwa_rust::align::{build_chains, filter_chains};

let mut chains = build_chains(&seeds, read_len);
filter_chains(&mut chains, 0.3); // Filter weak chains

// chains[0] is the highest-scoring chain
```

### Limit Chain Count

```rust
use bwa_rust::align::build_chains_with_limit;

// Extract at most 5 chains per contig
let chains = build_chains_with_limit(&seeds, read_len, 5);
```

## Step 5: Smith-Waterman Alignment

Perform banded affine-gap Smith-Waterman local alignment in gap regions between seeds.

### Parameter Configuration

```rust
use bwa_rust::align::{banded_sw, SwParams};

let sw_params = SwParams {
    match_score: 2,
    mismatch_penalty: 1,
    gap_open: 2,
    gap_extend: 1,
    band_width: 16,
};

let result = banded_sw(query, ref_seq, sw_params);
println!("Score: {}, CIGAR: {}, NM: {}", result.score, result.cigar, result.nm);
```

### BWA-MEM Default Parameters

| Parameter | Value |
|-----------|-------|
| match | 1 |
| mismatch | 4 |
| gap_open | 6 |
| gap_ext | 1 |
| band_width | 100 |

## Step 6: Full Configuration

Use `AlignOpt` for complete parameter configuration:

```rust
use bwa_rust::align::AlignOpt;

let opt = AlignOpt {
    // Scoring parameters
    match_score: 2,
    mismatch_penalty: 1,
    gap_open: 2,
    gap_extend: 1,
    clip_penalty: 1,

    // Alignment parameters
    band_width: 16,
    score_threshold: 20,
    min_seed_len: 19,

    // Parallelism
    threads: 4,

    // Memory protection
    max_occ: 500,              // Skip highly repetitive seeds
    max_chains_per_contig: 5,  // Max chains per contig
    max_alignments_per_read: 5, // Max output alignments
};

// Validate parameters
opt.validate().expect("Invalid AlignOpt");
```

## Complete Pipeline

```
FASTA reference
    → FM index construction (SA → BWT → C/Occ)
    → Serialize to .fm file

FASTQ reads
    → Load FM index
    → For each read:
        → SMEM seed finding (forward + reverse complement, max_occ filter)
        → Seed chaining and filtering (max_chains limit)
        → Chain → SW extension → full alignment (semi-global refinement)
        → Candidate dedup, MAPQ estimation
    → Output SAM (max_alignments limit)
```

## Run Example

```bash
cargo run --example simple_align
```

For detailed source code, visit the [GitHub repository](https://github.com/LessUp/bwa-rust).
