# Memory Safety

## Overview

bwa-rust enforces a **zero unsafe code** policy through Cargo lint configuration. This design choice provides strong memory safety guarantees critical for bioinformatics applications.

## Policy Enforcement

```toml
# Cargo.toml
[lints]
unsafe_code = "forbid"
```

This configuration makes any use of `unsafe` blocks a **compilation error**, not just a warning.

## What This Means

### Guaranteed by the Compiler

- **No buffer overflows**: Array accesses are bounds-checked
- **No use-after-free**: Ownership system prevents dangling references
- **No data races**: Thread safety enforced at compile time
- **No null pointer dereferences**: `Option<T>` replaces nullable pointers

### Not Claimed

- **Algorithm correctness**: Safety guarantees don't ensure algorithmic correctness
- **Performance optimization**: Safe code may be slower than hand-tuned unsafe code
- **BWA compatibility**: Output format is similar but not bit-identical to BWA

## Design Philosophy

### Why Forbid unsafe?

1. **Auditability**: Every line of code is safe by default
2. **Learning value**: Students can read the entire codebase without unsafe concerns
3. **Security**: Suitable for processing untrusted genomic data
4. **Maintainability**: Refactoring is safer without unsafe invariants

### Trade-offs

| Aspect | With unsafe | Without unsafe |
|--------|-------------|----------------|
| Performance | Potentially faster | Adequate for intended use |
| SIMD | Can use intrinsics | Use portable alternatives |
| FFI | Can call C libraries | Pure Rust only |
| Memory layout | Can optimize | Rely on compiler |

## Code Examples

### Safe Array Access

```rust
// This would be a compile error in bwa-rust
let value = unsafe { *ptr.offset(i) }; // ❌ Forbidden

// Instead, use safe indexing
let value = vec[i]; // ✅ Bounds-checked
```

### Safe Concurrency

```rust
use rayon::prelude::*;

// Parallel processing without data races
reads.par_iter().for_each(|read| {
    // Each iteration is isolated by Rust's ownership rules
    let alignment = align_read(read, &index);
});
```

## Verification

The policy is verified at compile time:

```bash
cargo build
# Any unsafe code would cause: error: usage of an `unsafe` block
```

## References

- [Rust Safety Guarantees](https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html)
- [Cargo Lint Levels](https://doc.rust-lang.org/cargo/reference/manifest.html#the-lints-section)

---

[Next: Performance Analysis →](/en/deep-dive/performance)
