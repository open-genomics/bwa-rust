# Validation Boundaries

## Overview

This page documents how bwa-rust verifies correctness and the boundaries of that verification.

## Test Coverage

### Unit Tests

Each module has co-located tests:

```
src/
├── align/
│   ├── mod.rs
│   ├── sw.rs
│   └── tests.rs  ← Module tests
├── index/
│   ├── fm.rs
│   └── tests.rs
└── ...
```

### Integration Tests

Located in `tests/`:

- `integration_test.rs`: End-to-end alignment tests
- `sam_output_test.rs`: SAM format validation

### Test Data

Reference test data in `tests/data/`:

- `tiny.fasta`: Minimal reference for quick tests
- `ecoli_subset.fasta`: E. coli subset for integration tests
- `reads.fastq`: Sample reads

## CI Pipeline

### Continuous Integration

```yaml
# .github/workflows/ci.yml
jobs:
  test:
    - cargo test --all-targets
    - cargo clippy -- -D warnings
    - cargo fmt -- --check
```

### Coverage Reporting

Coverage is tracked but not enforced as a hard threshold:

- Current coverage: ~80% (approximate)
- Focus: Critical paths (alignment, SAM output)

## Verification Methods

### 1. Format Validation

SAM output is validated against the [SAM specification](https://samtools.github.io/hts-specs/SAMv1.pdf):

- CIGAR string validity
- MAPQ range [0, 255]
- Tag format (AS:i, XS:i, NM:i, MD:Z, SA:Z)

### 2. Round-Trip Testing

```rust
// Index round-trip
let index = FmIndex::build(&reference);
let bytes = index.serialize();
let restored = FmIndex::deserialize(&bytes)?;
assert_eq!(index, restored);
```

### 3. Property Testing

Using `proptest` for:

- DNA encoding/decoding
- CIGAR string operations
- Coordinate transformations

### 4. Known-Answer Testing

Compare against known results for:

- Small references (manually computed)
- E. coli subset (validated against external tools)

## What Is NOT Verified

### Not Bit-Level BWA Compatible

We do **not** verify output matches BWA exactly:

- Different floating-point decisions
- Different tie-breaking rules
- Different heuristic thresholds

### Not Production-Scale Tested

Testing focuses on correctness, not scale:

- Human genome not in CI (too large)
- Million-read datasets not in CI (too slow)

### Not Performance Regression Tested

No automated performance regression tests:

- Benchmarks are manual
- Performance varies by hardware

## Manual Verification

For production use, verify:

```bash
# Build index
bwa-rust index reference.fasta -o index.fm

# Align test reads
bwa-rust align index.fm test.fastq -o output.sam

# Validate SAM
samtools view -Sb output.sam > /dev/null

# Compare with BWA (optional)
bwa mem reference.fasta test.fastq > bwa.sam
# Compare key metrics: alignment rate, MAPQ distribution
```

---

[Next: Limitations →](/en/specs/limitations)
