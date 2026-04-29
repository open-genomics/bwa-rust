# Contributing

Thanks for improving bwa-rust. This repository is maintained as a compact Rust library + CLI, so contributions should be small, verified, and aligned with OpenSpec.

## Development Rules

- Requirements live in `openspec/specs/`.
- CLI defaults live in `src/align/mod.rs` `AlignOpt::default()`.
- `unsafe` code is forbidden.
- Bug fixes need regression tests.
- Planned capabilities must stay labeled planned until shipped.

## Local Setup

```bash
git clone https://github.com/LessUp/bwa-rust.git
cd bwa-rust
cargo build
npm install
```

## Before Work

```bash
git status --short --branch
openspec list --json
```

Use an OpenSpec change for features, public API changes, index format changes, complex refactors, workflow changes, or major documentation/site changes.

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
npm run docs:build
```

Run `cargo bench` when changing performance-sensitive code.

## Git Flow

This is a single-maintainer repository. Direct push to `master` after local verification is normal. Use a temporary branch or worktree only for risky or parallel work, and clean it up after landing.

## Commit Style

Use Conventional Commits:

- `feat:` shipped user-facing capability
- `fix:` bug fix
- `docs:` documentation/site change
- `test:` tests only
- `ci:` workflows
- `chore:` maintenance

## Review

Request review for changes to alignment correctness, index serialization, SAM tags, workflow permissions, release automation, or large deletions.

## Project Structure

```text
src/index/      FM-index construction and serialization
src/align/      SMEM, chaining, SW, candidate ranking, pipeline
src/io/         FASTA, FASTQ, SAM
src/util/       DNA encoding and reverse complement
openspec/       normative requirements and change artifacts
site/           VitePress public documentation
docs/           internal development/tooling notes
```
