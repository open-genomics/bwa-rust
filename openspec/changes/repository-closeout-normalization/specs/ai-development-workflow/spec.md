## ADDED Requirements

### Requirement: Project-Specific AI Instructions
The repository SHALL maintain AI instruction files that encode project-specific constraints, module boundaries, workflow rules, and closeout priorities.

#### Scenario: Updating instruction files
- **WHEN** `AGENTS.md`, `CLAUDE.md`, or Copilot instruction files are updated
- **THEN** they SHALL describe the actual repository structure, OpenSpec workflow, and project closeout intent
- **AND** they SHALL avoid generic filler that does not change developer or model behavior for bwa-rust

### Requirement: Tool Role Separation
The workflow SHALL assign clear roles to planning, implementation, and review tools.

#### Scenario: Running a scoped change
- **WHEN** a change spans planning, specification, implementation, and review
- **THEN** OpenSpec SHALL define the contract, implementation agents SHALL follow the approved specs, and a review model or `/review` step SHALL be available for risky or high-impact changes
- **AND** subagents SHALL be used only for naturally parallel audit, verification, or implementation slices

### Requirement: Lightweight Session Awareness
AI-assisted development sessions SHALL make local repository state visible without requiring heavyweight GitHub-side preflight for every change.

#### Scenario: Starting an AI session
- **WHEN** Claude, Codex, Copilot, OpenCode, or another AI assistant is used for a repository change
- **THEN** the workflow SHALL surface local modifications and branch/worktree context when they materially affect the task
- **AND** `gh` checks SHALL remain optional unless the change needs GitHub-side operations

### Requirement: Minimal Tooling Configuration
The repository SHALL prefer a small, explicit tooling surface over broad generic integrations.

#### Scenario: Selecting baseline tooling
- **WHEN** project-level tooling guidance is documented
- **THEN** it SHALL define a minimal recommended baseline for Rust, TOML, YAML, and Markdown editing plus repository-specific Copilot guidance
- **AND** it SHALL treat high-context-cost MCPs or plugins as opt-in exceptions rather than default requirements

#### Scenario: Adding an MCP or plugin
- **WHEN** a new MCP, plugin, or auxiliary tool is proposed
- **THEN** the repository SHALL document the recurring project-specific value it provides
- **AND** the tool SHALL NOT be added by default if the same outcome is achievable with lower context and lower maintenance cost
