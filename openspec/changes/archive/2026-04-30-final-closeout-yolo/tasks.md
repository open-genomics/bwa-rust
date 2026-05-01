## 1. OpenSpec and planning foundation

- [x] 1.1 Create the `final-closeout-yolo` OpenSpec proposal, design, delta specs, and implementation checklist
- [x] 1.2 Save the execution plan under `docs/superpowers/plans/` with concrete verification gates

## 2. Alignment correctness and CLI truth

- [x] 2.1 Add failing regression tests for zdrop plumbing, soft-clipped MD/SA tag handling, and CLI default drift
- [x] 2.2 Thread `AlignOpt::zdrop` through chain extension and fix soft-clipped query/reference tag slices
- [x] 2.3 Unify CLI default values with `AlignOpt::default()` and update affected docs/spec claims

## 3. Documentation and Pages normalization

- [x] 3.1 Build a shared capability matrix and rewrite README/README.zh-CN around shipped strengths and limitations
- [x] 3.2 Reduce `docs/` to durable development/API guidance and remove or merge stale tutorial/architecture duplicates
- [x] 3.3 Rebuild VitePress navigation/content so Pages is a public portal, not a README/changelog mirror

## 4. Workflow and tooling minimization

- [x] 4.1 Simplify GitHub Actions to least-privilege high-signal CI, Pages, release, and audit workflows
- [x] 4.2 Remove disabled/noisy dependency, benchmark, link-check, PWA, analytics, or sitemap scaffolding that lacks closeout value
- [x] 4.3 Normalize Node/editor/LSP/tooling configuration and verify docs build from the reduced toolchain

## 5. AI instruction and handoff hardening

- [x] 5.1 Rewrite `AGENTS.md`, `CLAUDE.md`, and Copilot instructions to be concise, project-specific, and non-duplicative
- [x] 5.2 Update development guidance with the direct-to-`master` flow, `/review` usage, CLI skills vs MCP trade-off, and closeout handoff rules
- [x] 5.3 Package any remaining optional work into a final backlog rather than leaving implicit TODOs

## 6. Verification, metadata, and cleanup

- [x] 6.1 Run full verification: format check, clippy, all-target tests, and Pages build
- [x] 6.2 Request final code/config review and address findings
- [x] 6.3 Update GitHub description/homepage/topics with `gh` if authenticated
- [ ] 6.4 Archive the OpenSpec change, merge/push to `master`, and remove verified stale local/remote worktrees or branches
