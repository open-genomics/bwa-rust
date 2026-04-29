## ADDED Requirements

### Requirement: Shared Capability Matrix
README files, GitHub Pages, roadmap-facing content, and GitHub About metadata SHALL describe shipped, planned, and unsupported capabilities from one reconciled capability matrix.

#### Scenario: Publishing capability claims
- **WHEN** a public surface mentions FM-index building, SMEM seeding, chaining, Smith-Waterman extension, SAM output, paired-end support, BAM output, or performance claims
- **THEN** the capability SHALL be labeled shipped, planned, experimental, or unsupported consistently across README and Pages
- **AND** planned capabilities SHALL NOT appear in standard usage examples as if already supported

### Requirement: Pages as Public Portal
GitHub Pages SHALL act as the primary public portal and SHALL add structure beyond README duplication.

#### Scenario: Designing the landing page
- **WHEN** the Pages landing page is revised
- **THEN** it SHALL present the project promise, target users, key differentiators, explicit limitations, quick adoption path, and architecture value within the first-level experience
- **AND** it SHALL avoid changelog dumping or one-to-one README section mirroring as the main content strategy

### Requirement: Link and Language Truthfulness
Public documentation SHALL avoid links, language switches, release artifact names, or installation claims that are not supported by tracked repository behavior.

#### Scenario: Publishing install instructions
- **WHEN** installation docs mention release artifacts, Docker images, crates, or Pages URLs
- **THEN** those claims SHALL match existing workflow outputs or be clearly marked future/planned
- **AND** unsupported language trees or hidden routes SHALL not be advertised as available documentation
