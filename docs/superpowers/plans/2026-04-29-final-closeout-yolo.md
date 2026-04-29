# Final Closeout YOLO Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make bwa-rust closeout-ready by fixing known alignment defects, normalizing specs/docs/site/CI/AI tooling, verifying the full repository, and cleaning stale git state.

**Architecture:** Keep the Rust alignment pipeline intact and apply minimal correctness fixes at the option plumbing and SAM tag boundaries. Treat OpenSpec as the normative contract, Pages as the public portal, and GitHub Actions/AI guidance as minimal project-specific support surfaces.

**Tech Stack:** Rust 2021, Cargo, clap, rayon, VitePress, GitHub Actions, OpenSpec.

---

### Task 1: OpenSpec Foundation

**Files:**
- Create: `openspec/changes/final-closeout-yolo/proposal.md`
- Create: `openspec/changes/final-closeout-yolo/design.md`
- Create: `openspec/changes/final-closeout-yolo/specs/**/spec.md`
- Create: `openspec/changes/final-closeout-yolo/tasks.md`
- Create: `docs/superpowers/plans/2026-04-29-final-closeout-yolo.md`

- [ ] **Step 1: Verify change status**

Run: `openspec status --change final-closeout-yolo --json`
Expected: `applyRequires` includes `tasks` and artifacts become complete after files are written.

- [ ] **Step 2: Validate apply readiness**

Run: `openspec status --change final-closeout-yolo`
Expected: change is ready for implementation with proposal, design, specs, and tasks present.

### Task 2: Alignment Correctness

**Files:**
- Modify: `src/align/extend.rs`
- Modify: `src/align/candidate.rs`
- Modify: `src/align/pipeline.rs`
- Modify: `src/align/supplementary.rs`
- Modify: `src/io/sam.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Add failing tests**

Add regression tests proving `zdrop` changes extension length, soft-clipped MD uses aligned bases, SA generation is not tied to MD availability, and `mem` default values match `AlignOpt::default()`.

- [ ] **Step 2: Run focused tests and confirm failure**

Run: `cargo test zdrop md_tag_with_soft_clip align_single_read -- --nocapture`
Expected: at least one new test fails before implementation.

- [ ] **Step 3: Implement minimal fixes**

Thread `opt.zdrop` into chain extension, store full oriented query slices for CIGAR-based MD generation, emit SA independently from MD, and set CLI defaults to `AlignOpt::default()` values.

- [ ] **Step 4: Verify focused tests pass**

Run: `cargo test zdrop md_tag_with_soft_clip align_single_read -- --nocapture`
Expected: all focused tests pass.

### Task 3: Public Docs and Pages

**Files:**
- Modify: `README.md`
- Modify: `README.zh-CN.md`
- Modify/Delete: `docs/**`
- Modify/Delete: `site/**`

- [ ] **Step 1: Build capability matrix**

Define shipped/planned/unsupported claims for single-end alignment, FM-index, SMEM/chaining/SW, SAM output, paired-end, BAM, performance, and library usage.

- [ ] **Step 2: Rewrite README pair**

Keep README focused on positioning, installation, quick start, current scope, docs link, and verification commands.

- [ ] **Step 3: Normalize docs and site**

Remove stale duplicates, fix broken links/artifact names, and make Pages content a portal with target users, differentiators, limitations, adoption, and architecture value.

- [ ] **Step 4: Build site**

Run: `npm run docs:build`
Expected: VitePress build succeeds.

### Task 4: Workflow and Tooling

**Files:**
- Modify/Delete: `.github/workflows/*.yml`
- Modify/Delete: `.github/dependabot.yml`
- Modify: `package.json`
- Modify: `package-lock.json`
- Modify: `.vscode/settings.json`
- Modify: `docs/development/tooling.md`

- [ ] **Step 1: Simplify workflows**

Keep CI, Pages, release, and audit with least privilege; remove or disable duplicated/noisy scheduled automation.

- [ ] **Step 2: Trim unused Node tooling**

Remove unused PWA/analytics/sitemap dependencies and wiring unless actively used by VitePress.

- [ ] **Step 3: Verify local toolchain**

Run: `npm install` then `npm run docs:build`
Expected: lockfile is consistent and docs build succeeds.

### Task 5: AI Guidance and Handoff

**Files:**
- Modify: `AGENTS.md`
- Modify: `CLAUDE.md`
- Create/Modify: `.github/copilot-instructions.md`
- Modify: `docs/development/ai-workflow.md`
- Modify: `docs/development/tooling.md`

- [ ] **Step 1: Rewrite AI guidance**

Make each file short, project-specific, and non-duplicative: pipeline, module map, OpenSpec, no unsafe, no code comments unless requested, verification, and closeout workflow.

- [ ] **Step 2: Record MCP vs CLI-skills posture**

Prefer CLI skills and local tools for low-token recurring work; use MCP only for repeated external integration value.

- [ ] **Step 3: Record final backlog**

Keep only explicit future work with validation gates; do not leave vague TODOs.

### Task 6: Final Verification and Cleanup

**Files:**
- Modify: `openspec/changes/final-closeout-yolo/tasks.md`
- Eventually archive to: `openspec/changes/archive/2026-04-29-final-closeout-yolo/`

- [ ] **Step 1: Full Rust verification**

Run: `cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-targets --all-features`
Expected: all pass.

- [ ] **Step 2: Review**

Run a code/config review over Rust, docs, CI, and AI guidance; address high/medium findings.

- [ ] **Step 3: GitHub metadata**

Run `gh repo edit LessUp/bwa-rust --description "Memory-safe BWA-MEM style single-end DNA aligner in Rust" --homepage "https://lessup.github.io/bwa-rust/" --add-topic bioinformatics --add-topic genomics --add-topic rust --add-topic fm-index --add-topic sequence-alignment` if authenticated.

- [ ] **Step 4: Archive and cleanup**

Archive the OpenSpec change after tasks complete, merge/push `master`, then remove verified stale worktrees and merged/recovery branches.
