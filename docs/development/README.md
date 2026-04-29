# Development Guide

This repository is a Rust library + CLI for a BWA-MEM-style single-end aligner. Development is intentionally direct: inspect local state, update OpenSpec when behavior changes, implement, verify, and push to `master` after checks pass.

## Required Local Checks

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
npm run docs:build
```

## Source Of Truth

- Requirements: `openspec/specs/`
- Change proposals: `openspec/changes/<name>/`
- CLI defaults: `src/align/mod.rs` `AlignOpt::default()`
- Public story: `README.md`, `README.zh-CN.md`, and `site/`

## Default Flow

1. Run `git status --short --branch`.
2. For features, API changes, complex refactors, or closeout governance changes, create an OpenSpec change.
3. Keep code changes small and test-first for bug fixes.
4. Run local verification before claiming completion.
5. Push directly to `master` for routine single-maintainer work.
6. Use a temporary branch/worktree only for risky or parallel work, then remove it after landing.

Use `/review` or a review subagent for alignment logic, index format, workflow permissions, or large documentation rewrites.
