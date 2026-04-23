## Context

bwa-rust currently has a healthy implementation baseline: formatting, clippy, tests, and the VitePress docs build all pass. The main problems are cross-cutting repository drift issues rather than a broken build: the repo has both `openspec/` and legacy `specs/` structures, public documents disagree on shipped capabilities and defaults, Pages positioning is weak, AI/developer guidance is fragmented, and GitHub metadata is only partially configured.

This change is driven by a closeout objective. The project should be easy to finish well and safe to hand off to lower-cost models, not optimized for ongoing feature expansion. The selected cleanup strategy is aggressive convergence: remove duplicate truth sources, delete low-value artifacts, and accept internal path/link changes when that is the cleanest resolution.

Constraints:
- The repository already has a dirty working tree, so implementation must avoid clobbering unrelated edits.
- The project remains a Rust/Cargo codebase with VitePress docs and GitHub-hosted workflows; the cleanup should not introduce a second build system.
- Cross-platform support must remain intact for Linux, macOS, and Windows.
- `gh` is available and should be the source of truth for repository metadata and PR state.

## Goals / Non-Goals

**Goals:**
- Make `openspec/` the only normative spec system.
- Reconcile implementation reality, specs, README, site, and GitHub About into one truthful project narrative.
- Define one lightweight, project-specific AI-assisted workflow using worktrees, PRs, review gates, and `gh` preflight.
- Reduce workflows, docs, and configuration to a smaller, higher-signal set suitable for archive-readiness.
- Produce OpenSpec artifacts and task sequencing that lower-cost models can execute safely.

**Non-Goals:**
- Finish large planned features such as paired-end alignment, BAM output, or BWA native index compatibility unless they are strictly required to remove contradictions.
- Re-architect the core aligner pipeline or change fundamental algorithm choices without a separately scoped change.
- Preserve all legacy repo paths, links, or duplicated documentation systems for compatibility.
- Add broad MCP/plugin ecosystems or heavyweight automation that increases maintenance burden.

## Decisions

### Decision: Converge on OpenSpec as the only normative repository contract

`openspec/` becomes the sole canonical source for requirements and workflow contracts. Legacy `specs/` content will be classified into migrate, condense, archive, or delete, and all contributor-facing references will be redirected to `openspec/`.

**Why:** Leaving both trees in place guarantees continued drift. Aggressive convergence reduces ambiguity for humans and AI tools.

**Alternatives considered:**
- **Keep both systems with bridge docs:** rejected because it preserves dual ownership and doubles review burden.
- **Soft-deprecate legacy `specs/` indefinitely:** rejected because stale references will continue to mislead cheaper models.

### Decision: Perform truth reconciliation before public polish

The cleanup will first reconcile what the code actually does, what the specs say, and what public surfaces claim. README, Pages, roadmap, changelog, and GitHub metadata will only be refreshed after that truth matrix is stable.

**Why:** Polishing first would freeze incorrect claims into more places and make later cleanup harder.

**Alternatives considered:**
- **Redesign Pages first:** rejected because the current narrative already overstates some capabilities.
- **Only fix code/document mismatches opportunistically:** rejected because it misses repo-wide consistency failures.

### Decision: Encode a closeout-first workflow around worktrees, PRs, and `gh` preflight

Every scoped change should start from a dedicated branch/worktree, run a short preflight (`gh auth`, `git status`, `git worktree list`, `gh pr status`), and end in a PR that passes review before merge. Local hooks should be purposeful and deterministic, not magical.

**Why:** The repo is already drifting partly because changes can land without a consistent integration path. A lightweight, explicit workflow lowers the coordination burden for AI-assisted development.

**Alternatives considered:**
- **Direct branch editing without worktrees:** rejected because it makes parallel cleanup work riskier.
- **Heavy local automation with many hidden hooks:** rejected because it becomes opaque and fragile.

### Decision: Prefer minimal, project-specific AI tooling over broad generic tooling

The repository will keep strong project-local instructions (`AGENTS.md`, `CLAUDE.md`, Copilot instructions) and a small recommended LSP/editor baseline, while treating MCP/plugin additions as opt-in exceptions that require explicit value justification.

**Why:** This repo benefits more from precise project guidance than from a large tool surface that consumes context and increases maintenance cost.

**Alternatives considered:**
- **Enable many MCP servers and plugins by default:** rejected because context cost is high and the repo is near closeout.
- **Rely only on ad hoc prompts:** rejected because cheaper models need stronger repository-specific constraints.

### Decision: Reduce automation by signal, not by count alone

Existing workflows and hooks will be reviewed for closeout value. The target state is a small set of workflows that prove correctness, build public docs, and support releases/essential governance, while removing redundant or low-signal jobs.

**Why:** Archive-ready does not mean no automation; it means only meaningful automation survives.

**Alternatives considered:**
- **Keep every existing workflow because it is green:** rejected because green noise is still noise.
- **Collapse everything into one mega-workflow:** rejected because it hurts clarity and failure isolation.

### Decision: Build public presentation from a single capability matrix

README, GitHub Pages, repository About, and topic metadata will be rewritten from the same reconciled capability matrix: shipped, experimental, and planned. Unsupported features will not appear in usage examples as if they are already available.

**Why:** The current problem is not a lack of content but contradictory content.

**Alternatives considered:**
- **Maintain separate narratives for README and Pages:** rejected because it recreates the drift.
- **Keep broad future-looking marketing copy:** rejected because the project is being optimized for clean finish, not hype.

## Risks / Trade-offs

- **[Aggressive deletion breaks internal links]** → Migrate unique content first, then update references in the same cleanup wave.
- **[Dirty working tree collides with normalization edits]** → Inspect targeted files carefully and isolate future work in scoped branches/worktrees.
- **[Truthful positioning makes the project look smaller]** → Emphasize real strengths: memory safety, single-file FM index, readable architecture, and stable baseline quality.
- **[Workflow reduction removes useful guardrails]** → Keep only workflows that provide unique closeout value and verify coverage before deleting others.
- **[Cheaper models still drift]** → Encode the workflow and capability boundaries directly in OpenSpec, AGENTS, CLAUDE, and Copilot instructions.

## Migration Plan

1. Create the umbrella OpenSpec change and define the new governance/presentation/workflow requirements.
2. Inventory repo surfaces and map every legacy or conflicting artifact to migrate, condense, archive, or delete.
3. Reconcile capability/default/version claims against actual implementation and produce the bug/drift list.
4. Normalize repo docs, AI guidance, workflows, hooks, and GitHub metadata in dependency order.
5. Refresh README and Pages only after canonical truth and workflow guidance are stable.
6. Run full verification and leave an execution-ready closeout backlog for remaining fixes.

## Open Questions

- Should the repository keep `master` as the default branch or rename it to `main` as part of the cleanup?
- Should bilingual docs remain broad, or should the public docs surface be reduced to a smaller, tighter bilingual set?
