# Index Building Specification

## Purpose

The index building capability constructs FM-index from FASTA reference genome for efficient sequence alignment queries.

## Requirements

### Requirement: FASTA Parsing and Normalization

The system SHALL parse FASTA files and normalize sequences for index construction.

#### Scenario: Parse multi-contig FASTA

- **WHEN** a FASTA file with multiple contigs is parsed
- **THEN** extract all sequences with their names
- **AND** detect and reject duplicate contig names

#### Scenario: Normalize sequences

- **WHEN** sequences are processed
- **THEN** convert to uppercase
- **AND** filter non-standard characters to N
- **AND** reject empty sequences with error

### Requirement: Suffix Array Construction

The system SHALL construct suffix array using doubling algorithm.

#### Scenario: Build SA with O(n log²n) complexity

- **WHEN** a normalized sequence is provided
- **THEN** construct suffix array in O(n log²n) time
- **AND** use O(n) space for SA and rank arrays

#### Scenario: Handle DNA alphabet

- **WHEN** building SA for DNA sequences
- **THEN** use alphabet size 6 ($, A, C, G, T, N)
- **AND** encode: $→0, A→1, C→2, G→3, T→4, N→5

### Requirement: BWT Construction

The system SHALL generate BWT from suffix array.

#### Scenario: Build BWT from SA

- **WHEN** suffix array is available
- **THEN** generate BWT using BWT[i] = text[(SA[i] - 1) mod n]
- **AND** complete in O(n) time

### Requirement: FM-Index Structure

The system SHALL construct FM-index with C-table and Occ sampling.

#### Scenario: Build C-table

- **WHEN** BWT is constructed
- **THEN** compute cumulative character frequencies
- **AND** store as Vec<u32> of size sigma+1

#### Scenario: Build Occ sampling table

- **WHEN** BWT is constructed
- **THEN** sample occurrences at block intervals (default: 64)
- **AND** achieve O(1) query with O(block) scan

### Requirement: Backward Search

The system SHALL support exact pattern matching via backward search.

#### Scenario: Search exact pattern

- **WHEN** a pattern is provided
- **THEN** return SA interval [l, r) for pattern occurrences
- **AND** complete in O(m) time where m is pattern length

#### Scenario: Handle no-match case

- **WHEN** pattern has no exact matches in reference
- **THEN** return None
- **AND** handle gracefully without errors

### Requirement: Index Serialization

The system SHALL serialize index to single `.fm` file using bincode.

#### Scenario: Serialize complete index

- **WHEN** index is built
- **THEN** write to single `.fm` file with magic header `BWAFM_RS`
- **AND** include version number (current: 2)

#### Scenario: Store index components

- **WHEN** serializing index
- **THEN** include: C-table, BWT, Occ samples, sparse SA, contig metadata, original text
- **AND** include optional build metadata (version, timestamp, stats)

### Requirement: Sparse SA Sampling

The system SHALL use sparse SA sampling to reduce memory.

#### Scenario: Sample SA at intervals

- **WHEN** building index
- **THEN** store SA values at sample rate (default: 4)
- **AND** compute missing values via LF-mapping backtrack

#### Scenario: Query SA value

- **WHEN** SA value at position i is requested
- **THEN** return stored value if at sample position
- **AND** backtrack via LF-mapping otherwise

### Requirement: Memory Estimation

The system SHALL provide memory estimates for index size.

#### Scenario: Estimate memory for genome

- **WHEN** building index for reference of length n
- **THEN** estimate total memory as ~2.4n bytes
- **AND** report estimated memory before construction

## Why

FM-index enables O(m) exact pattern matching and efficient approximate alignment seeding, making it the foundation of BWA-MEM style aligners. The single-file format simplifies deployment compared to BWA's multi-file format.
