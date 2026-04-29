# Changelog

All notable changes to bwa-rust are recorded here.

## [Unreleased]

### Fixed

- Threaded configured `zdrop` through chain extension instead of using a hard-coded extension threshold.
- Preserved correct query coordinate space for MD:Z generation on soft-clipped alignments.
- Allowed SA:Z tags to be emitted even when MD:Z data is unavailable for a candidate.
- Normalized `mem` subcommand defaults to match `AlignOpt::default()`.

### Changed

- Rebuilt README and GitHub Pages around a single shipped/planned capability matrix.
- Reduced `docs/` to internal development/tooling guidance; public user docs now live under `site/`.
- Simplified GitHub Actions to least-privilege CI, Pages, release, and audit workflows.
- Removed disabled/noisy Dependabot, scheduled benchmark, link-check issue creation, and unused PWA/analytics/sitemap scaffolding.
- Rewrote AI guidance for OpenCode, Claude, and Copilot to be short and project-specific.

## [0.2.0] - 2026-04-17

### Added

- Configurable memory-protection knobs: `max_occ`, `max_chains_per_contig`, and `max_alignments_per_read`.
- CLI flags `--max-occ`, `--max-chains`, and `--max-alignments` for `align` and `mem`.
- Real-data test scaffold behind the optional `real-data` feature.

### Fixed

- Strong reverse-complement candidates are sorted before score threshold filtering.
- Semi-global refinement improves mismatch/indel CIGAR and NM accuracy.
- Single-base insertion/deletion cases emit real `I`/`D` CIGAR operations.
- FASTA/FASTQ validation reports clearer errors for malformed input.
- Thread-pool construction errors are propagated instead of unwrapped.

### Changed

- Added clip-penalty-aware candidate ranking.
- Improved hot-path allocation behavior for read/quality and reverse-complement output.
- Expanded tests to 188 unit tests and 11 integration tests.

## [0.1.0] - 2026-02-13

### Added

- FASTA parsing and FM-index construction.
- Suffix array, BWT, C table, Occ sampling, sparse SA support, and `.fm` serialization.
- FASTQ single-end alignment through SMEM seeding, chain construction, banded Smith-Waterman, MAPQ, and SAM output.
- `index`, `align`, and `mem` CLI commands.
- Rayon read-level parallelism.
- Criterion benchmarks and GitHub Actions CI.

[unreleased]: https://github.com/LessUp/bwa-rust/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/LessUp/bwa-rust/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/LessUp/bwa-rust/releases/tag/v0.1.0
