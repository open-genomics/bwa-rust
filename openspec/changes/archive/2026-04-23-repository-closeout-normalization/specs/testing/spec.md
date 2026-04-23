## ADDED Requirements

### Requirement: Repository Consistency Verification
The repository SHALL verify documentation and consistency gates in addition to Rust code correctness gates.

#### Scenario: Verifying public documentation buildability
- **WHEN** the repository runs its closeout verification workflow
- **THEN** it SHALL build the GitHub Pages documentation successfully
- **AND** the verification SHALL fail if the public docs surface no longer builds from tracked sources

#### Scenario: Verifying canonical references
- **WHEN** repository consistency checks run
- **THEN** they SHALL fail if contributor-facing or support-facing documents still present retired spec paths as canonical
- **AND** they SHALL fail if generated outputs that are meant to stay ignored are committed as source artifacts
