# Architecture Design

## Context

bwa-rust is a BWA-MEM style DNA short-read aligner implemented in Rust. The architecture prioritizes memory safety, modularity, and performance.

## Goals / Non-Goals

**Goals:**
- Zero unsafe code with compile-time memory safety guarantees
- Modular architecture with clear separation of concerns
- Near BWA-MEM alignment accuracy and performance
- Single-file index format for easy deployment

**Non-Goals:**
- BWA native index file compatibility (different format)
- Real-time streaming alignment (batch processing model)
- GPU acceleration (CPU-only implementation)

## Decisions

### Three-Layer Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    CLI Layer (main.rs)                       │
│                  clap command parsing + dispatch            │
├─────────────┬─────────────┬──────────────┬──────────────────┤
│    io/      │   index/    │    align/    │      util/       │
│  I/O        │   Indexing  │  Alignment   │     Utilities    │
├─────────────┼─────────────┼──────────────┼──────────────────┤
│  FASTA      │   SA        │  Seed        │  DNA encoding    │
│  FASTQ      │   BWT       │  Chain       │  Rev complement  │
│  SAM        │   FM        │  SW/Extend   │                  │
│             │  Builder    │  Candidate   │                  │
│             │             │  MAPQ        │                  │
│             │             │  Pipeline    │                  │
└─────────────┴─────────────┴──────────────┴──────────────────┘
```

**Rationale:** Clear module boundaries enable independent testing and optimization. Each module has a single responsibility.

### Single-File Index Format

Store all index components in one `.fm` file using bincode:
- Magic header for validation
- Version for forward compatibility
- All components serialized in sequence

**Rationale:** Simpler than BWA's multi-file format (`.bwt`, `.sa`, `.pac`). Reduces deployment complexity and file management errors.

### Sparse SA Sampling

Store SA values at intervals (default rate: 4) instead of full array.

**Rationale:** 75% memory reduction with O(rate) query time. Acceptable trade-off for genomic data where index size matters.

### Rayon for Parallelism

Use rayon for read-level parallel processing.

**Rationale:** Safe, ergonomic parallelism with work-stealing. Zero data races guaranteed by Rust's borrow checker.

### Doubling Algorithm for SA

O(n log²n) suffix array construction instead of linear algorithms.

**Rationale:** Simpler implementation with acceptable performance for typical genome sizes. Linear algorithms (SA-IS, DC3) are more complex and error-prone.

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| SA construction slower than BWA | Acceptable for indexing (one-time cost) |
| Sparse SA increases query time | O(4) is effectively O(1) for most queries |
| jemalloc not available on Windows | Fall back to system allocator gracefully |
| Different index format than BWA | Document format clearly, no compatibility goal |

## Performance Characteristics

| Operation | Complexity | Typical Time |
|-----------|------------|--------------|
| SA construction | O(n log²n) | ~30s for 100M bp |
| BWT construction | O(n) | ~1s for 100M bp |
| Backward search | O(m) | <1µs for 100bp |
| Full alignment | O(m + w×l) | 0.1-1ms per read |

Where:
- n = reference length
- m = read length
- w = band width
- l = alignment length

## Module Dependencies

```
main.rs
    ├── io::fasta (parse reference)
    ├── io::fastq (parse reads)
    ├── io::sam (output alignments)
    ├── index::builder (build index)
    │       ├── index::sa (suffix array)
    │       ├── index::bwt (BWT)
    │       └── index::fm (FM-index)
    └── align::pipeline (alignment)
            ├── align::seed (SMEM finding)
            ├── align::chain (chain building)
            ├── align::sw (Smith-Waterman)
            ├── align::extend (chain extension)
            ├── align::candidate (candidate management)
            └── align::mapq (MAPQ estimation)
```

## Safety Guarantees

```toml
[lints.rust]
unsafe_code = "forbid"
```

All memory safety guaranteed by Rust compiler:
- No buffer overflows
- No use-after-free
- No null pointer dereferences
- No data races in parallel code
