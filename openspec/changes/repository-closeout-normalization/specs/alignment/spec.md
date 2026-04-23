## ADDED Requirements

### Requirement: Canonical Alignment Defaults
The system SHALL maintain one canonical set of default alignment parameters across code, specs, and public documentation.

#### Scenario: Changing a default parameter
- **WHEN** a default such as `band_width`, `min_seed_len`, `max_occ`, `max_chains_per_contig`, or `max_alignments_per_read` changes
- **THEN** the implementation, canonical specs, and public-facing documentation SHALL be updated in the same change
- **AND** the repository SHALL NOT merge conflicting published defaults for the same parameter

#### Scenario: Describing alignment behavior publicly
- **WHEN** README, Pages, or support docs describe alignment behavior, limitations, or performance guards
- **THEN** the description SHALL match the currently shipped single-end alignment pipeline and memory-protection behavior
- **AND** planned capabilities SHALL be separated clearly from shipped behavior
