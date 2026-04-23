## ADDED Requirements

### Requirement: Index Command

The system SHALL provide an `index` subcommand for building FM-index.

#### Scenario: Build index from FASTA

- **WHEN** user runs `bwa-rust index <ref.fa> -o <prefix>`
- **THEN** build FM-index from reference FASTA
- **AND** output `<prefix>.fm` file

### Requirement: Align Command

The system SHALL provide an `align` subcommand for aligning reads to existing index.

#### Scenario: Align FASTQ to index

- **WHEN** user runs `bwa-rust align -i <index.fm> <reads.fq>`
- **THEN** align reads to pre-built index
- **AND** output SAM to stdout

### Requirement: Mem Command

The system SHALL provide a `mem` subcommand for one-step index and align (BWA-MEM style).

#### Scenario: One-step alignment

- **WHEN** user runs `bwa-rust mem <ref.fa> <reads.fq>`
- **THEN** build index if not exists
- **AND** align reads to index
- **AND** output SAM to stdout
