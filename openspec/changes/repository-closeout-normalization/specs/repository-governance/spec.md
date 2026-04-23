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

### Requirement: Lightweight Git Hygiene
The repository SHALL keep git workflow lightweight for a single-maintainer project while still making local state explicit before substantial changes.

#### Scenario: Starting a scoped change
- **WHEN** a developer or AI tool begins a new scoped cleanup or feature task
- **THEN** the workflow SHALL inspect local repository state before edits
- **AND** direct push to the default branch SHALL be allowed once local validation passes

#### Scenario: Using optional isolation
- **WHEN** a change is risky, parallelized, or likely to benefit from isolation
- **THEN** the workflow MAY use a temporary branch or worktree
- **AND** that isolation SHALL remain optional rather than mandatory repository policy

### Requirement: Minimal Automation Surface
The repository SHALL retain only automation that has a clear closeout or maintenance purpose.

#### Scenario: Evaluating workflows and hooks
- **WHEN** a GitHub workflow, local hook, or engineering config is added or retained
- **THEN** it SHALL have a project-specific justification tied to correctness, releaseability, documentation, or workflow governance
- **AND** redundant or low-signal automation SHALL be removed during normalization
