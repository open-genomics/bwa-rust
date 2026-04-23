# CLI Specification

## Purpose

The CLI provides command-line interface for bwa-rust with index building and alignment commands following BWA-MEM conventions.

## Requirements

### Requirement: Index Command

The system SHALL provide an `index` subcommand for building FM-index.

#### Scenario: Build index from FASTA

- **WHEN** user runs `bwa-rust index <ref.fa> -o <prefix>`
- **THEN** build FM-index from reference FASTA
- **AND** output `<prefix>.fm` file

#### Scenario: Validate input file

- **WHEN** invalid or non-existent FASTA is provided
- **THEN** return error with descriptive message
- **AND** exit with code 1

### Requirement: Align Command

The system SHALL provide an `align` subcommand for aligning reads to existing index.

#### Scenario: Align FASTQ to index

- **WHEN** user runs `bwa-rust align -i <index.fm> <reads.fq>`
- **THEN** align reads to pre-built index
- **AND** output SAM to stdout

#### Scenario: Configure threading

- **WHEN** `-t` option is specified
- **THEN** use specified thread count
- **AND** default to single thread if not specified

#### Scenario: Output to file

- **WHEN** `-o` option is specified
- **THEN** write SAM to specified file
- **AND** create parent directories if needed

### Requirement: Mem Command

The system SHALL provide a `mem` subcommand for one-step index and align (BWA-MEM style).

#### Scenario: One-step alignment

- **WHEN** user runs `bwa-rust mem <ref.fa> <reads.fq>`
- **THEN** build index if not exists
- **AND** align reads to index
- **AND** output SAM to stdout

#### Scenario: Reuse existing index

- **WHEN** index file already exists
- **THEN** skip index building
- **AND** use existing index

### Requirement: Intentional Default Parameter Difference

The system SHALL use different default scoring parameters for `align` vs `mem` commands to reflect their different use cases.

**Rationale**: The `align` command is optimized for fast exact/near-exact matching against pre-built indexes (e.g., quality control, known reference scenarios), while `mem` follows BWA-MEM conventions for general-purpose alignment with greater tolerance for mismatches and gaps.

#### Scenario: Align command defaults (fast exact matching)

- **WHEN** user runs `bwa-rust align` without explicit parameters
- **THEN** use `AlignOpt::default()` values:
  - `match_score: 2`, `mismatch_penalty: 1`
  - `gap_open: 2`, `gap_extend: 1`
  - `band_width: 16`, `score_threshold: 20`
- **AND** prioritize speed and exact matches
- **EFFECT**: Lower band width (16) and higher threshold (20) favor exact/near-exact alignments

#### Scenario: Mem command defaults (BWA-MEM compatibility)

- **WHEN** user runs `bwa-rust mem` without explicit parameters
- **THEN** use BWA-MEM-like values:
  - `match_score: 1`, `mismatch_penalty: 4`
  - `gap_open: 6`, `gap_extend: 1`
  - `band_width: 100`, `score_threshold: 10`
- **AND** tolerate more divergence from reference
- **EFFECT**: Higher band width (100) and lower threshold (10) allow more distant matches typical in real sequencing data

**Note**: Both commands accept explicit parameter overrides via CLI options. This difference is intentional and should NOT be considered drift or inconsistency.

### Requirement: Alignment Parameters

The system SHALL support configuration of alignment parameters via CLI options.

#### Scenario: Configure memory limits

- **WHEN** user specifies `--max-occ`
- **THEN** filter seeds with occurrence above threshold
- **AND** default to 500 if not specified

#### Scenario: Configure chain limits

- **WHEN** user specifies `--max-chains`
- **THEN** limit chains per contig
- **AND** default to 5 if not specified

#### Scenario: Configure output limits

- **WHEN** user specifies `--max-alignments`
- **THEN** limit alignments per read
- **AND** default to 5 if not specified

### Requirement: SAM Output Format

The system SHALL output valid SAM format with complete headers and records.

#### Scenario: Generate SAM header

- **WHEN** outputting SAM
- **THEN** include @HD line with VN:1.6 and SO:unsorted
- **AND** include @SQ lines for each contig
- **AND** include @PG line with program info

#### Scenario: Format alignment record

- **WHEN** outputting alignment record
- **THEN** include all 11 required fields
- **AND** include AS:i, XS:i, NM:i tags

#### Scenario: Handle unmapped reads

- **WHEN** read has no valid alignment
- **THEN** output unmapped record with FLAG 4
- **AND** set RNAME and POS to *

### Requirement: Error Handling

The system SHALL provide clear error messages for common issues.

#### Scenario: Report invalid FASTA

- **WHEN** FASTA parsing fails
- **THEN** report file path and specific error
- **AND** exit with code 1

#### Scenario: Report invalid index

- **WHEN** index file is corrupted or wrong version
- **THEN** report specific validation failure
- **AND** suggest rebuilding index

#### Scenario: Report I/O errors

- **WHEN** file read/write fails
- **THEN** report path and system error
- **AND** suggest checking permissions/disk space

## Why

A familiar CLI interface following BWA-MEM conventions makes bwa-rust easy to adopt for existing bioinformatics workflows while providing Rust-specific safety guarantees.

## Reference: Default Parameter Values

### Align Command Defaults

The `align` command uses `AlignOpt::default()` values, optimized for fast exact matching:

| Parameter | Default Value | Description |
|-----------|---------------|-------------|
| `match_score` | 2 | Score for matching bases |
| `mismatch_penalty` | 1 | Penalty for mismatches |
| `gap_open` | 2 | Gap opening penalty |
| `gap_extend` | 1 | Gap extension penalty |
| `clip_penalty` | 1 | Penalty for soft-clipped bases (candidate sorting) |
| `band_width` | 16 | Smith-Waterman band width |
| `score_threshold` | 20 | Minimum alignment score for output |
| `min_seed_len` | 19 | Minimum SMEM seed length |
| `zdrop` | 100 | Z-drop threshold for early termination |

**Source of Truth**: `src/align/mod.rs` `AlignOpt::default()` implementation.

### Mem Command Defaults

The `mem` command uses BWA-MEM-compatible values for general-purpose alignment:

| Parameter | Default Value | Description |
|-----------|---------------|-------------|
| `match_score` | 1 | Score for matching bases (BWA-MEM: 1) |
| `mismatch_penalty` | 4 | Penalty for mismatches (BWA-MEM: 4) |
| `gap_open` | 6 | Gap opening penalty (BWA-MEM: 6) |
| `gap_extend` | 1 | Gap extension penalty (BWA-MEM: 1) |
| `clip_penalty` | 1 | Penalty for soft-clipped bases |
| `band_width` | 100 | Smith-Waterman band width (BWA-MEM: 100) |
| `score_threshold` | 10 | Minimum alignment score for output (lowered for real data) |
| `min_seed_len` | 19 | Minimum SMEM seed length (BWA-MEM: 19) |
| `zdrop` | 100 | Z-drop threshold for early termination |

**Source of Truth**: `src/main.rs` `Mem` command struct default annotations.

### Note on Parameter Overrides

Both commands accept explicit parameter overrides via CLI options (e.g., `--match`, `--mismatch`, `--band-width`). The different defaults reflect intentional design choices for different use cases and should NOT be considered drift or inconsistency.
