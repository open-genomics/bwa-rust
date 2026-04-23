## ADDED Requirements

### Requirement: Canonical Spec Source
The repository SHALL treat `openspec/` as the only normative source for requirements, workflow contracts, and capability definitions.

#### Scenario: Updating normative references
- **WHEN** contributor guidance, automation, or support docs reference the project's source of truth
- **THEN** they SHALL reference `openspec/` paths only
- **AND** they SHALL NOT present legacy `specs/` paths as canonical

#### Scenario: Retiring legacy spec content
- **WHEN** legacy `specs/` content contains information not yet represented in `openspec/`
- **THEN** that information SHALL be migrated or intentionally discarded before the legacy file is removed
- **AND** the repository SHALL not keep two active spec trees for the same concern after the change merges

### Requirement: Controlled Repository Topology
The repository SHALL maintain a single approved structure for specifications, implementation, documentation, automation, and generated outputs.

#### Scenario: Adding or retaining top-level content
- **WHEN** a top-level file or directory is added or kept during normalization
- **THEN** it SHALL have a distinct project-specific responsibility
- **AND** duplicate mirrors, abandoned scaffolding, or low-value generated artifacts SHALL be removed from version control

#### Scenario: Handling generated outputs
- **WHEN** build artifacts such as Pages output, benchmark output, or local caches are generated
- **THEN** they SHALL remain ignored or session-local unless the project explicitly versions them as source material
- **AND** closeout cleanup SHALL verify that such outputs are not part of the canonical repository topology

### Requirement: Worktree and PR Discipline
The repository SHALL use change-scoped branches and worktrees with explicit GitHub preflight checks before substantial work begins.

#### Scenario: Starting a scoped change
- **WHEN** a developer or AI tool begins a new scoped cleanup or feature task
- **THEN** the workflow SHALL check `gh` authentication, local git status, worktree state, and PR state before edits
- **AND** the work SHALL proceed on a dedicated branch or worktree until it is merged or retired

#### Scenario: Detecting stale local or remote state
- **WHEN** preflight finds stale worktrees, abandoned branches, or open PRs that conflict with the new task
- **THEN** those conflicts SHALL be surfaced before continuing
- **AND** the workflow SHALL require pruning, reuse, or explicit deferral instead of silently accumulating more drift

### Requirement: Minimal Automation Surface
The repository SHALL retain only automation that has a clear closeout or maintenance purpose.

#### Scenario: Evaluating workflows and hooks
- **WHEN** a GitHub workflow, local hook, or engineering config is added or retained
- **THEN** it SHALL have a project-specific justification tied to correctness, releaseability, documentation, or workflow governance
- **AND** redundant or low-signal automation SHALL be removed during normalization
