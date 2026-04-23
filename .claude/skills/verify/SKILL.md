---
name: verify
description: Run full verification suite (format check, clippy, tests) before marking work done.
---

Run the verification pipeline for this Rust project:

```bash
cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test
```

Report any failures. Do NOT mark work as complete if verification fails.
