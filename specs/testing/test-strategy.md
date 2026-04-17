# Testing Specification

> **Version**: v0.2.0  
> **Last Updated**: 2026-04-16

## Overview

This document defines the testing strategy and acceptance criteria for bwa-rust.

## Test Categories

### 1. Unit Tests

Location: `#[cfg(test)] mod tests` within each module.

**Coverage Requirements**:
- All public functions must have unit tests
- Boundary conditions must be tested
- Error paths must be tested

**Examples**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_sa_empty_input() {
        let sa = build_sa(&[]);
        assert!(sa.is_empty());
    }

    #[test]
    fn test_backward_search_exact_match() {
        // ...
    }
}
```

### 2. Integration Tests

Location: `tests/integration.rs`

**Coverage Requirements**:
- End-to-end pipeline workflows
- Cross-module interactions
- Real file I/O operations

**Test Cases**:
- Build index from FASTA and verify `.fm` file creation
- Align single-end reads and verify SAM output
- Full `mem` command end-to-end test

### 3. Edge Case Tests

**Required Coverage**:

| Category | Test Cases |
|----------|-----------|
| Empty input | Empty FASTA, empty FASTQ, empty reference |
| Invalid format | Malformed FASTQ, invalid FASTA headers |
| Boundary coordinates | Position 0, position n-1, single-base sequences |
| Reverse complement | Palindromic sequences, full reverse complement |
| Duplicate names | Duplicate contig names in FASTA |
| Score thresholds | Zero-score alignments, exact-score alignments |

### 4. Property-Based Tests

Where applicable, use property-based testing to verify algorithm invariants:

**Examples**:
- `reverse_complement(reverse_complement(seq)) == seq`
- `FMIndex backward search returns valid SA interval`
- `CIGAR string length matches alignment length`

## Test Execution

### Run All Tests

```bash
cargo test --all-targets --all-features
```

### Run Library Tests Only

```bash
cargo test --lib
```

### Run Integration Tests Only

```bash
cargo test --test integration
```

### Run Single Test

```bash
# By substring match
cargo test error_display

# Exact match
cargo test error_display -- --exact

# Library test exact match
cargo test --lib error_display -- --exact

# Integration test exact match
cargo test --test integration e2e_build_index_and_exact_search -- --exact

# With output
cargo test align_single_read_unmapped -- --exact --nocapture

# List all tests
cargo test -- --list
```

## Test Count (v0.2.0)

| Category | Count |
|----------|-------|
| Unit tests | 151 |
| Integration tests | 11 |
| Module tests | 5 |
| Doc tests | 1 |
| **Total** | **168** |

## Benchmark Tests

Location: `benches/benchmark.rs`

Run benchmarks:
```bash
cargo bench
```

**Benchmark Categories**:
- Index build performance
- Alignment performance (single-thread vs multi-thread)
- Memory usage

## CI Test Requirements

All PRs must pass:
1. `cargo fmt --all -- --check`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo test --all-targets --all-features`
4. `cargo build --release`

## Testing Anti-Patterns (Avoid)

```rust
// ❌ Don't use loose boolean assertions
assert!(result.is_ok());  // Too vague

// ✅ Use precise assertions
assert!(matches!(result, Ok(_)));
assert_eq!(result.unwrap().len(), expected_len);

// ❌ Don't skip error cases
// ✅ Test both success and failure paths
#[test]
fn test_parse_empty_fasta() {
    let result = parse_fasta("tests/data/empty.fa");
    assert!(result.is_err());
}
```
