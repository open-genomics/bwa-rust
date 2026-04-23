## ADDED Requirements

### Requirement: FM-Index Sequence Alignment

The system SHALL build FM-index from FASTA reference genome for sequence alignment.

#### Scenario: Build index from valid FASTA

- **WHEN** a valid FASTA file is provided to the index command
- **THEN** create a `.fm` index file with magic header `BWAFM_RS`
- **AND** the index SHALL contain C-table, BWT, and sparse SA samples

### Requirement: SMEM Seed Finding

The system SHALL find Super-Maximal Exact Matches (SMEMs) for seeding alignment.

#### Scenario: Find longest exact match

- **WHEN** a read sequence is processed for seeding
- **THEN** find the longest exact match covering each position
- **AND** extend left until the match is no longer maximal

### Requirement: Banded Smith-Waterman Alignment

The system SHALL perform banded affine-gap local alignment for chain extension.

#### Scenario: Align with banded SW

- **WHEN** a chain is selected for extension
- **THEN** perform banded Smith-Waterman alignment
- **AND** use configurable band width (default: 16)

### Requirement: SAM Output

The system SHALL output alignment results in SAM format.

#### Scenario: Generate SAM header

- **WHEN** alignment output begins
- **THEN** generate `@HD` header line (VN:1.6, SO:unsorted)
- **AND** generate `@SQ` lines for each contig
