# AI-Assisted Development Workflow

> Canonical workflow for developing bwa-rust with OpenSpec and AI coding assistants in a single-maintainer repo.

This document defines the concrete, project-specific workflow for AI-assisted development on bwa-rust. The default path is simple: check local state, update OpenSpec when needed, implement, validate, and push directly to `master`. Branches, worktrees, PRs, and `gh`-driven review remain available as optional tools for risky or parallel work, not mandatory ceremony.

---

## Core Principles

1. **Direct push by default** — Validate locally, then push to `master`
2. **OpenSpec as source of truth** — Specs before code
3. **Keep local state visible** — Know what is already dirty before editing
4. **Use isolation only when it helps** — Branches/worktrees are optional tools
5. **Review is selective** — Use `/review` or a review model for risky changes, not every typo fix
6. **AI tool specialization** — Right tool for the job

---

## Workflow Overview

```
┌─────────────────────────────────────────────────────────────────┐
│ 1. LOCAL STATE CHECK                                             │
│    git status / git branch --show-current                        │
│    → Understand the current worktree before starting             │
├─────────────────────────────────────────────────────────────────┤
│ 2. PROPOSE (OpenSpec)                                            │
│    /opsx:propose <name>                                          │
│    → Generate proposal.md, design.md, tasks.md                   │
│    → Review and confirm before implementation                    │
├─────────────────────────────────────────────────────────────────┤
│ 3. IMPLEMENT                                                     │
│    current worktree by default                                   │
│    → use branch/worktree only for risky or parallel changes      │
├─────────────────────────────────────────────────────────────────┤
│ 4. VALIDATE                                                      │
│    /opsx:apply                                                   │
│    + cargo fmt/clippy/test                                       │
│    → land a clean, verified change                               │
├─────────────────────────────────────────────────────────────────┤
│ 5. PUSH                                                          │
│    git push origin master                                        │
│    → direct push is the normal single-maintainer path            │
├─────────────────────────────────────────────────────────────────┤
│ 6. ARCHIVE (OpenSpec)                                            │
│    /opsx:archive                                                 │
│    → Move proposal/design/tasks to openspec/changes/archive/     │
└─────────────────────────────────────────────────────────────────┘
```

---

## 1. Local State Check

**Recommended before starting new work:**

```bash
# See what is already dirty
git status --short

# Confirm where you are working
git branch --show-current

# Only if you are already using worktrees
git worktree list
```

**Optional GitHub-side checks (only when needed):**

```bash
gh auth status
gh pr list --state open
```

**Red flags:**
- Unknown local modifications → Inspect them before editing
- Stale worktrees you no longer need → Remove them
- Open PRs only matter if you intentionally choose a branch/PR workflow for this change

**Do NOT start blind:**
- If the current worktree is already dirty and you do not understand why
- If an old worktree or branch will cause you to edit the wrong copy of the repo

---

## 2. OpenSpec Lifecycle

### 2.1 Explore Phase (Optional)

Use when you need to understand the problem before proposing:

```bash
/opsx:explore
```

**When to explore:**
- Unclear requirements
- Need to investigate existing code
- Complex changes requiring research
- User request conflicts with existing specs

**Deliverables:**
- Understanding of current implementation
- Clarified requirements
- Identified spec conflicts

### 2.2 Propose Phase (Required for Features)

Create a change proposal before implementing:

```bash
/opsx:propose <change-name>
```

**Naming convention:** `<topic>-<brief-description>`
- `paired-end-alignment`
- `optimize-seed-chain`
- `add-bam-output`

**Generated artifacts in `openspec/changes/<change-name>/`:**
- `proposal.md` — What and why
- `design.md` — How (architecture, interfaces)
- `tasks.md` — Implementation checklist

**Review before proceeding:**
- Does design align with `openspec/specs/`?
- Are all stakeholders aligned?
- Is scope reasonable for one landing unit?

**Do NOT skip proposal for:**
- New features
- API/interface changes
- Breaking changes
- Complex refactors

**Can skip proposal for:**
- Documentation fixes
- Typo corrections
- Non-functional changes (formatting, comments)
- Obvious bug fixes with clear solution

### 2.3 Apply Phase (Implementation)

Execute tasks from the proposal:

```bash
/opsx:apply
```

**Agent will:**
- Read `tasks.md` from current change
- Implement tasks in order
- Mark tasks complete in tracking system
- Follow specs in `openspec/specs/`

**Your role:**
- Review code changes as they happen
- Test incrementally
- Commit logical units of work

### 2.4 Archive Phase (After Landing the Change)

Move completed change to archive:

```bash
/opsx:archive
```

**Moves files:**
```
openspec/changes/<change-name>/
  → openspec/changes/archive/<change-name>/
```

**When to archive:**
- The change has landed on the default branch (`master`)
- All tasks are complete
- Change is no longer active

**Do NOT archive if:**
- An optional review branch or PR is still active
- Follow-up work is planned
- Change is blocked

---

## 3. Optional Worktree + Branch Management

Branches and worktrees are available when they make the change safer or easier to reason about. They are **not** the default requirement for this repository.

### 3.1 Create Worktree (Optional)

```bash
# From the default-branch worktree (e.g., /home/shane/dev/bwa-rust)
git worktree add ../bwa-rust-<change-name> -b feature/<change-name>
cd ../bwa-rust-<change-name>
```

**Worktree naming:** `bwa-rust-<change-name>`
- `bwa-rust-paired-end`
- `bwa-rust-optimize-seeds`
- `bwa-rust-fix-mapq`

**Branch naming:** `feature/<change-name>` or `fix/<change-name>`
- `feature/paired-end-alignment`
- `fix/mapq-overflow`
- `docs/update-tutorial`

**Why worktrees?**
- Isolate work without disrupting the default branch
- Switch context without stashing
- Test changes in isolation
- Keep the default-branch worktree clean for quick fixes

### 3.2 Work in Worktree

```bash
# Normal git workflow
git add <files>
git commit -m "feat: implement X"
git push origin feature/<change-name>
```

**Commit discipline:**
- Follow [Conventional Commits](https://www.conventionalcommits.org/)
- Commit logical units (not too big, not too small)
- Write clear commit messages
- Reference issue/task numbers when applicable

### 3.3 Remove Worktree After Landing the Change

```bash
# After the change has landed
cd /home/shane/dev/bwa-rust  # Back to the default-branch worktree
git worktree remove ../bwa-rust-<change-name>
git branch -d feature/<change-name>
git pull origin master  # Update the default branch
```

**Cleanup checklist:**
- Worktree removed
- Local branch deleted
- Main branch updated
- Temporary branch deleted if you created one

---

## 4. Optional Review and Branch Workflow

Direct push to `master` is the default path for this single-maintainer project once local validation passes.

Use a branch + PR only when one of these is true:
- the change is unusually risky
- you want a GitHub review surface for a large diff
- you are collaborating with someone else on the same change

### 4.1 Create PR (Optional)

```bash
# Push branch
git push origin feature/<change-name>

# Create PR (auto-fills from commits)
gh pr create --fill

# Or with custom template
gh pr create \
  --title "feat: paired-end alignment support" \
  --body "Implements tasks from openspec/changes/paired-end-alignment/tasks.md"
```

**PR checklist:**
- Title follows conventional commits format
- Description references OpenSpec proposal
- All CI checks pass (fmt, clippy, test)
- Tasks.md is complete
- No merge conflicts

### 4.2 Request Review

**When to use AI review:**

```bash
# For automated code review
gh copilot review
```

**Use `/review` agent when:**
- Large PR (>500 lines)
- Complex logic changes
- Critical path code (alignment algorithm, index building)
- First-time contributor PR review

**Do NOT use `/review` for:**
- Documentation-only changes
- Formatting/whitespace changes
- Obvious trivial fixes

**Review model selection:**
- **Haiku** — Fast review for docs/config/simple changes
- **Sonnet** — Standard review for most PRs
- **Opus** — Deep review for critical algorithm changes

### 4.3 Address Feedback

```bash
# Make changes in worktree
git add <files>
git commit -m "fix: address review feedback on X"
git push origin feature/<change-name>
```

**Feedback discipline:**
- Address all review comments
- Reply to comments when clarification is needed
- Request re-review when ready
- Do NOT force-push unless explicitly needed

### 4.4 Merge Strategy

**Default: Squash merge**
```bash
gh pr merge --squash
```

**Use rebase when:**
- Commit history is clean and meaningful
- Each commit is independently valid
- Preserving authorship is important

```bash
gh pr merge --rebase
```

**If you choose a PR flow**, `gh pr merge` keeps the GitHub-side history tidy.
For the normal single-maintainer path, push directly to `master` after local validation passes.

---

## 5. Local State Hygiene

### 5.1 Periodic Audits

Run monthly or when you feel cluttered:

```bash
# Check all worktrees
git worktree list

# Check local branches
git branch -vv

# Check remote branches
git branch -r

# Check unmerged work
git branch --no-merged master
```

### 5.2 Cleanup Stale State

```bash
# Remove stale worktrees
git worktree prune

# Delete merged local branches
git branch --merged master | grep -v "master" | xargs git branch -d

# Delete remote-tracking branches that no longer exist
git fetch --prune

# Remove worktree if work is abandoned
git worktree remove ../bwa-rust-<abandoned-change> --force
```

**Warning signs:**
- Worktrees older than 2 weeks without commits
- Temporary branches you no longer need
- Multiple unmerged branches

---

## 6. AI Tool Specialization

### 6.1 OpenSpec Agent (Primary)

**Use for:** All structured development work

```bash
/opsx:propose <name>  # Create proposal
/opsx:explore         # Investigate problem
/opsx:apply           # Implement tasks
/opsx:archive         # Archive completed change
```

**Characteristics:**
- Follows `openspec/specs/` as single source of truth
- Generates design artifacts
- Tracks tasks systematically
- Enforces spec compliance

### 6.2 Claude Code (claude.ai/code)

**Use for:**
- Interactive exploration of codebase
- Quick prototyping
- Learning/understanding existing code
- Ad-hoc analysis
- Pair programming sessions

**Context:**
- Reads `CLAUDE.md` for project context
- Has access to full codebase
- Can execute commands interactively

**Best for:**
- "Explain how SMEM seed finding works"
- "What's the difference between chain scoring and SW scoring?"
- "Help me understand the BWT construction algorithm"

**Do NOT use for:**
- Production code changes (use OpenSpec instead)
- Spec-driven feature development (use `/opsx:propose`)
- Changes requiring formal design review

### 6.3 Codex (GitHub Copilot CLI)

**Use for:**
- Command-line assistance
- Shell scripting
- Quick git operations
- Data transformations

**Example queries:**
```bash
gh copilot suggest "list all rust files with TODO comments"
gh copilot explain "cargo bench --bench index_building"
```

**Best for:**
- One-off commands
- Shell pipelines
- Git operations
- Environment setup

### 6.4 Copilot (IDE/Editor)

**Use for:**
- Inline code completion
- Function implementation from signature
- Test case generation
- Boilerplate code

**Settings:**
- Enable for Rust
- Context window: Include adjacent modules
- Accept suggestions selectively

**Best for:**
- Implementing straightforward functions
- Writing test cases
- Repetitive code patterns

---

## 7. When NOT to Use AI

**Do NOT delegate to AI:**

1. **Architectural decisions** — Discuss with team first
2. **Breaking changes** — Requires human judgment
3. **Security-critical code** — Needs expert review
4. **Performance-critical paths** — Needs profiling/benchmarking
5. **Spec conflicts** — Requires human resolution

**Red flags:**
- AI suggests violating `openspec/specs/`
- AI proposes `unsafe` code
- AI skips tests
- AI ignores existing patterns
- AI creates circular dependencies

**When to stop and ask:**
- Spec conflict detected
- Unclear requirements
- Multiple valid approaches
- Breaking change required
- Cross-module impact

---

## 8. Closeout Discipline

### 8.1 Before Considering Work Done

**Complete these steps:**

1. **Run full validation:**
   ```bash
   cargo fmt --all && \
   cargo clippy --all-targets --all-features -- -D warnings && \
   cargo test --all-targets --all-features
   ```

2. **Check for stale state:**
   ```bash
   gh pr list --state open
   git worktree list
   git status
   ```

3. **Archive OpenSpec change:**
   ```bash
   /opsx:archive
   ```

4. **Update documentation if needed:**
   - README.md if CLI changed
   - openspec/specs/ if capability changed
   - docs/ if user-facing behavior changed

5. **Verify PR merged:**
   ```bash
   gh pr view <number> --json state
   ```

### 8.2 What NOT to Do During Closeout

**Do NOT:**
- Leave worktrees lying around
- Skip test runs
- Merge without CI passing
- Archive before PR is merged
- Leave uncommitted changes
- Skip documentation updates
- Force-push to merged branches

**Do NOT create "cleanup PRs" for:**
- Worktree removal (local only)
- Branch deletion (GitHub handles this)
- Archive operations (OpenSpec local state)

---

## 9. Reference Commands

### OpenSpec

```bash
/opsx:propose <name>     # Create change proposal
/opsx:explore            # Investigate problem
/opsx:apply              # Implement tasks
/opsx:archive            # Archive completed change
```

### Git Worktree

```bash
# Create
git worktree add <path> -b <branch>

# List
git worktree list

# Remove
git worktree remove <path>

# Prune stale
git worktree prune
```

### GitHub CLI

```bash
# PRs
gh pr list --state open
gh pr create --fill
gh pr view <number>
gh pr merge --squash

# Review
gh pr review <number>
gh pr checks <number>

# Status
gh pr status
```

### Cargo Validation

```bash
# Full check
cargo fmt --all && \
cargo clippy --all-targets --all-features -- -D warnings && \
cargo test --all-targets --all-features

# Quick check
cargo check

# Watch mode (during development)
cargo watch -x check -x test
```

---

## 10. Troubleshooting

### "Worktree already exists"

```bash
# Check existing worktrees
git worktree list

# Remove if stale
git worktree remove <path>

# Force remove if necessary
git worktree remove <path> --force
```

### "Branch already exists"

```bash
# Check if branch is merged
git branch --merged master | grep <branch>

# Delete if merged
git branch -d <branch>

# Force delete if needed
git branch -D <branch>
```

### "Cannot create PR - branch behind the default branch"

```bash
# In worktree
git fetch origin master
git rebase origin/master
git push origin <branch> --force-with-lease
```

### "OpenSpec change directory not found"

```bash
# Verify you're in correct directory
pwd  # Should be in worktree root

# Check if change exists
ls openspec/changes/

# Re-propose if needed
/opsx:propose <name>
```

---

## 11. Examples

### Example 1: New Feature

```bash
# 1. Preflight
gh pr list --state open          # Verify no open PRs
git worktree list                # Verify no stale worktrees

# 2. Propose
/opsx:propose paired-end-alignment

# 3. Create worktree
git worktree add ../bwa-rust-paired-end -b feature/paired-end-alignment
cd ../bwa-rust-paired-end

# 4. Implement
/opsx:apply

# 5. Validate
cargo fmt --all && cargo clippy && cargo test

# 6. Create PR
git push origin feature/paired-end-alignment
gh pr create --fill

# 7. Merge (after review)
gh pr merge --squash

# 8. Cleanup
cd /home/shane/dev/bwa-rust
git worktree remove ../bwa-rust-paired-end
git branch -d feature/paired-end-alignment
git pull origin master

# 9. Archive
/opsx:archive
```

### Example 2: Bug Fix

```bash
# 1. Preflight (same as above)

# 2. Quick proposal (or skip for obvious fix)
/opsx:propose fix-mapq-overflow

# 3. Create worktree
git worktree add ../bwa-rust-fix-mapq -b fix/mapq-overflow
cd ../bwa-rust-fix-mapq

# 4. Implement
/opsx:apply

# 5. Add regression test
# (edit tests/integration.rs)
cargo test test_mapq_overflow -- --nocapture

# 6. Validate
cargo fmt --all && cargo clippy && cargo test

# 7-9. Same as Example 1
```

### Example 3: Documentation Update

```bash
# Can skip OpenSpec for pure doc changes
git worktree add ../bwa-rust-docs -b docs/update-tutorial
cd ../bwa-rust-docs

# Edit docs
vim docs/tutorial/getting-started.md

# Commit
git add docs/tutorial/getting-started.md
git commit -m "docs: clarify index building examples"

# Push and PR
git push origin docs/update-tutorial
gh pr create --fill

# Merge (after review)
gh pr merge --squash

# Cleanup
cd /home/shane/dev/bwa-rust
git worktree remove ../bwa-rust-docs
git branch -d docs/update-tutorial
git pull origin master
```

---

## Related Documentation

- [AGENTS.md](../../AGENTS.md) — AI agent instructions and OpenSpec overview
- [CONTRIBUTING.md](../../CONTRIBUTING.md) — General contribution guidelines
- [Development Guide](README.md) — Setup and coding standards
- [OpenSpec Specs](../../openspec/specs/) — Single source of truth for features

---

**Last Updated:** 2024-01 (v0.2.0)
