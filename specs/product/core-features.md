# Product Specification: Core Features

> **Version**: v0.2.0
> **Status**: Released
> **Last Updated**: 2026-04-16

## Overview

bwa-rust is a BWA-MEM style DNA short-read aligner implemented in Rust. This document defines the core product features and acceptance criteria.

## Feature List

### F-001: FM-Index Based Sequence Alignment

**Description**: Implement FM-index based exact match finding and approximate alignment for DNA short reads.

**Acceptance Criteria**:
- [ ] Build FM-index from FASTA reference genome
- [ ] Support single-end read alignment from FASTQ input
- [ ] Output SAM format alignment results
- [ ] Handle multi-contig reference genomes
- [ ] Support reverse complement alignment

**Technical Constraints**:
- Alphabet size: 6 ($, A, C, G, T, N)
- Index file format: Single `.fm` file with magic header `BWAFM_RS`
- Index file version: 2

### F-002: SMEM Seed Finding

**Description**: Find Super-Maximal Exact Matches (SMEMs) for seeding alignment.

**Acceptance Criteria**:
- [ ] Find longest exact match covering each read position
- [ ] Support left extension until no longer maximal
- [ ] Filter seeds with occurrence count exceeding `max_occ` threshold
- [ ] Default `max_occ` value: 500

### F-003: Seed Chain Building

**Description**: Combine multiple seeds into coherent chains using dynamic programming.

**Acceptance Criteria**:
- [ ] DP-based chain scoring with gap penalties
- [ ] Greedy peeling for multi-chain extraction
- [ ] Filter low-score chains below threshold
- [ ] Limit chains per contig via `max_chains_per_contig` (default: 5)

### F-004: Banded Smith-Waterman Alignment

**Description**: Perform banded affine-gap local alignment for chain extension.

**Acceptance Criteria**:
- [ ] Banded SW with configurable band width (default: 16)
- [ ] Affine gap penalty (gap open + gap extend)
- [ ] Generate CIGAR string from alignment
- [ ] Compute NM (edit distance) tag
- [ ] Semi-global refinement for edge cases

**Default Scoring Parameters**:
| Parameter | Value |
|-----------|-------|
| Match score | 2 |
| Mismatch penalty | 1 |
| Gap open | 2 |
| Gap extend | 1 |

### F-005: SAM Output

**Description**: Format and output alignment results in SAM format.

**Acceptance Criteria**:
- [ ] Generate `@HD` header line (VN:1.6, SO:unsorted)
- [ ] Generate `@SQ` header lines for each contig
- [ ] Generate `@PG` header line with program info
- [ ] Format alignment records with correct FLAG values
- [ ] Include optional tags: AS:i (alignment score), XS:i (suboptimal score), NM:i (edit distance)

**FLAG Settings**:
- Forward alignment: FLAG = 0
- Reverse complement alignment: FLAG = 16
- Secondary alignment: FLAG |= 256

### F-006: Multi-Threading Support

**Description**: Support parallel alignment processing using rayon.

**Acceptance Criteria**:
- [ ] Configurable thread count via CLI
- [ ] Near-linear speedup on multi-core systems
- [ ] Thread-safe data structures

### F-007: Memory Protection

**Description**: Prevent memory explosion from repetitive sequences.

**Acceptance Criteria**:
- [ ] `max_occ`: Skip seeds with SA interval > threshold (default: 500)
- [ ] `max_chains_per_contig`: Limit chains per contig (default: 5)
- [ ] `max_alignments_per_read`: Limit output alignments per read (default: 5)

### F-008: MAPQ Estimation

**Description**: Compute Mapping Quality (MAPQ) scores for alignments.

**Acceptance Criteria**:
- [ ] Score-difference based MAPQ model
- [ ] Consider best vs second-best alignment score difference
- [ ] Output MAPQ in SAM record (column 5)

### F-009: CLI Interface

**Description**: Provide command-line interface with clap.

**Acceptance Criteria**:
- [ ] `index` subcommand: Build FM-index from FASTA
- [ ] `align` subcommand: Align FASTQ reads to existing index
- [ ] `mem` subcommand: One-step index + align (BWA-MEM style)
- [ ] Support `-o` output prefix option for index
- [ ] Support `-t` thread count option

### F-010: Input Validation

**Description**: Validate all inputs with clear error messages.

**Acceptance Criteria**:
- [ ] Reject empty FASTA sequences
- [ ] Reject duplicate contig names
- [ ] Handle malformed FASTQ records
- [ ] Validate thread count (must be > 0)
- [ ] Handle various line endings (LF/CRLF)

## Planned Features (Future Versions)

### F-011: Paired-End Alignment (v0.2.0+)

**Description**: Support paired-end read alignment with insert size constraints.

### F-012: BAM Output (v0.4.0+)

**Description**: Support compressed BAM format output.

### F-013: BWA Native Index Compatibility (v0.3.0+)

**Description**: Read BWA's `.bwt`/`.sa`/`.pac` index files directly.

## Non-Functional Requirements

### NFR-001: Memory Safety

- Zero `unsafe` code in the entire codebase
- All memory safety guaranteed by Rust compiler

### NFR-002: Performance

| Metric | Target |
|--------|--------|
| Index build (100M bp) | < 60s |
| Alignment (1K reads, 4 threads) | < 1s |
| Memory usage (human genome) | < 8 GB |

### NFR-003: Code Quality

- Pass `cargo fmt --all -- --check`
- Pass `cargo clippy --all-targets --all-features -- -D warnings`
- Pass `cargo test --all-targets --all-features`
- MSRV: Rust 1.70

### NFR-004: Platform Support

- Linux (primary)
- macOS
- Windows
