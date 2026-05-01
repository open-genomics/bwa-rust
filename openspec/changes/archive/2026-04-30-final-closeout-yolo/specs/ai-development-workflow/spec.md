## ADDED Requirements

### Requirement: Minimal Project-Specific AI Instructions
AI instruction files SHALL encode concise, project-specific behavior for bwa-rust rather than generic coding boilerplate.

#### Scenario: Updating AI guidance
- **WHEN** `AGENTS.md`, `CLAUDE.md`, or Copilot instructions are updated
- **THEN** they SHALL name the actual Rust modules, BWA-MEM-style pipeline, OpenSpec source of truth, validation commands, and closeout priorities
- **AND** they SHALL avoid duplicating long workflow manuals already represented in canonical docs or OpenSpec specs

### Requirement: Tool Role Separation
The AI workflow SHALL separate specification, implementation, review, and optional integration tooling.

#### Scenario: Choosing AI tools
- **WHEN** an AI assistant works on a scoped repository change
- **THEN** OpenSpec SHALL define the contract, implementation agents SHALL follow the contract, review agents SHALL inspect risky diffs, and GitHub CLI SHALL be used for GitHub-side metadata or PR operations
- **AND** MCPs or heavyweight plugins SHALL remain opt-in unless they provide recurring project-specific value that CLI skills cannot provide cheaply

### Requirement: Closeout Execution Discipline
AI-assisted closeout work SHALL favor small verified changes, visible local state, and explicit cleanup over broad unreviewed rewrites.

#### Scenario: Running closeout work
- **WHEN** a closeout task touches code, CI, docs, and AI configuration
- **THEN** the workflow SHALL use isolated worktrees or branches where helpful, preserve unrelated user changes, and run verification before declaring completion
- **AND** known stale worktrees or branches SHALL be inspected and resolved as part of final cleanup
