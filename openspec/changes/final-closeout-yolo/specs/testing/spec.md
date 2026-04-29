## ADDED Requirements

### Requirement: Closeout Regression Coverage
The test suite SHALL include regression tests for correctness defects fixed during closeout.

#### Scenario: Testing alignment option plumbing
- **WHEN** an exposed alignment option affects algorithm behavior
- **THEN** at least one unit or integration test SHALL fail if the option is ignored by the implementation

#### Scenario: Testing SAM tag coordinate handling
- **WHEN** MD:Z or SA:Z tag behavior is changed
- **THEN** tests SHALL cover soft-clipped or supplementary alignments that previously exposed coordinate or tag-suppression defects

### Requirement: Repository Consistency Verification
The repository SHALL verify documentation and configuration consistency in addition to Rust code correctness.

#### Scenario: Verifying public docs buildability
- **WHEN** closeout verification runs
- **THEN** the VitePress site SHALL build successfully from tracked sources
- **AND** the build SHALL not depend on unused PWA, analytics, or generated-output state

#### Scenario: Verifying branch and defaults consistency
- **WHEN** CI, docs, README, badges, and workflow examples reference the default branch or alignment defaults
- **THEN** they SHALL use the repository's actual default branch and `AlignOpt::default()` values unless an exception is explicitly documented
