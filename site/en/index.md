---
layout: home
---

<div class="whitepaper-header">
  <div class="whitepaper-title">bwa-rust</div>
  <div class="whitepaper-subtitle">Memory-safe BWA-MEM style single-end DNA short-read aligner</div>
</div>

<div class="whitepaper-intro">
A Rust 2021 implementation inspired by BWA-MEM, featuring single-file FM-index, zero unsafe code, and a clear seed-chain-extend pipeline. Designed for learning, extension, and security-sensitive bioinformatics experiments.
</div>

## Core Value Proposition

<div class="value-grid">
  <div class="value-item">
    <div class="value-item-title">📦 Single-file FM-index</div>
    <div class="value-item-desc">Suffix array, BWT, Occ samples, and contig metadata unified in <code>.fm</code> format—easier to move and archive than BWA multi-file indices.</div>
  </div>
  <div class="value-item">
    <div class="value-item-title">🔒 Zero unsafe code</div>
    <div class="value-item-desc"><code>unsafe_code = "forbid"</code> enforced by Cargo lint, providing memory safety boundaries for Rust bioinformatics experiments.</div>
  </div>
  <div class="value-item">
    <div class="value-item-title">🧬 Clear pipeline</div>
    <div class="value-item-desc">SMEM seeding, DP chain building, banded Smith-Waterman extension, and SAM output mapped to clean modules—ideal for learning and extension.</div>
  </div>
  <div class="value-item">
    <div class="value-item-title">🎯 Honest scope</div>
    <div class="value-item-desc">Single-end FASTQ to SAM delivered; paired-end and BAM/CRAM are planned features, not claimed as production-ready.</div>
  </div>
</div>

## Capability Matrix

| Capability | Status | Notes |
|------------|:------:|-------|
| FASTA reference input | <span class="status-badge delivered">✓ Delivered</span> | Multi-contig supported. |
| FASTQ single-end reads | <span class="status-badge delivered">✓ Delivered</span> | Current stable data path. |
| `.fm` index | <span class="status-badge delivered">✓ Delivered</span> | Single-file bincode format with magic/version validation. |
| SMEM + chaining + SW | <span class="status-badge delivered">✓ Delivered</span> | BWA-MEM style, not pursuing bit-level compatibility. |
| SAM output | <span class="status-badge delivered">✓ Delivered</span> | CIGAR, MAPQ, AS/XS/NM, MD:Z, SA:Z. |
| Rayon parallelism | <span class="status-badge delivered">✓ Delivered</span> | Read-level parallelism. |
| Paired-end alignment | <span class="status-badge planned">📋 Planned</span> | Design and partial infrastructure reserved, CLI not exposed. |
| BAM/CRAM output | <span class="status-badge planned">📋 Planned</span> | Currently outputs SAM only. |

## Who Should Use This

| User Profile | Value Proposition |
|--------------|-------------------|
| Rust bioinformatics developers | Reuse FM-index, SMEM, chaining, SW, and SAM components directly. |
| Algorithm learners | Read BWA-MEM style core flow in Rust instead of starting from a large C codebase. |
| Single-end read experiments | Need a configurable, testable, easy-to-archive single-end alignment baseline. |
| Security-sensitive prototypes | Require a DNA alignment experimental environment with `unsafe` forbidden. |

## Not Suitable For

- Production-grade paired-end workflows.
- Compatibility testing against BWA output.
- Mature production scheduling at human genome scale.
- Pipelines requiring native BAM/CRAM output.

<hr class="section-divider" />

## Quick Start

<div class="quick-start-block">

### Installation

```bash
cargo install bwa-rust
```

### Build Index

```bash
bwa-rust index reference.fasta -o reference.fm
```

### Align Reads

```bash
bwa-rust align reference.fm reads.fastq -o output.sam
```

</div>

## Entry Points

- [Installation & Quick Start](/en/guide/) — Get up and running
- [Core Architecture](/en/architecture/) — Understand the design
- [Benchmarks & Verification](/en/benchmarks) — Performance data
- [FAQ](/en/faq) — Common questions

<div style="text-align: center; margin-top: 2rem; color: var(--vp-c-text-3); font-size: 13px;">
  <a href="https://docs.rs/bwa-rust" target="_blank">API Reference (docs.rs)</a> ·
  <a href="https://github.com/LessUp/bwa-rust" target="_blank">GitHub Repository</a>
</div>
