# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### ✨ Added

#### Memory Protection Configuration

New configurable limits to prevent memory explosion on repetitive sequences:

| Parameter | Default | Description |
|-----------|---------|-------------|
| `max_occ` | 500 | Skip seeds with more occurrences |
| `max_chains_per_contig` | 5 | Limit chains extracted per contig |
| `max_alignments_per_read` | 5 | Limit output alignments per read |

- Added `AlignOpt::validate()` method for comprehensive parameter validation

### 🐛 Fixed

#### Alignment Quality

- **Candidate Filtering**: Fixed premature threshold filtering before sorting forward/reverse candidates
  - Strong reverse hits are no longer incorrectly marked as unmapped
- **Semi-global Refinement**: Added refinement for chain candidates
  - Improves mismatch/indel CIGAR and NM tag accuracy
  - Insertion/deletion reads now output real `I/D` CIGAR instead of fake full-length `M`
- **Clip Penalty**: Introduced clip penalty for candidate ranking
  - Prevents free soft-clips from masking single-base indels

#### Input Validation

- FASTA header without sequence name now raises clear error
- Empty sequences rejected during index build
- Duplicate contig names rejected during index build
- `--threads 0` now errors at CLI level instead of silent fallback

#### Error Handling

- Replaced double `unwrap` with proper error propagation in rayon thread pool
- Thread pool construction failures now return clear error messages

### 🔧 Changed

#### Code Quality

- Extracted named constants, eliminated magic numbers:
  - `MAX_ALIGNMENTS_PER_READ`, `MAX_CHAINS_PER_CONTIG`, `EXTEND_REF_PAD`, `DEFAULT_MAX_OCC`
- Added `Copy` trait to `MemSeed`, removed unnecessary clones
- Replaced `.cloned()` with `.copied()` for `Copy` types
- Removed `from_utf8_lossy(...).into_owned()` in hot paths

#### Documentation

- Added comprehensive doc comments to public APIs:
  - `chain.rs`, `candidate.rs`, `extend.rs`, `sw.rs`, `seed.rs`, `dna.rs`
- Added `parse_cigar` boundary behavior tests

### ⚡ Performance

- Optimized read/qual and reverse-complement string construction paths

---

## [0.1.0] - 2026-02-13

### ✨ Added

#### Index Building (`index` subcommand)

| Feature | Description |
|---------|-------------|
| FASTA Parser | Multi-contig support, various line endings, non-standard character filtering |
| Suffix Array | Doubling algorithm, O(n log²n) complexity |
| BWT Construction | Built from suffix array |
| FM Index | C table + block-based Occ sampling + sparse SA sampling |
| Serialization | Single `.fm` file with magic number, version, and build metadata |

#### Sequence Alignment (`align` subcommand)

| Feature | Description |
|---------|-------------|
| SMEM Seeding | Super-Maximal Exact Match seed finding |
| Seed Chaining | DP-based chain scoring + greedy peeling + filtering |
| Smith-Waterman | Banded affine-gap local alignment with CIGAR generation |
| Bidirectional | Forward and reverse-complement alignment |
| Candidate Management | Multi-chain deduplication, primary/secondary output |
| MAPQ Estimation | Based on primary/secondary score difference |
| SAM Output | Full header, CIGAR, AS/XS/NM tags |
| Multi-threading | Parallel processing via rayon (`--threads`) |

#### One-Step Alignment (`mem` subcommand)

- BWA-MEM style one-command workflow
- BWA-MEM default scoring: match=1, mismatch=4, gap-open=6, gap-ext=1

#### Engineering

- **Benchmarks**: Criterion performance tests
- **CI/CD**: GitHub Actions (fmt → clippy → test → release build)
- **Documentation**: Architecture docs, tutorial, example code
- **Test Coverage**: 167 tests total (151 unit + 11 integration + 5 module tests)

---

## Version History

| Version | Date | Highlights |
|---------|------|------------|
| Unreleased | — | Memory protection, alignment quality fixes, code quality improvements |
| 0.1.0 | 2026-02-13 | Initial release: FM index, SMEM seeding, banded SW, SAM output, multi-threading |

---

## Migration Guide

### From v0.1.0 to Unreleased

**No breaking changes**. All new features are backward compatible:

- New `AlignOpt` fields have sensible defaults
- Existing CLI commands work unchanged
- `.fm` index format unchanged (version 2)

### New CLI Options (Unreleased)

```bash
# Limit repetitive seeds (default: 500)
bwa-rust align -i ref.fm reads.fq --max-occ 200

# Limit chains per contig (default: 5)
bwa-rust mem ref.fa reads.fq --max-chains 3

# Limit output alignments per read (default: 5)
bwa-rust mem ref.fa reads.fq --max-alignments 10
```

---

## Release Checklist

Before each release, ensure:

- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] `cargo test --all-targets --all-features` passes
- [ ] `cargo build --release` succeeds
- [ ] CHANGELOG.md updated with release date
- [ ] Git tag created: `v{version}`
