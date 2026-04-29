# CLAUDE.md

Claude Code should follow `AGENTS.md` as the primary project instruction file. This file only adds Claude-specific operating guidance.

## Required Context

Before changing code, read the relevant OpenSpec capability under `openspec/specs/` and inspect local git state.

High-risk areas:

- `src/align/sw.rs`, `extend.rs`, `candidate.rs`, `pipeline.rs`: CIGAR/NM/MD/SA correctness.
- `src/index/fm.rs`: serialized index compatibility.
- `.github/workflows/`: permissions and release artifacts.
- `site/` and README files: public capability claims.

## Preferred Workflow

1. Use `/opsx:propose <name>` for non-trivial behavior, governance, workflow, or public documentation changes.
2. Use `/opsx:apply` to execute tasks.
3. Use TDD for bug fixes.
4. Run verification before claiming completion.
5. Use `/review` for risky diffs.
6. Use `/opsx:archive` after a completed change lands on `master`.

## Project Defaults

`AlignOpt::default()` is the only truth for ordinary CLI/library alignment defaults:

- `match_score=2`, `mismatch_penalty=1`
- `gap_open=2`, `gap_extend=1`
- `clip_penalty=1`, `band_width=16`, `score_threshold=20`
- `min_seed_len=19`, `max_occ=500`
- `max_chains_per_contig=5`, `max_alignments_per_read=5`
- `threads=1`, `zdrop=100`

Do not reintroduce BWA defaults for `mem` unless a named preset explicitly applies them.

## Commands

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
npm run docs:build
```

## Tooling Policy

Prefer local CLI skills and repository scripts over MCP servers. Add or recommend MCP only when it provides repeated, project-specific value that cannot be achieved cheaply with `cargo`, `openspec`, `gh`, or file search.
