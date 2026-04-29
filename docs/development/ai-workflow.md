# AI-Assisted Development Workflow

This is the project-specific workflow for AI agents working on bwa-rust.

## Non-Negotiables

- `openspec/specs/` is the only normative source for requirements.
- `AlignOpt::default()` in `src/align/mod.rs` is the only default-parameter truth source.
- The shipped product is a single-end FASTA/FASTQ -> FM-index -> SMEM -> chain -> SW -> SAM aligner.
- Do not add `unsafe` code.
- Do not add code comments unless explicitly requested or preserving nearby style.
- Preserve the single-maintainer default: verify locally, then push to `master`.

## Start Of Session

```bash
git status --short --branch
openspec list --json
```

Use GitHub CLI only when the task actually needs GitHub-side information or metadata.

## Change Selection

- New features, public API changes, index format changes, complex refactors, CI/governance changes: create an OpenSpec change.
- Obvious typo/docs fixes and simple local bug fixes may skip a proposal, but still need tests or verification.
- Risky or parallel work may use a temporary worktree; routine work may happen directly on the default branch.

## Implementation Discipline

- Bug fixes use TDD: write the regression test, watch it fail, implement the smallest fix, watch it pass.
- Keep algorithmic changes localized to `src/align/`, `src/index/`, `src/io/`, or `src/util/` according to responsibility.
- Do not rewrite candidate/chaining/SW boundaries unless the OpenSpec change explicitly calls for it.
- Update README, Pages, and specs when shipped behavior or public claims change.

## Review Discipline

Use `/review` or a review subagent for:

- alignment correctness, MAPQ, CIGAR, MD:Z, SA:Z, or index serialization changes;
- workflow permissions or release changes;
- repository-wide deletions;
- diffs too large to audit mentally.

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
npm run docs:build
```

Benchmarks are local/manual unless a performance change specifically requires them:

```bash
cargo bench
```

## Closeout Cleanup

Before final handoff or archival closeout:

```bash
```

Remove only worktrees and branches already represented on `master` or intentionally abandoned by the maintainer. Archive completed OpenSpec changes after they land.
