# Library Usage Guide

> Using bwa-rust as a Rust library.

---

## Table of Contents

- [Overview](#overview)
- [Basic Example](#basic-example)
- [Working with FM-Index](#working-with-fm-index)
- [Alignment Pipeline](#alignment-pipeline)
- [API Reference](#api-reference)

---

## Overview

bwa-rust can be used as a library in your Rust projects:

```toml
[dependencies]
bwa-rust = { path = "path/to/bwa-rust" }
```

---

## Basic Example

```rust
use bwa_rust::index::{sa, bwt, fm};
use bwa_rust::util::dna;

fn main() {
    // Reference sequence
    let reference = b"ACGTACGTACGT";
    
    // Normalize and encode
    let norm = dna::normalize_seq(reference);
    let mut text: Vec<u8> = norm.iter().map(|&b| dna::to_alphabet(b)).collect();
    text.push(0);  // sentinel
    
    // Build suffix array
    let sa_arr = sa::build_sa(&text);
    
    // Build BWT
    let bwt_arr = bwt::build_bwt(&text, &sa_arr);
    
    // Build FM index
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
        6,  // alphabet size
        4,  // SA sample rate
    );
    
    // Search pattern
    let pattern: Vec<u8> = b"CGT".iter()
        .map(|&b| dna::to_alphabet(b))
        .collect();
    
    if let Some((l, r)) = fm_idx.backward_search(&pattern) {
        println!("Found {} occurrences", r - l);
    }
}
```

---

## Working with FM-Index

### Building from FASTA

```rust
use bwa_rust::index::builder;
use std::fs::File;

fn build_index(fasta_path: &str, output: &str) {
    let mut file = File::open(fasta_path).unwrap();
    let fm_idx = builder::build_from_fasta(&mut file, 6, 4).unwrap();
    
    // Serialize to file
    fm_idx.save(output).unwrap();
}
```

### Loading and Searching

```rust
use bwa_rust::index::fm::FMIndex;
use bwa_rust::util::dna;

fn search_pattern(index_path: &str, pattern: &[u8]) {
    // Load index
    let fm = FMIndex::load(index_path).unwrap();
    
    // Encode pattern
    let encoded: Vec<u8> = dna::normalize_seq(pattern)
        .iter()
        .map(|&b| dna::to_alphabet(b))
        .collect();
    
    // Search
    if let Some((l, r)) = fm.backward_search(&encoded) {
        println!("Pattern occurs {} times", r - l);
        
        // Get positions
        for pos in fm.sa_interval_positions(l, r) {
            println!("  At position {}", pos);
        }
    }
}
```

---

## Alignment Pipeline

```rust
use bwa_rust::align::{AlignOpt, pipeline};
use bwa_rust::index::fm::FMIndex;
use bwa_rust::io::fastq;

fn align_reads(fm_path: &str, fastq_path: &str) {
    // Load index
    let fm = FMIndex::load(fm_path).unwrap();
    
    // Configure alignment
    let opt = AlignOpt {
        match_score: 2,
        mismatch_penalty: 1,
        gap_open: 2,
        gap_extend: 1,
        clip_penalty: 1,
        band_width: 16,
        score_threshold: 20,
        min_seed_len: 19,
        threads: 4,
        max_occ: 500,
        max_chains_per_contig: 5,
        max_alignments_per_read: 5,
    };
    
    // Parse FASTQ
    let reads = fastq::parse_file(fastq_path).unwrap();
    
    // Align
    let results = pipeline::align_reads(&fm, &reads, &opt);
    
    // Output SAM
    for record in results {
        println!("{}", record);
    }
}
```

---

## API Reference

### `index` Module

| Struct/Function | Description |
|-----------------|-------------|
| `FMIndex` | FM-index data structure |
| `build_sa()` | Build suffix array |
| `build_bwt()` | Build BWT from SA |
| `FMIndex::build()` | Build FM-index |
| `FMIndex::load()` | Load from file |
| `FMIndex::save()` | Save to file |

### `align` Module

| Struct/Function | Description |
|-----------------|-------------|
| `AlignOpt` | Alignment configuration |
| `banded_sw()` | Banded Smith-Waterman |
| `find_smem_seeds()` | Find SMEM seeds |
| `build_chains()` | Build seed chains |
| `pipeline::align_reads()` | Full alignment pipeline |

### `io` Module

| Function | Description |
|----------|-------------|
| `parse_fasta()` | Parse FASTA file |
| `parse_fastq()` | Parse FASTQ file |
| `write_sam_header()` | Write SAM header |
| `format_sam_record()` | Format SAM record |

### `util` Module

| Function | Description |
|----------|-------------|
| `normalize_seq()` | Normalize DNA sequence |
| `to_alphabet()` | Encode to 0-5 alphabet |
| `from_alphabet()` | Decode from 0-5 alphabet |
| `revcomp()` | Reverse complement |

---

## See Also

- [Tutorial](../tutorial/) — User guides
- [Architecture](../architecture/) — Implementation details
- [Development](../development/) — Contributing guide
