# Alignment Specification

## Purpose

The alignment capability performs BWA-MEM style DNA short-read alignment using FM-index based seeding, chaining, and Smith-Waterman extension.

## Requirements

### Requirement: FM-Index Sequence Alignment

The system SHALL build FM-index from FASTA reference genome for sequence alignment.

#### Scenario: Build index from valid FASTA

- **WHEN** a valid FASTA file is provided to the index command
- **THEN** create a `.fm` index file with magic header `BWAFM_RS`
- **AND** the index SHALL contain C-table, BWT, and sparse SA samples

#### Scenario: Handle multi-contig reference

- **WHEN** a FASTA file contains multiple contigs
- **THEN** concatenate sequences with `$` separators
- **AND** store contig metadata (name, length, offset) in index

### Requirement: SMEM Seed Finding

The system SHALL find Super-Maximal Exact Matches (SMEMs) for seeding alignment.

#### Scenario: Find longest exact match

- **WHEN** a read sequence is processed for seeding
- **THEN** find the longest exact match covering each position
- **AND** extend left until the match is no longer maximal

#### Scenario: Filter high-occurrence seeds

- **WHEN** a seed has occurrence count exceeding `max_occ` threshold (default: 500)
- **THEN** skip the seed to prevent memory explosion
- **AND** log the filtered seed count

### Requirement: Seed Chain Building

The system SHALL combine multiple seeds into coherent chains using dynamic programming.

#### Scenario: Build chains with DP scoring

- **WHEN** multiple seeds are found for a read
- **THEN** score chains using DP with gap penalties
- **AND** extract top chains via greedy peeling

#### Scenario: Filter low-score chains

- **WHEN** chains are scored
- **THEN** filter chains below score threshold
- **AND** limit chains per contig via `max_chains_per_contig` (default: 5)

### Requirement: Banded Smith-Waterman Alignment

The system SHALL perform banded affine-gap local alignment for chain extension.

#### Scenario: Align with banded SW

- **WHEN** a chain is selected for extension
- **THEN** perform banded Smith-Waterman alignment
- **AND** use configurable band width (default: 16)

#### Scenario: Generate CIGAR and NM tag

- **WHEN** SW alignment completes
- **THEN** generate CIGAR string from alignment path
- **AND** compute NM tag (edit distance)

### Requirement: SAM Output

The system SHALL output alignment results in SAM format.

#### Scenario: Generate SAM header

- **WHEN** alignment output begins
- **THEN** generate `@HD` header line (VN:1.6, SO:unsorted)
- **AND** generate `@SQ` lines for each contig

#### Scenario: Format alignment records

- **WHEN** an alignment is output
- **THEN** include all required SAM fields
- **AND** include optional tags: AS:i, XS:i, NM:i

### Requirement: Multi-Threading Support

The system SHALL support parallel alignment processing using rayon.

#### Scenario: Configure thread count

- **WHEN** `-t` option is specified
- **THEN** use specified number of threads
- **AND** achieve near-linear speedup on multi-core systems

### Requirement: Memory Protection

The system SHALL prevent memory explosion from repetitive sequences.

#### Scenario: Apply three-level protection

- **WHEN** processing repetitive sequences (e.g., poly-A)
- **THEN** apply `max_occ` (default: 500) for seed filtering
- **AND** apply `max_chains_per_contig` (default: 5) for chain limiting
- **AND** apply `max_alignments_per_read` (default: 5) for output limiting

## Why

This capability is the core of bwa-rust, enabling accurate and efficient DNA short-read alignment similar to BWA-MEM while maintaining memory safety and modern Rust practices.
