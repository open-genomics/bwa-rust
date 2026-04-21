---
layout: doc
---

# Changelog

All notable changes to this project will be documented in this file.

## [v0.3.0] - 🚧 In Development

### Added

- **Paired-End Alignment Support** (WIP)
  - Paired FASTQ parsing (separate files and interleaved format)
  - Insert size estimation with median and MAD
  - Automatic threshold update
  - PE FASTQ validation and error checking

### Specifications

- Product specification for PE alignment (F-011 to F-017)
- RFC-0004: Paired-End Alignment Technical Design
- Detailed implementation plan (5 phases)

## [v0.2.0] - 2026-04-17

### Documentation

- **Bilingual Documentation Suite**
  - Complete documentation overhaul with full Chinese and English support
  - Architecture documentation (overview, index building, alignment)
  - Tutorial documentation (getting started, algorithm tutorial)
  - API documentation (library usage guide)

### Added

- **Memory Protection Configuration**
  - `max_occ` (default: 500) - Skip seeds with more occurrences
  - `max_chains_per_contig` (default: 5) - Limit chains per contig
  - `max_alignments_per_read` (default: 5) - Limit alignments per read
  - `AlignOpt::validate()` method for parameter validation

### Fixed

- **Alignment Quality**
  - Fixed candidate filtering for strong reverse hits
  - Added semi-global refinement for better CIGAR/NM accuracy
  - Introduced clip penalty for candidate ranking

- **Input Validation**
  - FASTA header validation
  - Empty sequence detection
  - Duplicate contig name rejection
  - Thread count validation

### Changed

- **Code Quality**
  - Extracted named constants, eliminated magic numbers
  - Added comprehensive doc comments to public APIs
  - Optimized hot-path allocations

## [v0.1.0] - 2026-02-13

### Added

- **Index Building** (`index` subcommand)
  - FASTA parser with multi-contig support
  - Suffix array construction (Doubling algorithm)
  - BWT construction
  - FM-index with sparse SA sampling
  - Single `.fm` index file

- **Sequence Alignment** (`align` subcommand)
  - SMEM seed finding
  - Seed chain building with DP scoring
  - Banded Smith-Waterman alignment
  - Multi-threading via rayon
  - SAM output with full headers

- **One-Step Alignment** (`mem` subcommand)
  - BWA-MEM style workflow
  - Default BWA-MEM scoring parameters

- **Engineering**
  - 175 tests (unit + integration)
  - GitHub Actions CI/CD
  - Criterion benchmarks
  - Documentation site

---

[View all releases on GitHub](https://github.com/LessUp/bwa-rust/releases)
