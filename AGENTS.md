# AGENTS.md

## Project

BWA-MEM style DNA short-read aligner in Rust. Library + CLI.

**Pipeline**: FASTA/FASTQ → FM-index → SMEM seeds → chains → Smith-Waterman → SAM

**Current parameters** (v0.2.0, see `src/align/mod.rs` `AlignOpt::default()` for truth):
- match_score=2, mismatch_penalty=1, gap_open=2, gap_extend=1
- band_width=16, min_seed_len=19, max_occ=500
- max_chains_per_contig=5, max_alignments_per_read=5

## OpenSpec-Driven Development (Critical)

All code must follow `openspec/specs/` as the single source of truth.

**REQUIRED: Read [docs/development/ai-workflow.md](docs/development/ai-workflow.md) before starting any work.**

**Quick workflow reference**:
1. **Assess local state**: `git status --short` — Know what is already dirty before editing
2. **Propose**: `/opsx:propose <name>` — Generate proposal/design/tasks before coding
3. **Implement**: `/opsx:apply` — Execute tasks from tasks.md
4. **Validate**: `cargo fmt --all && cargo clippy && cargo test` — Ensure quality
5. **Push**: direct push to `master` is the default for this single-maintainer repo
6. **Isolate only when needed**: use a branch/worktree for risky or parallel work, not by default
7. **Archive**: `/opsx:archive` — Move proposal to archive after the change lands

### OpenSpec Commands

| Command | Description |
|---------|-------------|
| `/opsx:propose <name>` | Create a new change proposal |
| `/opsx:apply` | Implement tasks from the proposal |
| `/opsx:archive` | Archive completed change |
| `/opsx:explore` | Explore codebase before proposing |

### Spec Structure

```
openspec/specs/
├── alignment/           # Core alignment capability
├── index-building/      # FM-index construction
├── alignment-algorithm/ # Algorithmic details
├── cli/                 # CLI interface spec
├── architecture/        # Architecture design
├── ai-development-workflow/ # AI-assisted development workflow
├── repository-governance/   # Repository structure and maintenance rules
├── project-presentation/    # README, Pages, and GitHub metadata rules
├── testing/             # Testing strategy
└── paired-end-alignment/ # PE alignment (planned)
```

## Commands

```bash
cargo test                                    # All tests
cargo test <name> -- --nocapture              # Single test with output
cargo test --test integration                 # Integration tests only
cargo fmt --all -- --check                    # Format check
cargo clippy --all-targets --all-features -- -D warnings  # Lint
cargo bench                                   # Benchmarks
```

**After code changes**: run `cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings && cargo test`

## Critical Constraints

- **`unsafe_code = "forbid"`** — enforced in Cargo.toml lints
- **No comments in code** unless user explicitly requests
- Comments/docs in codebase use Chinese and English — maintain local style when editing nearby code
- Max line width: 120 chars (`rustfmt.toml`)

## Architecture

```
src/
├── main.rs              # CLI (clap)
├── lib.rs               # Library entry
├── index/               # SA, BWT, FM-index construction
├── align/               # Seeding, chaining, SW, pipeline
├── io/                  # FASTA, FASTQ, SAM
└── util/dna.rs          # DNA encoding/revcomp
```

**Tests**: Unit tests in `#[cfg(test)] mod tests` within modules; integration tests in `tests/integration.rs`

## Domain Knowledge

- **Alphabet**: `{0:$, 1:A, 2:C, 3:G, 4:T, 5:N}`, SIGMA=6
- **Index format**: Single `.fm` file (bincode), magic=`0x424D4146_4D5F5253`

## API Changes

- External API changes → update `openspec/specs/cli/`
- Index format changes → increment version, maintain backward compat
- New capabilities → use `/opsx:propose` to create spec first

## Tooling

**Editor/LSP**: See `.vscode/settings.json` and `.editorconfig` for recommended setup
**Pre-commit validation**: Use `scripts/pre-commit` hook (or run manually)
**MCP/Plugin guidance**: See [docs/development/tooling.md](docs/development/tooling.md)
