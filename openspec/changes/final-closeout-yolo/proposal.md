## Why

bwa-rust has a working Rust baseline, but closeout audits found repository drift across code behavior, public documentation, GitHub Actions, AI instructions, and stale worktrees. This change makes the repository truthful, compact, verifiable, and ready for low-risk single-maintainer operation rather than preserving noisy scaffolding from earlier model-driven development.

## What Changes

- Fix latent alignment correctness/configuration defects discovered during closeout: `zdrop` must affect extension, soft-clipped alignments must retain correct MD/SA tags, and CLI defaults must match the library truth source.
- Create a final closeout-readiness contract covering archive-ready repository shape, handoff backlog, worktree/branch hygiene, and final verification.
- Tighten governance around tracked files, generated outputs, stale branches, minimal automation, and direct-to-`master` single-maintainer workflow.
- Rebuild README, durable docs, and GitHub Pages around shipped single-end alignment strengths, explicit limitations, and a non-mirrored public landing experience.
- Reduce GitHub Actions, dependency automation, Node/VitePress configuration, and AI instruction files to the smallest high-signal project-specific surface.
- Update GitHub metadata and remove or retire stale worktrees/branches after verification.

## Capabilities

### New Capabilities
- `closeout-readiness`: Defines final-state completion criteria, branch/worktree cleanup, handoff backlog, and repository verification required before closeout.

### Modified Capabilities
- `alignment`: Require exposed alignment tuning options to affect the algorithm and require SAM auxiliary tags to stay correct for soft-clipped alignments.
- `cli`: Align `mem` and `align` defaults with `AlignOpt::default()` unless a documented preset overrides them.
- `repository-governance`: Tighten repository topology, generated-output discipline, branch/worktree cleanup, and minimal automation policy.
- `project-presentation`: Require README, Pages, and GitHub metadata to share one truthful capability matrix while Pages provides standalone value.
- `ai-development-workflow`: Minimize AI instructions and define tool-role boundaries for OpenSpec, review, CLI skills, and optional MCPs.
- `testing`: Extend verification to cover closeout consistency, docs buildability, configuration defaults, and regression tests for fixed alignment bugs.

## Impact

- Affected Rust code: `src/align/`, `src/io/sam.rs`, `src/main.rs`, and related tests.
- Affected specs: `openspec/specs/alignment/`, `cli/`, `repository-governance/`, `project-presentation/`, `ai-development-workflow/`, `testing/`, plus new `closeout-readiness/`.
- Affected repository surfaces: `README*.md`, `docs/`, `site/`, `.github/workflows/`, `.github/dependabot.yml`, package files, `AGENTS.md`, `CLAUDE.md`, Copilot instructions, editor/tooling config, and stale worktrees/branches.
- Verification impact: full closeout requires `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-targets --all-features`, and the Pages build.
