# Paired-End Alignment Specification

## Purpose

Define the paired-end (PE) read alignment capability with insert size estimation, mate rescue, and proper pairing logic.

## Requirements

### Requirement: Paired-End FASTQ Input

The system SHALL support paired-end FASTQ input files (R1/R2 pairs).

#### Scenario: Parse separate FASTQ files

- **WHEN** two FASTQ files are provided (reads_1.fq, reads_2.fq)
- **THEN** parse both files as paired reads
- **AND** validate read name pairing

#### Scenario: Validate read pairing

- **WHEN** parsing paired FASTQ files
- **THEN** verify read count matches between R1 and R2
- **AND** report error on mismatch

#### Scenario: Support interleaved format

- **WHEN** `-p` option is specified
- **THEN** parse interleaved FASTQ format
- **AND** treat adjacent records as R1/R2 pairs

### Requirement: Insert Size Estimation

The system SHALL estimate insert size distribution from properly paired alignments.

#### Scenario: Collect insert sizes

- **WHEN** alignments are processed
- **THEN** collect insert sizes from properly paired alignments
- **AND** calculate median and MAD

#### Scenario: Apply insert size constraint

- **WHEN** pairing mates
- **THEN** use median + 3*MAD as maximum insert size
- **AND** default to 500 bp before estimation

### Requirement: Pairing Logic

The system SHALL implement proper pairing rules for PE alignments.

#### Scenario: Find best pair

- **WHEN** both mates have multiple alignments
- **THEN** find best pair within insert size constraints
- **AND** prefer FR (forward-reverse) orientation

#### Scenario: Calculate pairing score

- **WHEN** evaluating alignment pairs
- **THEN** combine scores from both mates
- **AND** apply penalty for improper orientation

### Requirement: Mate Rescue

The system SHALL rescue unmapped mates by searching near the mapped mate.

#### Scenario: Detect unmapped mate

- **WHEN** one mate is unmapped but the other is mapped
- **THEN** search for unmapped mate within rescue window

#### Scenario: Perform rescue alignment

- **WHEN** rescue is attempted
- **THEN** use relaxed alignment parameters
- **AND** mark rescued alignment with appropriate FLAG

### Requirement: PE SAM Output

The system SHALL format paired-end alignments with proper SAM flags.

#### Scenario: Set paired flags

- **WHEN** outputting PE alignments
- **THEN** set FLAG bits for paired, proper pair, first/second in pair
- **AND** set mate information (RNEXT, PNEXT, TLEN)

#### Scenario: Handle improper pairs

- **WHEN** pair is not properly oriented or sized
- **THEN** clear proper pair flag (bit 0x2)
- **AND** still output both mates

## Why

Paired-end alignment is essential for most modern sequencing applications, providing positional information that improves mapping accuracy and enables structural variant detection.
