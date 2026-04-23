## ADDED Requirements

### Requirement: Truthful CLI Documentation
The system SHALL ensure that published CLI documentation, examples, and feature descriptions match the shipped CLI surface exactly.

#### Scenario: Publishing a command example
- **WHEN** README, Pages, support docs, or release notes publish a CLI example
- **THEN** the example SHALL use only currently supported subcommands, arguments, and invocation shapes
- **AND** the example SHALL NOT imply support for planned features that are not yet shipped

#### Scenario: Publishing CLI defaults or feature claims
- **WHEN** a public document describes CLI defaults, command behavior, or supported read modes
- **THEN** the document SHALL match the canonical CLI spec and current implementation at merge time
- **AND** unsupported workflows such as planned paired-end entry points SHALL be labeled as planned rather than standard usage
