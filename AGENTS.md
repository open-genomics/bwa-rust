# AGENTS.md

## Project Truth

bwa-rust is a Rust 2021 library + CLI for BWA-MEM-style single-end DNA short-read alignment.

Pipeline: `FASTA/FASTQ -> FM-index -> SMEM seeds -> chains -> Smith-Waterman -> SAM`.

Shipped scope:

- FASTA reference input, FASTQ single-end reads, SAM output.
- Single `.fm` index file built from suffix array + BWT + sampled Occ table.
- SMEM seeding, seed chaining, banded SW extension, MAPQ, MD:Z and SA:Z tags.
- Rayon read-level parallelism.

Not shipped: paired-end CLI alignment, BAM/CRAM output, BWA index compatibility, exact BWA output compatibility.

## Source Of Truth

- Requirements: `openspec/specs/`.
- Active changes: `openspec/changes/<name>/`.
- CLI/library defaults: `src/align/mod.rs` `AlignOpt::default()`.
- Public story: `README.md`, `README.zh-CN.md`, `site/`.

Use OpenSpec before new features, public API changes, index format changes, complex refactors, CI/governance changes, or major docs/site rewrites.

## Code Rules

- `unsafe_code = "forbid"`; do not introduce `unsafe`.
- No code comments unless explicitly requested or preserving local style.
- Keep line width within `rustfmt.toml`.
- Keep fixes small and behavior-focused; do not add compatibility layers without a concrete persisted/external need.
- Bug fixes require a failing regression test first.

## Module Map

- `src/main.rs`: clap CLI and command dispatch.
- `src/index/`: suffix array, BWT, FM-index build/load/save.
- `src/align/seed.rs`: SMEM/MEM seed discovery and occurrence filtering.
- `src/align/chain.rs`: chain scoring, filtering, multi-chain extraction.
- `src/align/sw.rs`: banded SW, global/semi-global alignment, extension primitives.
- `src/align/extend.rs`: chain-to-CIGAR extension and z-drop plumbing.
- `src/align/candidate.rs`: candidate collection, refinement, dedup/ranking fields.
- `src/align/pipeline.rs`: per-read alignment, MAPQ, primary/secondary/supplementary output.
- `src/io/`: FASTA/FASTQ/SAM parsing and formatting.
- `src/util/dna.rs`: normalization, alphabet mapping, reverse complement.

## Verification

After code or workflow/doc-site changes run the relevant subset, and before completion run all:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
npm run docs:build
```

Use `cargo bench` only for performance-sensitive changes.

## Git Flow

- Start by checking `git status --short --branch` and relevant worktrees.
- Direct push to `master` is the default after verification in this single-maintainer repo.
- Use temporary branches/worktrees only for risky or parallel work; remove them after landing.
- Never revert or delete unrelated user changes.

## Review Triggers

Request review for alignment correctness, index serialization, SAM tag behavior, workflow permissions, release changes, or large repository deletions.
