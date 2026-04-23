## ADDED Requirements

### Requirement: Truthful Capability Messaging
Public-facing repository surfaces SHALL describe only the capabilities that are actually shipped, experimental, or planned, using consistent labels across the project.

#### Scenario: Mentioning a planned capability
- **WHEN** README, Pages, roadmap, or GitHub About mentions a capability that is not shipped
- **THEN** that capability SHALL be labeled as planned or experimental consistently
- **AND** the repository SHALL NOT present it in standard usage examples as if it is already supported

#### Scenario: Publishing defaults, metrics, or counts
- **WHEN** a public document publishes default parameter values, workflow metrics, or test counts
- **THEN** the values SHALL match the current repository truth at merge time
- **AND** stale or inflated numbers SHALL be corrected in the same change that updates the surrounding narrative

### Requirement: Pages Must Add Standalone Value
GitHub Pages SHALL provide a useful project entry point that goes beyond mirroring the README.

#### Scenario: Designing the landing experience
- **WHEN** the Pages homepage or navigation is revised
- **THEN** the first navigation layer SHALL explain target users, core differentiators, limitations, and an adoption path
- **AND** the site SHALL not merely restate README sections one-to-one without added structure or value

### Requirement: Repository Metadata Alignment
GitHub repository metadata SHALL match the canonical public narrative and direct users to the project's primary public entry point.

#### Scenario: Updating repository About fields
- **WHEN** repository description, homepage URL, or topics are updated with `gh`
- **THEN** the homepage SHALL point to the active GitHub Pages URL
- **AND** the description and topic set SHALL reflect the reconciled shipped strengths of the project using a small, curated topic list
