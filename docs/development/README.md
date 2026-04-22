# Development Guide

> Guide for contributors and developers working on bwa-rust.

---

## Getting Started

### Setup Development Environment

```bash
# Clone repository
git clone https://github.com/LessUp/bwa-rust.git
cd bwa-rust

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Build and Test

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run all tests with features
cargo test --all-targets --all-features

# Run benchmarks
cargo bench
```

### Code Quality

```bash
# Format code
cargo fmt --all

# Check formatting
cargo fmt --all -- --check

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings
```

---

## Project Structure

```
src/
├── main.rs          # CLI entry
├── lib.rs           # Library entry
├── error.rs         # Error types
├── io/              # Input/output (FASTA/FASTQ/SAM)
├── index/           # Index building (SA, BWT, FM)
├── align/           # Alignment algorithms
└── util/            # Utilities (DNA encoding)
```

---

## Coding Standards

### Style Guide

- Follow standard Rust conventions
- Use `cargo fmt` for formatting
- Maximum line width: 120 characters
- Use descriptive variable names

### Unsafe Code

```toml
# Project enforces no unsafe code
[lints.rust]
unsafe_code = "forbid"
```

### Error Handling

```rust
// Library code: use BwaError/BwaResult
pub type BwaResult<T> = Result<T, BwaError>;

// Application code: use anyhow
use anyhow::{anyhow, bail, Result};
```

---

## Testing

### Running Tests

```bash
# All tests
cargo test

# Specific test
cargo test test_name

# With output
cargo test -- --nocapture

# Integration tests only
cargo test --test integration
```

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function() {
        let result = function_to_test();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_edge_case() {
        // Test boundaries
    }
}
```

---

## Architecture

### Module Dependencies

```
cli (main.rs)
  ↓
  ├── align/
  │     └── index/
  │     └── util/
  ├── index/
  │     └── util/
  └── io/
      └── util/
```

### Key Design Principles

1. **No unsafe code** — Memory safety via compiler
2. **Modular** — Single responsibility per module
3. **Testable** — Unit tests for all modules
4. **Documented** — Document public APIs

---

## Release Process

### Version Bump

1. Update `Cargo.toml` version
2. Update `CHANGELOG.md`
3. Update documentation
4. Run full test suite

### Creating Release

```bash
# Commit changes
git add .
git commit -m "chore: release v0.x.x"

# Tag release
git tag v0.x.x

# Push
git push origin main --tags

# Create GitHub release
gh release create v0.x.x --notes-file RELEASE_NOTES.md
```

---

## Contributing

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for detailed contribution guidelines.

---

## Resources

- [Architecture Docs](../architecture/)
- [Tutorial](../tutorial/)
- [API Reference](../api/)
