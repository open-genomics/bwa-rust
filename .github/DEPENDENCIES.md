# Dependencies

Core dependencies for bwa-rust:

| Crate | Purpose |
|-------|---------|
| `clap` | CLI argument parsing |
| `serde` + `bincode` | Index serialization |
| `rayon` | Multi-threading |
| `anyhow` | CLI error handling |
| `tikv-jemallocator` | Memory allocator (non-Windows) |

**Development:**
- `criterion` - Benchmarking

**Security:**
- Dependencies audited by `cargo audit` in CI
- Dependabot enabled for automated updates

All dependencies are MIT/Apache-2.0 compatible.
