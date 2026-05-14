# Benchmarks & Verification Boundaries

bwa-rust aims to provide a clear, memory-safe, tunable Rust single-end alignment baseline—not to fully replace BWA in the current version.

## Current Assessment

| Item | Status |
|------|--------|
| Microbenchmarks | `cargo bench` covers FM-index search, SMEM, SW, SA construction hotspots. |
| Single-end throughput | Supports rayon read-level parallelism. |
| Human genome production validation | Not claimed complete. |
| BWA bit-level compatibility | Not a goal. |

## Local Run

```bash
cargo bench
```

CI runs benchmarks intermittently because GitHub-hosted runners are noisy and lack a stable baseline trending system. Performance-related changes should run benchmarks locally or in dedicated environments with documented comparison conditions.

## Correctness Verification

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

Real data tests are in `tests/real_data.rs`, ignored by default, requiring explicit data preparation and feature flags.
