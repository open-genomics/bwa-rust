# Architecture

## Overview

bwa-rust uses a modular design with five main modules:

```
┌─────────────────────────────────────────────────────────┐
│                     main.rs (CLI)                       │
│              clap argument parsing + dispatch            │
├──────────┬──────────┬──────────────┬──────────┬─────────┤
│   io/    │  index/  │    align/    │  util/   │ error/  │
│  FASTA   │   SA     │  Seed        │ DNA enc  │ BwaError│
│  FASTQ   │   BWT    │  Chain       │ RevComp  │ BwaResult│
│  SAM     │   FM     │  SW/Extend   │          │         │
│          │  Builder │  Candidate   │          │         │
│          │          │  MAPQ        │          │         │
│          │          │  Pipeline    │          │         │
└──────────┴──────────┴──────────────┴──────────┴─────────┘
```

## Modules

### `io/` — Input/Output

- **`fasta.rs`** — FASTA parser: multi-contig support, different line endings, non-standard character filtering
- **`fastq.rs`** — FASTQ parser: 4-line record parsing, seq/qual length validation
- **`sam.rs`** — SAM output: header writing, mapped/unmapped record generation (AS/XS/NM tags)

### `index/` — Index Construction

- **`sa.rs`** — Suffix Array: doubling algorithm O(n log²n), multi-sentinel support
- **`bwt.rs`** — BWT: derived directly from SA
- **`fm.rs`** — FM Index: C table + block Occ sampling, backward search, sparse SA sampling, bincode serialization
- **`builder.rs`** — One-step FM index construction from FASTA

### `align/` — Sequence Alignment

- **`seed.rs`** — SMEM seed finding: per-position longest exact match extension
- **`chain.rs`** — Seed chaining (DP) and filtering (greedy peeling + overlap removal)
- **`sw.rs`** — Banded affine-gap Smith-Waterman: reusable buffer, CIGAR and NM generation
- **`extend.rs`** — Chain to full alignment: left/right SW extension + CIGAR merging
- **`candidate.rs`** — Alignment candidate collection and deduplication
- **`mapq.rs`** — MAPQ estimation: based on primary/secondary score difference
- **`pipeline.rs`** — Batch parallel alignment pipeline (rayon) + SAM output

### `error/` — Error Types

- `BwaError` enum (Io / IndexFormat / IndexBuild / Align / Parse) + `BwaResult<T>` alias

### `util/` — Utilities

- **`dna.rs`** — Alphabet encoding `{0:$, 1:A, 2:C, 3:G, 4:T, 5:N}`, normalization, reverse complement

## Index File Format

The `.fm` file uses bincode serialization:

| Field | Type | Description |
|-------|------|-------------|
| magic | `u64` | `0x424D_4146_4D5F5253` ("BWAFM_RS") |
| version | `u32` | Format version (currently 2) |
| sigma | `u8` | Alphabet size (6) |
| block | `u32` | Occ sampling block size |
| c | `Vec<u32>` | C table |
| bwt | `Vec<u8>` | BWT sequence |
| occ_samples | `Vec<u32>` | Occ sampling table |
| sa | `Vec<u32>` | SA (full or sparse) |
| sa_sample_rate | `u32` | Sparse sampling interval |
| contigs | `Vec<Contig>` | Contig metadata |
| text | `Vec<u8>` | Original encoded text |
| meta | `Option<IndexMeta>` | Build metadata |

## Alignment Algorithm Flow

```
FASTQ read
    │
    ├─ Forward normalization ────────┐
    │                                │
    ├─ Reverse complement ──────────┐│
    │                               ││
    ▼                               ▼▼
  SMEM seed finding ──→ Chaining ──→ Chain filtering
                                        │
                                        ▼
                                Chain → Alignment (SW extension)
                                        │
                                        ▼
                             Candidate dedup + sorting
                                        │
                                        ▼
                          Primary + secondary output
                                        │
                                        ▼
                                    SAM record
```

## Differences from BWA/BWA-MEM

| Aspect | BWA (C) | bwa-rust |
|--------|---------|----------|
| Index format | `.bwt/.sa/.pac` multi-file | Single `.fm` file (bincode) |
| SA construction | DC3/IS algorithm | Doubling algorithm |
| MEM finding | `bwt_smem1` (bidirectional BWT) | Unidirectional backward_search |
| Chaining | Complex greedy+DP | Simplified DP |
| MAPQ | Complex statistical model | Simplified score-diff model |
| Parallelism | pthread | rayon |
| Paired-end | Supported | Single-end only |

## Tech Stack

- **Rust 2021 Edition** — Systems programming language
- **clap 4.5** — CLI argument parsing
- **serde + bincode** — Index serialization
- **rayon** — Data parallelism
- **chrono** — Timestamps
- **criterion** — Benchmarking
- **anyhow** — Error handling
