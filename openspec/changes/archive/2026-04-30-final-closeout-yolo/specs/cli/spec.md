## ADDED Requirements

### Requirement: CLI Defaults Mirror Library Defaults
The CLI SHALL use `AlignOpt::default()` as the single source of truth for ordinary alignment defaults.

#### Scenario: Running mem without tuning flags
- **WHEN** a user runs `bwa-rust mem` without explicit scoring, seed, band, threshold, or occurrence flags
- **THEN** the constructed `AlignOpt` SHALL match `AlignOpt::default()` for those values
- **AND** any preset selected with `-x/--preset` SHALL be the only documented mechanism that changes multiple defaults at once

#### Scenario: Running align without tuning flags
- **WHEN** a user runs `bwa-rust align` without explicit tuning flags
- **THEN** the constructed `AlignOpt` SHALL match `AlignOpt::default()` for scoring, clipping, band, threshold, seed, occurrence, chain, alignment-count, thread, and zdrop values
