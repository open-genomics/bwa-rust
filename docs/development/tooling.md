# Development Tooling Guide

> Minimal high-value tooling configuration for bwa-rust development.

This document covers editor setup, LSP configuration, git hooks, and MCP/plugin usage guidance.

---

## Quick Start

```bash
# 1. Install git hooks
./scripts/setup-hooks

# 2. Open in editor with rust-analyzer support
# VSCode: Extensions will be recommended automatically
# Other editors: See "Editor Setup" section below
```

---

## Editor Setup

### VSCode (Recommended)

**Extensions** (auto-recommended via `.vscode/extensions.json`):
- `rust-lang.rust-analyzer` — Rust LSP
- `tamasfe.even-better-toml` — TOML support
- `redhat.vscode-yaml` — YAML validation
- `yzhang.markdown-all-in-one` — Markdown editing
- `github.copilot` — AI code completion
- `github.copilot-chat` — AI pair programming

**Settings** (`.vscode/settings.json` configured):
- Format on save enabled
- Clippy as default checker (with `-D warnings`)
- 120-char ruler matching rustfmt
- Inlay hints for types and parameters

### Neovim / Vim

**rust-analyzer via LSP:**

```lua
-- Using nvim-lspconfig
require('lspconfig').rust_analyzer.setup({
  settings = {
    ['rust-analyzer'] = {
      check = {
        command = 'clippy',
        extraArgs = { '--all-features', '--', '-D', 'warnings' }
      },
      cargo = {
        features = 'all'
      }
    }
  }
})
```

**Formatting:**
```vim
" Auto-format on save
autocmd BufWritePre *.rs lua vim.lsp.buf.format()
```

### Helix

Add to `~/.config/helix/languages.toml`:

```toml
[[language]]
name = "rust"
auto-format = true

[language-server.rust-analyzer]
command = "rust-analyzer"
config = { check = { command = "clippy", extraArgs = ["--all-features", "--", "-D", "warnings"] }, cargo = { features = "all" } }
```

### Other Editors

**Requirements:**
1. **rust-analyzer** LSP server (install via rustup or standalone)
2. **Format on save** with `rustfmt`
3. **Clippy integration** as primary checker
4. **EditorConfig support** (reads `.editorconfig`)

---

## Git Hooks

### Installation

```bash
# One-time setup
./scripts/setup-hooks
```

This installs `pre-commit` hook that runs:
1. **Format check** — `cargo fmt --all -- --check`
2. **Clippy** — `cargo clippy --all-targets --all-features -- -D warnings`
3. **Compilation check** — `cargo check --all-targets --all-features`

### Manual Validation

Run checks without committing:

```bash
./scripts/pre-commit
```

### Bypassing Hooks

Use sparingly (e.g., WIP commits in feature branch):

```bash
git commit --no-verify
```

**When to bypass:**
- Saving work-in-progress state
- Committing intentionally broken code for later fix
- Emergency hotfixes (but fix immediately after)

**When NOT to bypass:**
- Before creating PR
- Before merging to main
- When code is intended to be reviewed

---

## MCP / Plugin Tradeoffs

### What is MCP?

**MCP (Model Context Protocol)** allows AI tools to access external data sources and APIs through a standardized interface. For bwa-rust, this could include:
- GitHub API access (PR/issue management)
- Custom project indexing
- Automated spec validation
- Real-time codebase search

### Current Recommendation: GitHub Copilot CLI + Native Tools

**We deliberately avoid MCP plugins for bwa-rust** in favor of:

1. **GitHub CLI (`gh`)** — Direct GitHub API access
   ```bash
   gh pr list --state open
   gh pr create --fill
   gh pr merge --squash
   ```

2. **Native git tools** — Worktree management
   ```bash
   git worktree add ../bwa-rust-feature -b feature/name
   git worktree list
   git worktree remove ../bwa-rust-feature
   ```

3. **OpenSpec CLI** — Spec-driven development
   ```bash
   /opsx:propose <name>
   /opsx:apply
   /opsx:archive
   ```

4. **Standard tooling** — LSP, cargo, grep, ripgrep
   ```bash
   cargo clippy
   rg "TODO" --type rust
   ```

### Why Avoid MCP for This Project?

| Concern | Reason |
|---------|--------|
| **Maintenance overhead** | MCP servers require updates as APIs change |
| **Debugging complexity** | Harder to debug AI behavior with external dependencies |
| **Environment fragility** | Breaks when MCP server unavailable or misconfigured |
| **Workflow opacity** | Harder to teach/document vs explicit commands |
| **Diminishing returns** | `gh` + native tools cover 95% of needs |

### When MCP Might Make Sense

**Consider MCP if:**
- You need real-time external data (CI status, deployment metrics)
- You're integrating with complex internal APIs
- You have dedicated infrastructure team maintaining MCP servers
- Project scales to 10+ active contributors with complex coordination

**For bwa-rust (2-5 contributors, stable workflow):**
→ **Native tools are sufficient and more maintainable**

### Plugin Recommendations

**Use these IDE plugins freely** (low maintenance, high value):
- **rust-analyzer** — Essential LSP, zero config
- **GitHub Copilot** — Code completion, built-in
- **EditorConfig** — Consistent formatting, zero config
- **Markdown linting** — Docs quality, minimal config

**Avoid plugins that:**
- Require complex setup/config
- Duplicate cargo/clippy functionality
- Add custom build steps
- Require external services

---

## Copilot Usage Guidelines

### What Copilot Is Good At

✅ **Do use Copilot for:**
- Implementing straightforward functions from signatures
- Writing test cases based on existing patterns
- Generating boilerplate (struct impls, trait bounds)
- Completing repetitive code patterns
- CLI command suggestions (via `gh copilot suggest`)

### What Copilot Should NOT Do

❌ **Do NOT trust Copilot for:**
- Architectural decisions (discuss with team)
- Spec interpretation (read `openspec/specs/` yourself)
- Security-critical code (manual review required)
- Performance-critical paths (benchmark-driven)
- Breaking changes (requires human judgment)

### Copilot Configuration

**Workspace settings** (in `.vscode/settings.json`):
```json
{
  "github.copilot.enable": {
    "rust": true,
    "yaml": true,
    "markdown": true,
    "toml": true
  }
}
```

**Inline completion:**
- Accept full suggestions: `Tab`
- Accept word-by-word: `Ctrl+→`
- Reject: `Esc` or keep typing
- Cycle alternatives: `Alt+]` / `Alt+[`

**Chat usage:**
- Explain code: Select code → Copilot Chat → "Explain"
- Generate tests: `/tests` command
- Fix problems: `/fix` command
- Review guidance: See [ai-workflow.md](ai-workflow.md#review--pr-process)

---

## LSP Configuration Deep Dive

### rust-analyzer Settings

**Essential settings** (already in `.vscode/settings.json`):

```json
{
  "rust-analyzer.check.command": "clippy",
  "rust-analyzer.check.allTargets": true,
  "rust-analyzer.check.extraArgs": [
    "--all-features",
    "--",
    "-D",
    "warnings"
  ],
  "rust-analyzer.cargo.features": "all"
}
```

**Why these settings:**
- `clippy` as checker → Catches more issues than `cargo check`
- `allTargets` → Includes tests, benches, examples
- `all-features` → Validates all feature combinations
- `-D warnings` → Treats warnings as errors (same as CI)

### Inline Hints

**Recommended enabled:**
- Type hints on let bindings
- Parameter hints on function calls
- Chaining hints on iterator chains

**To disable if distracting:**
```json
{
  "rust-analyzer.inlayHints.chainingHints.enable": false,
  "rust-analyzer.inlayHints.parameterHints.enable": false
}
```

### Performance Tuning

**If rust-analyzer is slow** (unlikely for bwa-rust's size):

```json
{
  "rust-analyzer.cargo.loadOutDirsFromCheck": true,
  "rust-analyzer.procMacro.enable": true,
  "rust-analyzer.checkOnSave.enable": true,
  "rust-analyzer.checkOnSave.extraArgs": ["--target-dir", "target/rust-analyzer"]
}
```

This uses separate target dir to avoid invalidating main builds.

---

## Validation Commands

### Pre-commit (Local)

```bash
# Full validation before committing
cargo fmt --all && \
cargo clippy --all-targets --all-features -- -D warnings && \
cargo test --all-targets --all-features
```

### Quick Checks (During Development)

```bash
# Fast check without running tests
cargo check

# Watch mode (re-run on file changes)
cargo watch -x check -x test

# Single test with output
cargo test test_name -- --nocapture
```

### CI-equivalent (Before Pushing)

```bash
# Exact CI checks locally
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo build --release
```

---

## Troubleshooting

### rust-analyzer Not Working

**Symptoms:**
- No code completion
- No inline errors
- "rust-analyzer failed to load workspace" error

**Solutions:**

1. **Rebuild rust-analyzer index:**
   ```bash
   # VSCode
   Cmd+Shift+P → "rust-analyzer: Restart Server"
   
   # Or delete cache
   rm -rf target/debug/.fingerprint
   ```

2. **Check rust-analyzer logs:**
   ```bash
   # VSCode
   View → Output → Select "rust-analyzer"
   ```

3. **Verify toolchain:**
   ```bash
   rustup show
   # Should show: stable (default)
   # Components: rustfmt, clippy
   ```

4. **Check for lock file issues:**
   ```bash
   # Sometimes Cargo.lock gets corrupted
   cargo clean
   cargo update
   ```

### Pre-commit Hook Failing

**If hook fails on valid code:**

1. **Check actual errors:**
   ```bash
   ./scripts/pre-commit
   ```

2. **Fix formatting:**
   ```bash
   cargo fmt --all
   ```

3. **Fix clippy warnings:**
   ```bash
   cargo clippy --fix --allow-dirty
   ```

4. **If tests fail in hook but pass manually:**
   ```bash
   # Hook may be using different toolchain
   git diff --staged  # Check what's actually staged
   ```

### Copilot Not Suggesting

**Symptoms:**
- No inline suggestions
- Copilot icon shows "disabled"

**Solutions:**

1. **Check auth:**
   ```bash
   gh auth status
   # Should show GitHub Copilot access
   ```

2. **Check file type:**
   - Copilot may be disabled for certain file patterns
   - Check `.vscode/settings.json` → `github.copilot.enable`

3. **Restart editor**

---

## Configuration File Reference

```
.
├── .editorconfig              # Universal editor settings
├── .vscode/
│   ├── settings.json          # VSCode workspace settings
│   └── extensions.json        # Recommended extensions
├── .cargo/config.toml         # Cargo build settings
├── rust-toolchain.toml        # Rust version + components
├── rustfmt.toml              # Code formatting rules
├── clippy.toml               # Linter configuration
└── scripts/
    ├── pre-commit            # Git pre-commit hook
    └── setup-hooks           # Hook installation script
```

**Ownership:**
- `.editorconfig`, `rustfmt.toml`, `clippy.toml` — Project-wide, commit to repo
- `.vscode/` — Team recommendation, commit to repo
- `.git/hooks/` — Local, installed via `setup-hooks`
- `~/.config/` — Personal editor config, NOT in repo

---

## Best Practices

### DO

✅ Install git hooks on first clone
✅ Format on save in editor
✅ Run full validation before pushing
✅ Use Copilot for boilerplate, not architecture
✅ Check LSP diagnostics before committing
✅ Use `gh` CLI for GitHub operations
✅ Follow [ai-workflow.md](ai-workflow.md) for structured work

### DON'T

❌ Skip pre-commit checks regularly
❌ Disable clippy warnings without justification
❌ Use `#[allow(clippy::*)]` without comment
❌ Install complex MCP servers without team discussion
❌ Bypass format-on-save to "save time"
❌ Commit without running tests
❌ Use AI for security-critical code without review

---

## Related Documentation

- [ai-workflow.md](ai-workflow.md) — Complete development workflow
- [AGENTS.md](../../AGENTS.md) — AI agent instructions
- [CONTRIBUTING.md](../../CONTRIBUTING.md) — Contribution guidelines
- [OpenSpec Specs](../../openspec/specs/) — Single source of truth

---

**Last Updated:** 2024-01 (v0.2.0)
