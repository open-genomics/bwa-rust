# Tooling

## Required Tools

- Rust toolchain from `rust-toolchain.toml`.
- Cargo, rustfmt, clippy.
- Node 22 for VitePress docs.
- GitHub CLI for repository metadata, PRs, and workflow inspection when needed.

## Editor Baseline

Use rust-analyzer with clippy checks enabled. The tracked `.vscode/settings.json` configures the shared baseline; personal overrides belong in ignored local files.

Recommended extensions:

- rust-analyzer
- CodeLLDB, only when debugging native execution
- Even Better TOML
- YAML support

## Local Verification

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
npm run docs:build
```

Use `scripts/pre-commit` if you want the same checks as a local hook.

## CLI Skills vs MCP

Prefer lightweight CLI skills and local commands for recurring repo work:

- OpenSpec proposal/apply/archive.
- TDD and systematic debugging.
- Verification and benchmark runs.
- Review dispatch.

Use MCP only when it repeatedly saves more context than it costs, such as a stable external issue tracker or domain database that cannot be queried cheaply from CLI. Do not add an MCP just because it exists.

## Node/VitePress

The docs site is intentionally plain VitePress. PWA, analytics, sitemap generation, and visual widgets are out of scope unless they are fully wired and justified by a public-documentation requirement.
