# Roadmap

This roadmap is intentionally conservative. Shipped capabilities are documented in README and OpenSpec; planned items below are not promised CLI behavior until implemented and verified.

## Current Baseline: v0.2.x

Shipped:

- FASTA reference parsing and single `.fm` FM-index construction.
- FASTQ single-end read alignment.
- SMEM seed discovery with `max_occ` filtering.
- Seed chaining with bounded chain/alignment output.
- Banded Smith-Waterman extension with configurable z-drop.
- SAM output with CIGAR, MAPQ, AS/XS/NM, MD:Z, and SA:Z where available.
- Rayon read-level parallelism.
- OpenSpec-driven development and minimal closeout-oriented CI.

Known limits:

- No paired-end CLI pipeline.
- No BAM/CRAM output.
- No BWA index compatibility.
- No guarantee of exact BWA output equivalence.
- Human-genome-scale production validation remains future work.

## Planned Work

### v0.3.0: Paired-End Alignment

Objective: connect existing paired FASTQ and insert-size groundwork to the public pipeline.

Validation gate:

- paired FASTQ parsing tests;
- proper pair flag tests;
- insert-size behavior tests;
- single-end behavior unchanged.

### v0.4.0: Larger-Reference Robustness

Objective: reduce memory pressure and improve index/query behavior on larger references.

Candidates:

- more compact reference storage;
- better SA sampling trade-offs;
- benchmarked memory profiles on larger bacterial and mammalian references.

Validation gate:

- reproducible benchmark notes;
- no regression in current unit/integration tests;
- documented memory envelope.

### v0.5.0: Output Enhancements

Objective: add optional output formats only after SAM behavior remains stable.

Candidates:

- BAM output;
- coordinate sorting;
- richer optional tags.

Validation gate:

- format conformance tests;
- round-trip checks with common tooling where feasible.

### v1.0.0 Readiness

Conditions:

- stable public API;
- clear index-format compatibility policy;
- real-data validation documented;
- production-grade error messages and user docs;
- performance claims backed by reproducible methodology.

## Release Checklist

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
npm run docs:build
```

Push release tags to `master`:

```bash
git tag v0.x.y
git push origin master --tags
```
