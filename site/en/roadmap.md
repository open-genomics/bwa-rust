# Roadmap

> **Current version: v0.1.0** — Single-end BWA-MEM style aligner, all planned tasks completed.

---

## Project Goals

Implement a **Rust aligner inspired by BWA**:

- Architecture and algorithms follow BWA/BWA-MEM
- **Not aiming for 100% behavioral compatibility**
- Prioritize correctness, readability, and memory safety

---

## v0.1.0 ✅ Completed

| Phase | Content | Status |
|-------|---------|:------:|
| **Baseline** | Goal definition, test datasets, dev scripts | ✅ |
| **Index** | FASTA parsing, FM index serialization, SA/BWT verification | ✅ |
| **Alignment MVP** | Config, seed + banded SW, CIGAR/NM output | ✅ |
| **BWA-MEM Style** | SMEM seeds, chaining, extension, MAPQ | ✅ |
| **Performance** | Benchmarks, multi-threading, sparse SA, buffer reuse | ✅ |
| **Docs** | Architecture docs, tutorial, examples, CI | ✅ |

### Post-v0.1.0 Improvements

| Improvement | Description | Status |
|-------------|-------------|:------:|
| Memory protection | `max_occ`, `max_chains`, `max_alignments` | ✅ |
| Alignment quality | Semi-global refinement, clip penalty ranking | ✅ |
| Input validation | FASTA error detection, parameter validation | ✅ |
| Code quality | Named constants, API doc comments | ✅ |

---

## Future Plans

| Version | Milestone | Key Features |
|---------|-----------|--------------|
| **v0.2.0** | Paired-end | PE reads, insert size estimation, mate rescue |
| **v0.3.0** | Index compat | Read BWA native index files |
| **v0.4.0** | Output | Direct BAM output, sorted output |
| **v0.5.0** | Performance | SIMD acceleration, mmap index |
| **v1.0.0** | Production | Human genome validation, stable API |

See [`docs/plan.md`](https://github.com/LessUp/bwa-rust/blob/main/docs/plan.md) for detailed design.

---

## Versioning

This project follows [Semantic Versioning](https://semver.org/): **MAJOR.MINOR.PATCH**

| Part | Meaning |
|------|---------|
| **MAJOR** | Breaking changes (API, index format) |
| **MINOR** | New features, backward compatible |
| **PATCH** | Bug fixes, performance tweaks |

### `0.x` Phase Rules

- API and index format **allow breaking changes**
- Breaking changes marked `BREAKING` in CHANGELOG
- `.fm` index `version` field used for compatibility checks

### Conditions for `1.0.0`

1. ✅ Human genome (hg38) level correctness validation passes
2. ✅ Mapping rate within reasonable range vs C BWA
3. ✅ Public API stable
4. ✅ Documentation and error handling reach production quality

---

## Index Format Version

| Version | Change |
|---------|--------|
| v1 | Initial format |
| v2 | Added `IndexMeta` build metadata |

- Software upgrades without index format changes → no rebuild needed
- Breaking index format changes → increment version, error on load

---

## Release Process

```bash
# Code quality checks
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features

# Release build
cargo build --release

# Create tag
git tag v0.x.x
git push origin v0.x.x
```

---

## Test Coverage

| Type | Count |
|------|-------|
| Unit tests | 151 |
| Integration tests | 11 |
| Module tests | 5 |
| Doc tests | 1 |
| **Total** | **168** |

CI Pipeline: GitHub Actions (fmt → clippy → test → release)
