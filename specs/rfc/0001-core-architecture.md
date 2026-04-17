# RFC-0001: Core Architecture

> **Status**: Accepted  
> **Created**: 2026-02-13  
> **Updated**: 2026-04-16  
> **Version**: v0.2.0

## Summary

This document defines the core architecture of bwa-rust, a BWA-MEM style DNA short-read aligner implemented in Rust.

## Overall Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     CLI Layer (main.rs)                      │
│              Command parsing with clap + dispatch            │
├─────────────┬─────────────┬──────────────┬──────────────────┤
│    io/      │   index/    │    align/    │      util/       │
│    I/O      │   Indexing  │  Alignment   │     Utilities    │
├─────────────┼─────────────┼──────────────┼──────────────────┤
│  FASTA      │   SA        │  Seed        │  DNA encoding    │
│  FASTQ      │   BWT       │  Chain       │  Rev complement  │
│  SAM        │   FM        │  SW/Extend   │                  │
│             │  Builder    │  Candidate   │                  │
│             │             │  MAPQ        │                  │
│             │             │  Pipeline    │                  │
└─────────────┴─────────────┴──────────────┴──────────────────┘
```

### Design Principles

1. **Modularity** — Single responsibility per module, high cohesion, low coupling
2. **Memory Safety** — Zero `unsafe` code, compile-time safety guarantees
3. **Performance First** — Critical path optimizations (jemalloc, buffer reuse)

## Module Architecture

### 1. `io/` — I/O Layer

| File | Function | Key Functions |
|------|----------|---------------|
| `fasta.rs` | FASTA parsing | `parse_fasta()`, `normalize_seq()` |
| `fastq.rs` | FASTQ parsing | `parse_fastq_record()` |
| `sam.rs` | SAM output | `write_header()`, `format_record()` |

**FASTA Parsing Features:**
- Multi-contig support
- Auto-normalization (uppercase, filter non-standard chars)
- Supports various line endings (LF/CRLF)
- Empty sequence detection
- Duplicate contig name detection

### 2. `index/` — Index Building

| File | Function | Complexity |
|------|----------|------------|
| `sa.rs` | Suffix array construction | O(n log²n) |
| `bwt.rs` | BWT construction | O(n) |
| `fm.rs` | FM-index core | O(n) space |
| `builder.rs` | Build entry point | - |

### 3. `align/` — Alignment Algorithms

| File | Function | Key Functions |
|------|----------|---------------|
| `mod.rs` | Config definition | `AlignOpt`, `validate()` |
| `seed.rs` | SMEM seeds | `find_smem_seeds()` |
| `chain.rs` | Chain building | `build_chains()`, `filter_chains()` |
| `sw.rs` | Smith-Waterman | `banded_sw()` |
| `extend.rs` | Chain extension | `chain_to_alignment()` |
| `candidate.rs` | Candidate management | `collect_candidates()`, `dedup_candidates()` |
| `mapq.rs` | MAPQ estimation | `compute_mapq()` |
| `pipeline.rs` | Full pipeline | `align_reads()` |

### 4. `util/` — Utilities

| File | Function |
|------|----------|
| `dna.rs` | DNA encoding/decoding, reverse complement |

## Data Flow

```
FASTQ read
    │
    ├─ Forward normalization ─────────────┐
    │                                     │
    ├─ Reverse-complement normalization ──┤
    │                                     │
    ▼                                     ▼
┌──────────────────────────────────────────────┐
│ SMEM Seed Finding (seed.rs)                   │
│ • Find longest exact match for each position  │
│ • max_occ filtering for repetitive seeds      │
└──────────────────────────────────────────────┘
    │
    ▼
┌──────────────────────────────────────────────┐
│ Seed Chain Building (chain.rs)                │
│ • DP scoring + greedy peeling                 │
│ • max_chains_per_contig limit                 │
└──────────────────────────────────────────────┘
    │
    ▼
┌──────────────────────────────────────────────┐
│ SW Extension (extend.rs + sw.rs)              │
│ • Banded affine-gap local alignment           │
│ • Semi-global refinement                      │
│ • Generate CIGAR + NM                         │
└──────────────────────────────────────────────┘
    │
    ▼
┌──────────────────────────────────────────────┐
│ Candidate Deduplication & Sorting (candidate.rs)│
│ • Position/direction dedup                    │
│ • Clip penalty sorting                        │
└──────────────────────────────────────────────┘
    │
    ▼
┌──────────────────────────────────────────────┐
│ Primary/Secondary Output (pipeline.rs)        │
│ • max_alignments_per_read limit               │
│ • FLAG settings (0/16/256)                    │
└──────────────────────────────────────────────┘
    │
    ▼
  SAM output
```

## Technology Stack

| Dependency | Version | Purpose |
|------------|---------|---------|
| **Rust** | 2021 Edition (MSRV 1.70) | Systems programming language |
| **clap** | 4.5 | CLI argument parsing |
| **serde + bincode** | - | Index serialization |
| **rayon** | - | Data parallelism |
| **chrono** | - | Timestamps |
| **criterion** | - | Benchmarking |
| **anyhow** | - | Error handling (CLI) |
| **tikv-jemallocator** | - | Memory allocator (non-Windows) |

## Differences from BWA/BWA-MEM

| Aspect | BWA (C) | bwa-rust |
|--------|---------|----------|
| **Index Format** | Multiple files (`.bwt`, `.sa`, `.pac`) | Single `.fm` file |
| **SA Construction** | DC3/IS O(n) | Doubling O(n log²n) |
| **MEM Finding** | Bidirectional BWT | Unidirectional backward_search |
| **Chain Building** | Complex greedy+DP | Simplified DP + greedy peeling |
| **MAPQ** | Complex statistical model | Simplified score-diff ratio model |
| **Parallelism** | pthread | rayon |
| **Paired-end** | ✅ Supported | 📋 Planned (v0.2.0) |
| **BAM Output** | ✅ Supported | 📋 Planned (v0.4.0) |

## Safety Guarantees

```toml
[lints.rust]
unsafe_code = "forbid"  # Project-wide unsafe ban
```

- **Zero unsafe code**: All memory safety guaranteed by compiler
- **jemalloc allocator**: Non-Windows platforms use jemalloc for better multi-thread performance

## Performance Optimizations

| Optimization | Method | Effect |
|--------------|--------|--------|
| SA Storage | Sparse sampling (rate=4) | 75% memory reduction |
| SW Buffer | `SwBuffer` reuse | Reduced hot-path allocations |
| Multi-threading | rayon parallelism | Near-linear speedup on multi-core |
| Memory Allocator | jemalloc | Improved multi-thread throughput |

## Related Documents

- [RFC-0002: Index Building](./0002-index-building.md)
- [RFC-0003: Alignment Algorithm](./0003-alignment-algorithm.md)
