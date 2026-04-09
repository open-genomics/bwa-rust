# AGENTS.md

Repository guide for coding agents working in `bwa-rust`.

## Scope
- This repository is a Rust implementation of a BWA-MEM style DNA short-read aligner.
- Main pipeline: FASTA/FASTQ I/O -> FM index -> seeding -> chaining -> SW extension -> SAM output.
- Treat correctness and reproducibility as higher priority than clever refactors.
- Prefer small, local changes that match existing patterns.

## Rule Files
- No repo-local Cursor rules were found in `.cursor/rules/`.
- No `.cursorrules` file was found.
- No Copilot instructions file was found at `.github/copilot-instructions.md`.
- Follow this file, `CLAUDE.md`, repository config, and existing source patterns.

## Toolchain And CI
- Rust MSRV is `1.70` (`Cargo.toml`).
- CI runs, in order:
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-targets --all-features`
- `cargo build --release`
- If you change code, aim to leave the tree passing the same sequence.

## Primary Commands
- Build debug: `cargo build`
- Build release: `cargo build --release`
- Run all tests: `cargo test`
- Run all library tests: `cargo test --lib`
- Run integration tests only: `cargo test --test integration`
- Run all targets with all features: `cargo test --all-targets --all-features`
- Run formatting check: `cargo fmt --all -- --check`
- Apply formatting: `cargo fmt --all`
- Run clippy exactly like CI: `cargo clippy --all-targets --all-features -- -D warnings`
- Run benchmarks: `cargo bench`

## Running A Single Test
- Run a unit test by substring: `cargo test error_display`
- Run a unit test exactly: `cargo test error_display -- --exact`
- Run a lib test exactly: `cargo test --lib error_display -- --exact`
- Run an integration test exactly: `cargo test --test integration e2e_build_index_and_exact_search -- --exact`
- Show output for one test: `cargo test align_single_read_unmapped -- --exact --nocapture`
- List available tests: `cargo test -- --list`
- When changing one module, prefer the narrowest command that exercises that code first, then rerun broader checks.

## Binary And Example Commands
- Build an FM index: `cargo run -- index <ref.fa> -o <prefix>`
- Align against an existing index: `cargo run -- align -i <prefix>.fm <reads.fq>`
- One-step BWA-MEM style run: `cargo run -- mem <ref.fa> <reads.fq>`
- Run the example: `cargo run --example simple_align`

## Repository Layout
- `src/main.rs`: CLI entrypoint using `clap`.
- `src/lib.rs`: library entrypoint and test helpers.
- `src/error.rs`: structured library errors via `BwaError` and `BwaResult`.
- `src/index/`: suffix array, BWT, FM-index, index builder.
- `src/align/`: seeds, chains, SW, candidate generation, pipeline.
- `src/io/`: FASTA/FASTQ readers and SAM formatting.
- `src/util/`: DNA normalization, alphabet encoding, reverse complement.
- `tests/integration.rs`: end-to-end and cross-module validation.
- `benches/benchmarks.rs`: Criterion benchmarks.

## Code Style Expectations
- Use Rust 2021 edition style.
- Respect `rustfmt.toml`; maximum line width is `120`.
- Use 4-space indentation; do not use hard tabs.
- Let `cargo fmt` own final formatting instead of hand-aligning code.
- Keep code ASCII unless a file already uses non-ASCII heavily.
- Existing code includes Chinese and English comments/docstrings; preserve local style when editing nearby code.

## Imports
- Follow the existing import grouping pattern:
- Standard library imports first.
- Third-party crate imports next.
- `crate::...` imports next.
- `super::...` imports last.
- Separate groups with a blank line.
- Prefer explicit imports over glob imports.
- Use grouped imports when it improves readability, for example `use crate::index::{bwt, sa};`.

## Naming
- Types and traits: `UpperCamelCase`.
- Functions, methods, modules, variables, tests: `snake_case`.
- Constants: `UPPER_SNAKE_CASE`.
- Use domain names that match the codebase: `fm`, `sa`, `bwt`, `contig`, `seed`, `chain`, `mapq`, `revcomp`.
- Favor short, precise names over generic helper names.

## Types And Data Modeling
- Prefer plain structs with explicit fields for algorithm state and records.
- Derive only what is needed, commonly `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Serialize`, `Deserialize`, `Default`.
- Use `usize` for in-memory indexing and lengths.
- Use `u32` for persisted/reference coordinates where the repo already does so.
- When crossing `usize`/`u32` boundaries, use checked conversions (`try_from`) if overflow is possible.
- Mark pure helper functions `#[must_use]` when ignoring the result would be suspicious.

## Control Flow And Implementation Style
- Prefer early returns for invalid inputs and empty fast paths.
- Keep logic local; add a helper only when it clearly improves readability or reuse.
- Reuse buffers in hot paths when the surrounding code already does so.
- Preserve performance-sensitive choices such as rayon batching, SA sampling, and SW buffer reuse unless there is a strong reason to change them.
- Avoid unnecessary allocations in hot alignment/index code.
- Do not introduce `unsafe`; the crate forbids it.

## Error Handling
- Library-facing structured errors live in `src/error.rs` as `BwaError` and `BwaResult<T>`.
- Much of the implementation currently uses `anyhow::Result`; follow the surrounding layer.
- Use `anyhow!`/`bail!` for concise contextual failures in CLI/internal plumbing.
- Include concrete context in error messages, especially file paths, expected invariants, and offending values.
- Validate inputs early: empty FASTA entries, duplicate contig names, zero block sizes, malformed FASTQ records, invalid thread counts.
- Reserve `unwrap`/`expect` for tests or invariant-level assumptions that are already validated nearby.

## Comments And Documentation
- Add comments only where algorithmic intent or invariants are non-obvious.
- Prefer comments that explain why, not line-by-line narration.
- Public APIs and data structures often have concise doc comments; maintain that style.
- Keep examples and terminology aligned with bioinformatics concepts already used in the repo.

## Testing Expectations
- Add or update unit tests beside the changed module when behavior is local.
- Add or update integration tests in `tests/integration.rs` when behavior crosses modules or affects end-to-end output.
- Cover edge cases aggressively: empty input, invalid format, boundary coordinates, reverse complement behavior, duplicate names, score thresholds.
- Prefer exact assertions over loose truthiness when stable values are available.
- It is acceptable to use `unwrap()` in tests for setup and expected-success paths.

## Performance And Concurrency
- Non-Windows builds use `jemalloc`; do not remove this casually.
- Alignment pipeline supports single-thread and rayon-backed multi-thread execution.
- Avoid changing batching, thread-pool, or buffer-reuse behavior without measuring impact.
- Criterion benchmarks exist; use `cargo bench` for performance-sensitive work.

## Working In This Repo
- Check for nearby tests before editing a module.
- Prefer minimal diffs over broad cleanups.
- Keep public API changes intentional; this repo exposes a library as well as a CLI.
- Preserve serialized/index format compatibility unless a change explicitly intends to bump it.
- If you change file format assumptions, metadata, or SAM output semantics, update tests accordingly.
- If you touch algorithm ranking or scoring, review MAPQ, clipping, and secondary-alignment behavior together.

## Good Agent Workflow
- Read the target module and its tests first.
- Make the smallest correct code change.
- Run the narrowest relevant test.
- Run `cargo fmt --all` if you changed Rust code.
- Run targeted clippy/tests, then broader checks if the change is significant.
- Before handing off, summarize changed files, behavior changes, and any checks not run.

## Quick Module Map
- FM-index core: `src/index/fm.rs`
- Index construction: `src/index/builder.rs`, `src/index/sa.rs`, `src/index/bwt.rs`
- Seeding: `src/align/seed.rs`
- Chaining: `src/align/chain.rs`
- Extension/SW: `src/align/extend.rs`, `src/align/sw.rs`
- Candidate ranking and dedup: `src/align/candidate.rs`
- Full mapping pipeline: `src/align/pipeline.rs`
- FASTA/FASTQ parsing: `src/io/fasta.rs`, `src/io/fastq.rs`
- SAM formatting: `src/io/sam.rs`
- DNA utilities: `src/util/dna.rs`

## Agent Reminders
- Match existing style before introducing new patterns.
- Keep formatting, lint, and tests green.
- Prefer precise fixes over speculative refactors.
