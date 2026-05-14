---
layout: home

hero:
  name: 'bwa-rust'
  text: 'Memory-safe BWA-MEM style single-end aligner'
  tagline: 'Rust 2021 implementation: FASTA/FASTQ to FM-index, SMEM, chaining, Smith-Waterman, and SAM.'
  actions:
    - theme: brand
      text: 'Quick Start'
      link: '/en/guide/quickstart'
    - theme: alt
      text: 'Architecture'
      link: '/en/architecture/'
    - theme: alt
      text: 'GitHub'
      link: 'https://github.com/LessUp/bwa-rust'

features:
  - title: 'Single-file FM-index'
    details: 'Suffix array, BWT, Occ samples, and contig metadata unified in .fm format—easier to move and archive than BWA multi-file indices.'
  - title: 'Clear seed-chain-extend pipeline'
    details: 'SMEM seeding, DP chain building, banded Smith-Waterman extension, and SAM output mapped to clean modules—ideal for learning and extension.'
  - title: 'Zero unsafe code'
    details: 'unsafe_code = forbid enforced by Cargo lint, for Rust bioinformatics experiments requiring memory safety boundaries.'
  - title: 'Honest scope'
    details: 'Single-end FASTQ to SAM delivered; paired-end and BAM/CRAM are planned features, not claimed as production-ready.'
---

## Who Should Use This

| User | Value |
|------|-------|
| Rust bioinformatics developers | Reuse FM-index, SMEM, chaining, SW, and SAM components directly. |
| Algorithm learners | Read BWA-MEM style core flow in Rust instead of starting from a large C codebase. |
| Single-end read experiments | Need a configurable, testable, easy-to-archive single-end alignment baseline. |
| Security-sensitive prototypes | Require a DNA alignment experimental environment with `unsafe` forbidden. |

## Capability Matrix

| Capability | Status | Notes |
|------------|--------|-------|
| FASTA reference input | Delivered | Multi-contig supported. |
| FASTQ single-end reads | Delivered | Current stable data path. |
| `.fm` index | Delivered | Single-file bincode format with magic/version validation. |
| SMEM + chaining + SW | Delivered | BWA-MEM style, not pursuing bit-level compatibility. |
| SAM output | Delivered | CIGAR, MAPQ, AS/XS/NM, MD:Z, SA:Z. |
| Rayon parallelism | Delivered | Read-level parallelism. |
| Paired-end | Planned | Design and partial infrastructure reserved, CLI not exposed. |
| BAM/CRAM | Planned | Currently outputs SAM only. |

## Not Suitable For

- Production-grade paired-end workflows.
- Compatibility testing against BWA output.
- Mature production scheduling at human genome scale.
- Pipelines requiring native BAM/CRAM output.

## Entry Points

- [Installation & Quick Start](/en/guide/)
- [Core Architecture](/en/architecture/)
- [Benchmarks & Verification](/en/benchmarks)
- [FAQ](/en/faq)
