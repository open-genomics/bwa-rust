## ADDED Requirements

### Requirement: Configurable Extension Termination
The alignment pipeline SHALL use the configured Z-drop threshold when extending chained seeds into CIGAR-bearing alignments.

#### Scenario: Applying zdrop during chain extension
- **WHEN** an alignment option sets `zdrop` to a positive value
- **THEN** left and right chain extension SHALL terminate according to that configured value
- **AND** the implementation SHALL NOT use a hard-coded Z-drop constant that ignores `AlignOpt::zdrop`

### Requirement: Correct SAM Tags for Soft-Clipped Alignments
The alignment pipeline SHALL generate MD:Z and SA:Z tags from query/reference slices that match the full CIGAR coordinate space.

#### Scenario: Generating MD for soft-clipped alignments
- **WHEN** a mapped alignment contains leading or trailing soft clipping
- **THEN** MD:Z generation SHALL compare the aligned `M/=/X/D/N` operations against the correct query and reference bases
- **AND** soft-clipped query bases SHALL be skipped without causing aligned query bases to be offset, truncated, or dropped

#### Scenario: Emitting SA independently from MD availability
- **WHEN** supplementary alignments exist for a read
- **THEN** SA:Z generation SHALL be available for records even if MD:Z generation is unavailable for a candidate
- **AND** missing MD data SHALL NOT suppress an otherwise valid SA:Z tag
