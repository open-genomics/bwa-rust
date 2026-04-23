# Testing Specification

## Purpose

Define testing strategy and acceptance criteria for bwa-rust to ensure correctness and maintain code quality.

## Requirements

### Requirement: Unit Test Coverage

The system SHALL maintain comprehensive unit tests for all public functions.

#### Scenario: Test public functions

- **WHEN** a function is public
- **THEN** unit tests SHALL cover normal cases
- **AND** unit tests SHALL cover edge cases
- **AND** unit tests SHALL cover error paths

#### Scenario: Test in module context

- **WHEN** writing unit tests
- **THEN** place tests in `#[cfg(test)] mod tests` within module
- **AND** maintain minimum 188 unit test count

### Requirement: Integration Tests

The system SHALL provide integration tests for end-to-end workflows.

#### Scenario: Test index building

- **WHEN** running integration tests
- **THEN** test building index from FASTA
- **AND** verify `.fm` file creation and format

#### Scenario: Test alignment pipeline

- **WHEN** running integration tests
- **THEN** test full alignment workflow
- **AND** verify SAM output format and content

#### Scenario: Place in tests directory

- **WHEN** writing integration tests
- **THEN** place in `tests/integration.rs`
- **AND** maintain 11+ integration test count

### Requirement: Edge Case Testing

The system SHALL test edge cases and boundary conditions.

#### Scenario: Test empty inputs

- **WHEN** empty FASTA or FASTQ is provided
- **THEN** handle gracefully with appropriate error
- **AND** do not panic

#### Scenario: Test boundary coordinates

- **WHEN** alignment is at position 0 or n-1
- **THEN** process correctly without overflow
- **AND** output valid SAM

#### Scenario: Test reverse complement

- **WHEN** read aligns to reverse strand
- **THEN** produce correct reverse complement
- **AND** set FLAG bit 4 appropriately

### Requirement: Code Quality Gates

The system SHALL enforce code quality via CI.

#### Scenario: Check formatting

- **WHEN** CI runs
- **THEN** execute `cargo fmt --all -- --check`
- **AND** fail on any formatting issues

#### Scenario: Check lints

- **WHEN** CI runs
- **THEN** execute `cargo clippy --all-targets --all-features -- -D warnings`
- **AND** fail on any clippy warnings

#### Scenario: Run all tests

- **WHEN** CI runs
- **THEN** execute `cargo test --all-targets --all-features`
- **AND** fail on any test failures

### Requirement: Benchmark Tests

The system SHALL provide performance benchmarks.

#### Scenario: Benchmark index building

- **WHEN** running benchmarks
- **THEN** measure index construction time
- **AND** track against baseline

#### Scenario: Benchmark alignment

- **WHEN** running benchmarks
- **THEN** measure alignment throughput
- **AND** compare single-thread vs multi-thread

### Requirement: Test Anti-Patterns Avoided

The system SHALL avoid common testing anti-patterns.

#### Scenario: Use precise assertions

- **WHEN** writing test assertions
- **THEN** use specific matchers and equality checks
- **AND** avoid vague `assert!(result.is_ok())`

#### Scenario: Test error paths

- **WHEN** a function can fail
- **THEN** test the error case explicitly
- **AND** verify error message content

## Test Execution Commands

### Run all tests
```bash
cargo test --all-targets --all-features
```

### Run specific test categories
```bash
# Library tests only
cargo test --lib

# Integration tests only
cargo test --test integration

# Single test by name
cargo test error_display

# Single test exact match
cargo test error_display -- --exact

# With output
cargo test align_single_read_unmapped -- --exact --nocapture

# List all tests
cargo test -- --list
```

### Run benchmarks
```bash
cargo bench
```

## Test Count Inventory (v0.2.0)

| Category | Count | Status |
|----------|-------|--------|
| Unit tests | 188 | ✅ Active |
| Integration tests | 11 | ✅ Active |
| Doc tests | 2 | ✅ Active |
| **Total** | **201** | ✅ |

These counts represent the baseline for v0.2.0. New features MUST include tests; test count should increase, never decrease without documented justification.

## Why

Comprehensive testing ensures bwa-rust produces correct alignments and maintains code quality standards. The combination of unit, integration, and benchmark tests provides confidence in correctness and performance.
