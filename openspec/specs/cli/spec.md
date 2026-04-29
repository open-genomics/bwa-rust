# CLI Specification

## Purpose

Define the shipped command-line interface for bwa-rust: index construction, single-end alignment against an existing index, and one-step in-memory `mem` alignment.

## Requirements

### Requirement: Index Command

The system SHALL provide an `index` subcommand for building a single-file FM-index from FASTA.

#### Scenario: Build index from FASTA

- **WHEN** a user runs `bwa-rust index <ref.fa> -o <prefix>`
- **THEN** the CLI SHALL build an FM-index from the reference FASTA
- **AND** write `<prefix>.fm`

#### Scenario: Reject invalid reference input

- **WHEN** the FASTA path is missing, malformed, empty, or contains duplicate contig names
- **THEN** the CLI SHALL return a descriptive error
- **AND** exit unsuccessfully

### Requirement: Align Command

The system SHALL provide an `align` subcommand for aligning single-end FASTQ reads to an existing `.fm` index.

#### Scenario: Align FASTQ to existing index

- **WHEN** a user runs `bwa-rust align -i <index.fm> <reads.fq>`
- **THEN** the CLI SHALL load the pre-built index
- **AND** output SAM records to stdout unless `-o/--out` is provided

#### Scenario: Configure output file

- **WHEN** `-o/--out <path>` is specified
- **THEN** the CLI SHALL write SAM output to that path

#### Scenario: Configure threading

- **WHEN** `-t/--threads <n>` is specified with `n >= 1`
- **THEN** the CLI SHALL use that thread count for read-level parallelism
- **AND** reject `0` as invalid

### Requirement: Mem Command

The system SHALL provide a `mem` subcommand for building an index in memory and aligning single-end FASTQ reads in one command.

#### Scenario: One-step alignment

- **WHEN** a user runs `bwa-rust mem <ref.fa> <reads.fq>`
- **THEN** the CLI SHALL build an FM-index from the FASTA reference in memory
- **AND** align the reads
- **AND** output SAM records to stdout unless `-o/--out` is provided

#### Scenario: Scope remains single-end

- **WHEN** a user needs paired-end alignment or BAM/CRAM output
- **THEN** the CLI SHALL NOT present those workflows as shipped `mem` behavior
- **AND** public documentation SHALL label them as planned until implemented

### Requirement: CLI Defaults Mirror Library Defaults

The CLI SHALL use `AlignOpt::default()` as the single source of truth for ordinary alignment defaults in both `align` and `mem`.

#### Scenario: Running align without tuning flags

- **WHEN** a user runs `bwa-rust align` without scoring, band, seed, occurrence, chain, alignment-count, thread, or zdrop overrides
- **THEN** the constructed alignment options SHALL match `AlignOpt::default()` for those fields

#### Scenario: Running mem without tuning flags

- **WHEN** a user runs `bwa-rust mem` without scoring, band, seed, occurrence, chain, alignment-count, thread, or zdrop overrides
- **THEN** the constructed alignment options SHALL match `AlignOpt::default()` for those fields
- **AND** named presets SHALL be the only documented mechanism that changes multiple defaults at once

### Requirement: Alignment Parameters

The system SHALL expose alignment tuning parameters consistently across `align` and `mem`.

#### Scenario: Configure scoring and extension

- **WHEN** a user specifies match, mismatch, gap, clip penalty, band width, score threshold, seed length, or zdrop options
- **THEN** the CLI SHALL pass those values into `AlignOpt`
- **AND** the alignment implementation SHALL use the configured values

#### Scenario: Configure memory and output limits

- **WHEN** a user specifies `--max-occ`, `--max-chains`, or `--max-alignments`
- **THEN** the CLI SHALL use those values for repetitive seed filtering, chain extraction, and per-read output limiting

### Requirement: SAM Output Format

The system SHALL output valid SAM with headers and required alignment fields.

#### Scenario: Generate SAM header

- **WHEN** outputting SAM
- **THEN** the output SHALL include `@HD`, one `@SQ` per contig, and `@PG`

#### Scenario: Format mapped alignment record

- **WHEN** a read has a valid alignment
- **THEN** the SAM record SHALL include all 11 required fields
- **AND** include `AS:i`, `XS:i`, `NM:i`, and available `MD:Z`/`SA:Z` tags

#### Scenario: Format unmapped read

- **WHEN** no candidate passes the score threshold
- **THEN** the output SHALL be an unmapped SAM record with FLAG `4`, RNAME `*`, and POS `0`

### Requirement: Error Handling

The system SHALL provide clear CLI errors for common user failures.

#### Scenario: Report invalid index

- **WHEN** an index file is corrupted, unsupported, or has the wrong magic/version
- **THEN** the CLI SHALL report the validation failure
- **AND** suggest rebuilding the index where applicable

#### Scenario: Report I/O errors

- **WHEN** file read or write fails
- **THEN** the CLI SHALL report the path and underlying system error

## Reference: Default Parameter Values

Both `align` and `mem` use `AlignOpt::default()` unless explicitly overridden or changed by a named preset.

| Parameter | Default Value |
|-----------|---------------|
| `match_score` | 2 |
| `mismatch_penalty` | 1 |
| `gap_open` | 2 |
| `gap_extend` | 1 |
| `clip_penalty` | 1 |
| `band_width` | 16 |
| `score_threshold` | 20 |
| `min_seed_len` | 19 |
| `threads` | 1 |
| `max_occ` | 500 |
| `max_chains_per_contig` | 5 |
| `max_alignments_per_read` | 5 |
| `zdrop` | 100 |

Source of truth: `src/align/mod.rs` `AlignOpt::default()`.
