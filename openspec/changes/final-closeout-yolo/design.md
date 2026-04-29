## Context

The repository currently passes the baseline Rust suite, but multiple independent audits found drift: `zdrop` is exposed but ignored by extension, SAM MD/SA tag generation is fragile for soft clips, the `mem` subcommand carries BWA-like defaults that conflict with `AlignOpt::default()`, public docs repeat and contradict each other, CI is overbuilt for a single-maintainer project, and AI guidance repeats boilerplate across tools.

This design treats the work as one closeout normalization wave. It keeps the Rust pipeline and OpenSpec source-of-truth model intact while reducing repository surface area and adding targeted regression tests for correctness defects.

## Goals / Non-Goals

**Goals:**

- Restore behavioral truth between CLI options, `AlignOpt`, and alignment internals.
- Make generated SAM auxiliary tags valid for soft-clipped alignments rather than suppressing or truncating tags.
- Establish a final closeout contract and align public presentation, automation, and AI tooling to it.
- Preserve cross-platform Rust 2021 compatibility, `unsafe_code = "forbid"`, and existing single-end alignment scope.
- Keep Pages as a high-value public entry point rather than a README mirror.

**Non-Goals:**

- Implement paired-end alignment, BAM/CRAM output, human-genome-scale production guarantees, or exact BWA behavioral compatibility.
- Preserve all historical docs, workflows, generated artifacts, or provider-specific AI files when they do not provide current project-specific value.
- Add heavyweight MCP servers, PWA/analytics features, or CI systems that lack clear closeout value.

## Decisions

### Decision: Fix correctness before presentation cleanup

Alignment correctness defects are handled first with TDD regression tests. `AlignOpt::zdrop` will be threaded through chain extension, query segments stored for SAM tag generation will match the CIGAR coordinate space, and CLI defaults will defer to `AlignOpt::default()` constants rather than a second parameter truth table.

Alternatives considered:
- Defer code bugs to a later feature branch. Rejected because closeout cannot be truthful if exposed knobs or SAM tags are known wrong.
- Rewrite candidate/SAM boundaries now. Rejected because the smallest correct fix is lower risk and sufficient for the discovered defects.

### Decision: Keep OpenSpec as the only normative contract

The new `closeout-readiness` capability defines final-state acceptance, while delta specs update alignment, CLI, governance, presentation, testing, and AI workflow. All durable docs should reference OpenSpec for requirements and avoid parallel normative instructions.

Alternatives considered:
- Put closeout rules only in `AGENTS.md`. Rejected because AI instructions are guidance, not the project contract.
- Keep closeout as a private plan. Rejected because follow-on agents need auditable requirements.

### Decision: Use one capability matrix for README, Pages, and GitHub metadata

The public story is constrained to shipped strengths: memory safety, single-file FM index, SMEM/chaining/SW pipeline, SAM output, rayon read parallelism, and stable single-end workflow. Planned features remain labeled as planned and are not shown as standard usage.

Alternatives considered:
- Make Pages a styled README copy. Rejected because it increases maintenance without adding public value.
- Make the site fully bilingual now. Rejected unless both locale trees are maintained; a truthful Chinese-first site with English README is lower risk.

### Decision: Minimize automation and AI tooling

CI should keep unique correctness signal: format, clippy, tests, docs build where relevant, release on tags, and security audit with minimal permissions. Disabled Dependabot scaffolding, scheduled noisy benchmarks, auto issue creation for flaky link checks, unused PWA/analytics dependencies, and duplicated AI command/skill guidance should be removed or compressed.

Alternatives considered:
- Keep all green workflows. Rejected because green noise still consumes CI and maintainer attention.
- Remove nearly all automation. Rejected because closeout still needs reproducible verification.

## Risks / Trade-offs

- **Aggressive deletion can remove useful knowledge** → classify docs before deletion and merge unique current content into canonical README/site/development pages first.
- **CI simplification can remove signal** → retain local full verification commands and release-time cross-platform builds; only remove duplicated or low-signal automation.
- **CLI default unification changes `mem` behavior** → document the change as normalization to the library truth source; presets remain available for long-read tuning.
- **GitHub-side cleanup can be destructive** → delete local/remote stale branches only after verifying they are merged or already represented on `master`.
- **Large closeout diff is hard to review** → group commits by code correctness, OpenSpec, docs/site, CI/tooling, AI guidance, and final cleanup.

## Migration Plan

1. Create regression tests for alignment parameter/tag/default behavior and make them pass.
2. Commit OpenSpec proposal, design, delta specs, and implementation tasks.
3. Normalize docs and Pages from the shared capability matrix; delete redundant docs only after preserving unique current content.
4. Simplify workflows and Node dependencies; ensure Pages still builds.
5. Rewrite AI/tooling guidance around project-specific rules and remove duplicated provider scaffolding.
6. Run full verification, perform review, update GitHub metadata, archive the OpenSpec change, merge to `master`, push, and clean stale branches/worktrees.

## Open Questions

None. The user explicitly requested YOLO execution without confirmation; decisions above are selected to minimize future maintenance while preserving shipped behavior and verification confidence.
