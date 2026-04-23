# AI-Assisted Development Workflow

> Canonical workflow for developing bwa-rust with OpenSpec, GitHub CLI, worktrees, and AI coding assistants.

This document defines the concrete, project-specific workflow for AI-assisted development on bwa-rust. It combines OpenSpec lifecycle management, git worktree discipline, GitHub CLI integration, and effective use of Claude/Codex/Copilot.

---

## Core Principles

1. **One worktree per change** — Isolate work, avoid context switching
2. **One branch per feature** — Clean PR history
3. **OpenSpec as source of truth** — Specs before code
4. **Preflight before work** — Check state before starting
5. **PR merge discipline** — Review, test, merge, cleanup
6. **AI tool specialization** — Right tool for the job

---

## Workflow Overview

```
┌─────────────────────────────────────────────────────────────────┐
│ 1. PREFLIGHT                                                     │
│    gh pr list / git worktree list / git branch                   │
│    → Check for stale state before starting                       │
├─────────────────────────────────────────────────────────────────┤
│ 2. PROPOSE (OpenSpec)                                            │
│    /opsx:propose <name>                                          │
│    → Generate proposal.md, design.md, tasks.md                   │
│    → Review and confirm before implementation                    │
├─────────────────────────────────────────────────────────────────┤
│ 3. WORKTREE + BRANCH                                             │
│    git worktree add ../bwa-rust-<change-name> -b <branch-name>  │
│    → Isolate work, prevent default-branch contamination          │
├─────────────────────────────────────────────────────────────────┤
│ 4. IMPLEMENT (OpenSpec Apply)                                    │
│    /opsx:apply                                                   │
│    → Execute tasks from tasks.md                                 │
│    → Commit incrementally with meaningful messages               │
├─────────────────────────────────────────────────────────────────┤
│ 5. VALIDATE                                                      │
│    cargo fmt --all && cargo clippy && cargo test                 │
│    → Ensure code quality before pushing                          │
├─────────────────────────────────────────────────────────────────┤
│ 6. REVIEW & PR                                                   │
│    git push origin <branch-name>                                 │
│    gh pr create --fill                                           │
│    → Request review, address feedback                            │
├─────────────────────────────────────────────────────────────────┤
│ 7. MERGE & CLEANUP                                               │
│    gh pr merge --squash / --rebase                               │
│    git worktree remove ../bwa-rust-<change-name>                 │
│    git branch -d <branch-name>                                   │
├─────────────────────────────────────────────────────────────────┤
│ 8. ARCHIVE (OpenSpec)                                            │
│    /opsx:archive                                                 │
│    → Move proposal/design/tasks to openspec/changes/archive/     │
└─────────────────────────────────────────────────────────────────┘
```

---

## 1. Preflight Checklist

**Always run before starting new work:**

```bash
# Check for open PRs
gh pr list --state open

# Check for stale worktrees
git worktree list

# Check for unmerged branches
git branch --no-merged master

# Check for uncommitted changes in the default-branch worktree
git status
```

**Red flags:**
- Open PRs → Merge or close before starting new work
- Stale worktrees → Remove with `git worktree remove <path>`
- Unmerged branches → Investigate and clean up
- Uncommitted changes in the default-branch worktree → Commit or stash

**Do NOT start new work if:**
- You have open PRs awaiting review/merge
- Previous worktrees exist but aren't actively being worked on
- Main worktree has uncommitted changes

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
- Is scope reasonable for one PR?

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

### 2.4 Archive Phase (After Merge)

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
- PR is merged to the default branch (`master`)
- All tasks are complete
- Change is no longer active

**Do NOT archive if:**
- PR is still open
- Follow-up work is planned
- Change is blocked

---

## 3. Worktree + Branch Management

### 3.1 Create Worktree

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

### 3.3 Remove Worktree After Merge

```bash
# After PR is merged
cd /home/shane/dev/bwa-rust  # Back to the default-branch worktree
git worktree remove ../bwa-rust-<change-name>
git branch -d feature/<change-name>
git pull origin master  # Update the default branch
```

**Cleanup checklist:**
- Worktree removed
- Local branch deleted
- Main branch updated
- Remote branch deleted (GitHub does this automatically on merge)

---

## 4. PR & Review Process

### 4.1 Create PR

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

**Never merge locally** — Always use `gh pr merge` to ensure:
- CI passes
- Reviews are approved
- Branch protection rules are satisfied

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
- Branches with no associated PR
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
