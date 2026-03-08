# Roadmap

> **Current version: v0.1.0** — Single-end BWA-MEM style aligner, all planned tasks completed.

## v0.1.0 Completed

| Phase | Content | Status |
|-------|---------|--------|
| **0. Baseline** | Goal definition, test datasets, dev scripts | ✅ |
| **1. Index stabilization** | FASTA parsing, FM index serialization, SA/BWT correctness | ✅ |
| **2. Alignment MVP** | Align config, seed + banded SW, CIGAR/NM output | ✅ |
| **3. BWA-MEM style** | SMEM seeds, chaining, chain-to-alignment, MAPQ | ✅ |
| **4. Performance** | Benchmarks, multi-threading, sparse SA, buffer reuse | ✅ |
| **5. Docs & maintenance** | Architecture docs, tutorial, examples, CI | ✅ |

## Future Plans

| Version | Milestone | Key Features |
|---------|-----------|-------------|
| **0.2.0** | Paired-end | PE reads, insert size estimation, mate rescue |
| **0.3.0** | Index compat | Read BWA native index files |
| **0.4.0** | Output | Direct BAM output, sorted output |
| **0.5.0** | Performance | Smith-Waterman SIMD acceleration, mmap index |
| **1.0.0** | Production | Human genome validation, stable API |

## Versioning

This project follows [Semantic Versioning](https://semver.org/): **MAJOR.MINOR.PATCH**

- **MAJOR**: API stability commitment; breaking changes
- **MINOR**: New features, backward compatible
- **PATCH**: Bug fixes, performance tweaks, documentation
