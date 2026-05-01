## ADDED Requirements

### Requirement: Final Repository Baseline
The repository SHALL define a closeout-ready baseline that remains public, buildable, testable, and maintainable for the shipped single-end BWA-MEM-style aligner.

#### Scenario: Declaring closeout readiness
- **WHEN** the repository is described as closeout-ready
- **THEN** it SHALL have one canonical OpenSpec source, one coherent public narrative, one approved durable documentation tree, and one minimal verified automation surface
- **AND** unsupported capabilities such as paired-end alignment or BAM output SHALL be labeled planned or experimental rather than shipped

#### Scenario: Retaining repository surfaces
- **WHEN** a file tree, workflow, instruction file, or public page remains tracked
- **THEN** it SHALL have a current project-specific purpose tied to alignment correctness, releaseability, documentation, adoption, or handoff
- **AND** low-value generated outputs, stale scaffolding, or duplicate narrative surfaces SHALL be removed or merged

### Requirement: Worktree and Branch Closeout
The repository SHALL leave local and remote git state clean after closeout work lands.

#### Scenario: Evaluating local worktrees
- **WHEN** closeout verification runs
- **THEN** stale worktrees SHALL be inspected for unmerged or uncommitted work before removal
- **AND** only work represented on `master` or intentionally discarded by the maintainer SHALL be deleted

#### Scenario: Evaluating merged branches
- **WHEN** local or remote branches are cleanup candidates
- **THEN** the workflow SHALL verify they are merged, obsolete recovery branches, or otherwise safe to remove
- **AND** destructive branch deletion SHALL NOT occur before verification evidence is recorded in the session

### Requirement: Handoff Backlog Packaging
The repository SHALL package any remaining work into an execution-ready backlog that can be followed without rediscovering repository truth.

#### Scenario: Recording unresolved work
- **WHEN** a defect, cleanup item, or optional polish item is intentionally left out of the current closeout wave
- **THEN** it SHALL be recorded with objective, affected surfaces, dependency order, and validation gate
- **AND** the repository SHALL not depend on undocumented tribal knowledge for post-closeout work
