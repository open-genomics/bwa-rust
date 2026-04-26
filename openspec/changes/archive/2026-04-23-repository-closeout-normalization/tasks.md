## 1. Canonicalize repository governance ✅ COMPLETE

- [x] 1.1 Audit all repo references to legacy `specs/` paths and classify each file as migrate, rewrite, archive, or delete.
- [x] 1.2 Migrate any unique legacy spec content into `openspec/` artifacts and remove the retired `specs/` tree.
- [x] 1.3 Update contributor, support, and AI-facing docs so `openspec/` is the only normative spec source.
- [x] 1.4 Add a lightweight single-maintainer workflow that keeps local state visible without requiring PR/worktree ceremony for every change.

**Status**: All legacy specs/ references rewritten, OpenSpec established as canonical, AI workflow documented.

## 2. Reconcile shipped behavior with published claims ✅ COMPLETE

- [x] 2.1 Build a mismatch matrix for current CLI/alignment behavior versus README, Pages, roadmap, changelog, and support docs.
- [x] 2.2 Correct shipped-versus-planned capability labels, CLI examples, parameter defaults, and published metrics so they match the current implementation.
- [x] 2.3 Add or update regression checks for any corrected defaults, examples, or consistency-sensitive behavior.

**Status**: Product truth reconciled, test count updated to 201 tests (188 unit + 11 integration + 2 doc), default parameters documented, PE support clarified as planned.

## 3. Simplify engineering and AI workflow surfaces ✅ COMPLETE

- [x] 3.1 Review GitHub workflows, hooks, and engineering config files; delete, merge, or rewrite low-signal items for archive-ready maintenance.
- [x] 3.2 Redesign `AGENTS.md`, `CLAUDE.md`, `CONTRIBUTING.md`, and project-level Copilot instructions around the approved OpenSpec-driven workflow.
- [x] 3.3 Add the minimal shared editor/LSP/tooling guidance for Rust, TOML, YAML, and Markdown, and document MCP/plugin trade-offs.

**Status**: Coverage/typos/funding workflows removed, pre-commit hooks added, AI workflow documented at `docs/development/ai-workflow.md`, tooling guidance added.

## 4. Refresh public-facing surfaces ✅ COMPLETE

- [x] 4.1 Rewrite `README.md` and `README.zh-CN.md` from the reconciled capability matrix.
- [x] 4.2 Rebuild the GitHub Pages information architecture and landing content so the site adds value beyond the README.
- [x] 4.3 Use `gh` to update repository description, homepage URL, and curated topics to match the new public narrative.

**Status**: READMEs refreshed to show v0.2.0 single-end only, site streamlined, duplicate content removed.

## 5. Finalize the closeout baseline ⏳ IN PROGRESS

- [x] 5.1 Run the full repository verification suite and docs build after each cleanup wave, fixing any regressions introduced by normalization.
- [x] 5.2 Produce the final OpenSpec-aligned closeout backlog and merge/review discipline for follow-on execution by lower-cost models.

**Status**: Verification suite clean (all tests passing). Final closeout backlog packaged below.

---

## Remaining Closeout Tasks (Ready for Staged Execution)

### Stage 1: Version Control Hygiene (BLOCKING - must complete first)

**Critical**: These untracked files must be committed before the normalization changes are pushed to `master`.

#### Task 1.1: Track OpenSpec specifications
```bash
git add openspec/
git commit -m "feat: add OpenSpec specifications as single source of truth

OpenSpec replaces legacy specs/ tree and establishes canonical
governance for all future changes.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

**Verification**: `git ls-files openspec/ | wc -l` should show ~30+ tracked files

#### Task 1.2: Track AI workflow documentation
```bash
git add docs/development/ai-workflow.md docs/development/tooling.md
git commit -m "docs: add AI-assisted development workflow

Defines OpenSpec-driven workflow for Claude/Copilot/subagents with
single-maintainer direct-push defaults and optional isolation tools.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

**Verification**: `git ls-files docs/development/ | grep -E '(ai-workflow|tooling)'` should list both files

#### Task 1.3: Track site content additions
```bash
git add site/architecture/ site/benchmarks.md site/faq.md site/guide/ site/install.md
git commit -m "docs: add enhanced site content

Adds architecture guides, FAQ, benchmark info, and quickstart to
improve GitHub Pages value beyond README mirror.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

**Verification**: `git ls-files site/ | grep -E '(architecture|benchmarks|faq|install)'` should list all new content

#### Task 1.4: Track engineering tooling
```bash
git add scripts/pre-commit scripts/setup-hooks .github/DEPLOY.md
git commit -m "feat: add pre-commit hooks and tooling guidance

Pre-commit hooks enforce fmt/clippy/test discipline.
Tooling surface documents editor/LSP/MCP decisions.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

**Verification**: `git ls-files scripts/ | grep -E '(pre-commit|setup-hooks)'`

#### Task 1.5: Track test data and real_data integration test
```bash
git add tests/data/ tests/real_data.rs
git commit -m "test: add real data integration test with fixtures

Adds tests/data/ fixtures and tests/real_data.rs for end-to-end
verification with realistic sequences.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

**Verification**: `cargo test real_data` should pass

#### Task 1.6: Track .claude/ agent configuration
```bash
git add .claude/
git commit -m "feat: add Claude desktop agent configuration

Provides project-specific context for claude.ai/code sessions.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

**Verification**: `git ls-files .claude/` should show tracked files

### Stage 2: Land Normalization Changes (AFTER Stage 1)

**Goal**: Bundle all normalization changes into a clean direct-push sequence for the single-maintainer default workflow.

#### Task 2.1: Review all staged changes
```bash
git status
git diff --stat HEAD
```

**Expected**: ~50+ modified files, ~20+ deleted files, ~15+ new files

#### Task 2.2: Commit all normalization edits
```bash
git add -A
git commit -m "refactor: repository normalization and closeout preparation

Major changes:
- Remove legacy specs/ tree, establish openspec/ as canonical
- Update all references to point to OpenSpec governance
- Reconcile product truth (v0.2.0 = single-end only, PE planned)
- Update test count baseline (201 tests: 188 unit + 11 integration + 2 doc)
- Simplify GitHub workflows (remove coverage/typos/funding)
- Enhance AI workflow documentation and tooling guidance
- Refresh README/site for archive-ready state

This prepares the repository for final closeout with clear scope
boundaries and minimal technical debt.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

#### Task 2.3: Push directly to `master`
```bash
git push origin master
```

**Verification**: `git ls-remote --heads origin master` should include the new normalization commit

#### Task 2.4: Post-push verification
```bash
# Run full quality suite one more time
cargo fmt --all -- --check && \
cargo clippy --all-targets --all-features -- -D warnings && \
cargo test
```

**Expected**: All checks pass, repository ready for archive-oriented closeout

### Stage 3: Post-Landing Cleanups (AFTER Stage 2)

These are optional polish tasks that can be done incrementally after the normalization changes land on `master`.

#### Task 3.1: GitHub metadata alignment (optional)
```bash
# If homepage URL needs update after site deployment
gh repo edit --homepage "https://lessup.github.io/bwa-rust"
gh repo edit --description "BWA-MEM style DNA aligner in Rust - v0.2.0 single-end, archive-ready"
```

#### Task 3.2: Archive the OpenSpec change (optional)
```bash
# After all Stage 1-2 tasks complete
openspec archive repository-closeout-normalization
```

**Verification**: Change should move to `openspec/changes/archive/`

#### Task 3.3: Documentation site deployment (optional, if homepage configured)
See `.github/DEPLOY.md` for Pages deployment workflow.

---

## Execution Discipline for Lower-Cost Models

### Pre-Task Checklist
- [ ] Read the specific task fully
- [ ] Verify dependencies are complete (check Stage number)
- [ ] Check current git status: `git status`
- [ ] Verify tests pass before starting: `cargo test`

### During Task
- [ ] Execute exact commands shown (no variation unless blocked)
- [ ] Verify each step's success before proceeding
- [ ] If command fails, STOP and report (do not guess fixes)

### Post-Task Checklist
- [ ] Run verification command shown in task
- [ ] Run quality suite: `cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test`
- [ ] Check git status: `git status --short`
- [ ] Update SQL todo status: `UPDATE todos SET status = 'done' WHERE id = 'task-id'`

### Git Discipline
- All commits MUST include trailer: `Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>`
- Stage 1 commits are individual focused commits
- Stage 2 lands the normalization update directly on `master`
- Never commit with failing tests
- Never force-push to main

### When Blocked
- Document exact error message
- Check if dependencies (previous stages) are complete
- Report to human reviewer rather than attempting creative workarounds
