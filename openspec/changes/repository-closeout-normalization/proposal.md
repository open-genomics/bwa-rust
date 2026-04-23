## Why

The repository is operational, but its governance, documentation, and public project story have drifted out of sync. `openspec/` is intended to be the canonical spec source, yet the repo still carries a parallel legacy `specs/` system, contradictory feature/version claims, fragmented workflow guidance, and incomplete GitHub presentation, which makes the project harder to finish cleanly and increases the risk of low-quality maintenance work.

This change aggressively converges the repository toward an archive-ready state: one normative spec system, one truthful project narrative, one lightweight AI-assisted workflow, and a smaller set of high-signal engineering assets.

## What Changes

- Establish `openspec/` as the sole canonical spec system and retire legacy `specs/` content and references. **BREAKING**: internal repo paths, contributor guidance, and documentation links may change.
- Define repository governance requirements for directory structure, spec workflow, worktree/branch/PR hygiene, `gh` preflight checks, review gates, and hooks.
- Define project-specific AI/tooling governance for `AGENTS.md`, `CLAUDE.md`, Copilot instructions, LSP/editor setup, and minimal MCP/plugin usage.
- Reconcile public claims with implementation reality by removing or correcting stale feature promises, outdated defaults, stale counts, and invalid examples across README, site, roadmap, changelog, and support docs.
- Redesign GitHub Pages and GitHub About metadata so the project is presented around its actual strengths rather than a generic documentation mirror.
- Simplify workflows and engineering configuration to an archive-ready minimum that preserves confidence without redundant automation.
- Produce an execution-ready closeout backlog that lower-cost models can follow safely under OpenSpec.

## Capabilities

### New Capabilities
- `repository-governance`: Defines the canonical repository structure, normative spec source, workflow rules, and archive-readiness constraints.
- `ai-development-workflow`: Defines how OpenSpec, Claude, Codex, Copilot, reviews, subagents, worktrees, and PRs should be used together for this project.
- `project-presentation`: Defines truthful, consistent public-facing requirements for README, GitHub Pages, repository About metadata, and related project messaging.

### Modified Capabilities
- `cli`: Published CLI examples, descriptions, and supported-feature claims must match shipped behavior.
- `alignment`: Documented defaults and behavior claims must be reconciled with implementation or corrected at the source.
- `testing`: Repository closeout verification must cover documentation and consistency checks in addition to existing Rust quality gates.

## Impact

- Affected specs: `openspec/specs/cli/`, `openspec/specs/alignment/`, `openspec/specs/testing/`, plus new governance/presentation/workflow capabilities.
- Affected repo surfaces: `AGENTS.md`, `CLAUDE.md`, `CONTRIBUTING.md`, `README*.md`, `docs/`, `site/`, `.github/`, hook/config files, and GitHub repository metadata managed with `gh`.
- Affected implementation areas: CI/workflow definitions, developer guidance, site structure, and any code/docs/config fixes required to eliminate spec and behavior drift.
