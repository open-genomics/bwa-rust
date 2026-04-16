# Dependencies

This document lists the dependencies used by bwa-rust and their purposes.

## Direct Dependencies

| Crate | Version | Purpose | License |
|-------|---------|---------|---------|
| `clap` | 4.5 | CLI argument parsing | MIT/Apache-2.0 |
| `serde` | 1.0 | Serialization framework | MIT/Apache-2.0 |
| `bincode` | 1.3 | Binary serialization for index | MIT |
| `rayon` | 1.10 | Data parallelism | MIT/Apache-2.0 |
| `chrono` | 0.4 | Timestamp handling | MIT/Apache-2.0 |
| `anyhow` | 1.0 | Error handling (CLI) | MIT/Apache-2.0 |
| `tikv-jemallocator` | 0.6 | Memory allocator (non-Windows) | Apache-2.0 |

## Development Dependencies

| Crate | Version | Purpose | License |
|-------|---------|---------|---------|
| `criterion` | 0.5 | Benchmarking | MIT/Apache-2.0 |
| `proptest` | (optional) | Property testing | MIT/Apache-2.0 |

## Build Dependencies

| Crate | Version | Purpose | License |
|-------|---------|---------|---------|
| `tikv-jemalloc-sys` | 0.6 | jemalloc native bindings | BSD-3-Clause |

## License Compatibility

All dependencies are compatible with the MIT license used by bwa-rust.

## Security Audits

- Dependencies are automatically checked by `cargo audit` in CI
- Dependabot is enabled for automated dependency updates
