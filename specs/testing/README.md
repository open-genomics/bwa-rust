# Testing Specifications

This directory contains testing strategy and BDD test case specifications.

## Documents

| Document | Description |
|----------|-------------|
| [test-strategy.md](./test-strategy.md) | Testing strategy and requirements |

## Test Categories

| Category | Location | Count |
|----------|----------|-------|
| Unit Tests | `#[cfg(test)] mod tests` | 151 |
| Integration Tests | `tests/integration.rs` | 11 |
| Module Tests | Module files | 5 |
| Doc Tests | `src/lib.rs` | 1 |

## Test Execution

```bash
cargo test                          # Run all tests
cargo test --lib                    # Library tests only
cargo test --test integration       # Integration tests only
cargo test <name> -- --exact        # Run specific test
```
