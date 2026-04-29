## ADDED Requirements

### Requirement: Closeout Repository Topology
The repository SHALL retain only source, canonical specs, durable documentation, purposeful scripts, minimal automation, sample data, tests, benchmarks, and approved tool configuration.

#### Scenario: Keeping tracked content
- **WHEN** top-level files or directories are added or retained
- **THEN** each item SHALL have a distinct project-specific role for the Rust aligner, OpenSpec governance, public docs, verification, release, or AI-assisted maintenance
- **AND** duplicate mirrors, abandoned scaffolding, generated outputs, and local assistant state SHALL remain untracked or be removed

### Requirement: Minimal Automation Surface
The repository SHALL retain only automation that provides unique signal for correctness, security, documentation buildability, or releaseability.

#### Scenario: Evaluating GitHub workflows
- **WHEN** a workflow is retained
- **THEN** it SHALL use least-privilege permissions and avoid duplicated builds already covered by another workflow
- **AND** scheduled workflows SHALL provide actionable signal rather than only producing noisy artifacts or flaky issue creation

### Requirement: Single-Maintainer Branch Flow
The repository SHALL default to a direct-to-`master` workflow after local verification while allowing temporary isolation for risky or parallel changes.

#### Scenario: Completing isolated work
- **WHEN** a temporary branch or worktree has landed or is no longer needed
- **THEN** it SHALL be removed after its state is verified
- **AND** only the default branch SHALL remain as the durable local development line unless active work requires otherwise
