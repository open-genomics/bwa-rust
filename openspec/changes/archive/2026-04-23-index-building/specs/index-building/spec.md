## ADDED Requirements

### Requirement: FASTA Parsing and Normalization

The system SHALL parse FASTA files and normalize sequences for index construction.

#### Scenario: Parse multi-contig FASTA

- **WHEN** a FASTA file with multiple contigs is parsed
- **THEN** extract all sequences with their names
- **AND** detect and reject duplicate contig names

### Requirement: Suffix Array Construction

The system SHALL construct suffix array using doubling algorithm.

#### Scenario: Build SA with O(n log²n) complexity

- **WHEN** a normalized sequence is provided
- **THEN** construct suffix array in O(n log²n) time
- **AND** use O(n) space for SA and rank arrays

### Requirement: FM-Index Structure

The system SHALL construct FM-index with C-table and Occ sampling.

#### Scenario: Build C-table

- **WHEN** BWT is constructed
- **THEN** compute cumulative character frequencies
- **AND** store as Vec<u32> of size sigma+1

### Requirement: Backward Search

The system SHALL support exact pattern matching via backward search.

#### Scenario: Search exact pattern

- **WHEN** a pattern is provided
- **THEN** return SA interval [l, r) for pattern occurrences
- **AND** complete in O(m) time where m is pattern length
