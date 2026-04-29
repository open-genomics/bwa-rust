# Copilot Instructions

This repository is a Rust 2021 BWA-MEM-style single-end DNA aligner. Keep suggestions aligned with the actual shipped pipeline:

`FASTA/FASTQ -> FM-index -> SMEM seeds -> chains -> Smith-Waterman -> SAM`.

Project constraints:

- Do not suggest `unsafe` code.
- Do not add code comments unless requested or matching nearby style.
- Use `AlignOpt::default()` in `src/align/mod.rs` as the only default-parameter truth source.
- Treat `openspec/specs/` as the requirement source for behavior and governance.
- Keep paired-end and BAM/CRAM suggestions clearly marked as planned, not shipped.

Important modules:

- `src/index/`: suffix array, BWT, FM-index serialization.
- `src/align/`: seeding, chaining, SW extension, MAPQ, SAM candidate pipeline.
- `src/io/`: FASTA, FASTQ, SAM.
- `src/util/dna.rs`: DNA normalization and reverse complement.

Verification commands:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
npm run docs:build
```
