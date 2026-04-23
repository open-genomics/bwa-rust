# Alignment Algorithm Specification

## Purpose

This specification defines the algorithmic details of SMEM seed finding, chain building, and Smith-Waterman alignment in bwa-rust.

## Requirements

### Requirement: SMEM Definition and Finding

The system SHALL find Super-Maximal Exact Matches defined as the longest exact match covering each position that is not contained by any other exact match.

#### Scenario: Extend right for longest match

- **WHEN** finding SMEM at position i
- **THEN** extend right from i to find longest match
- **AND** record match length and SA interval

#### Scenario: Extend left for maximality

- **WHEN** right extension completes
- **THEN** extend left until match is maximal
- **AND** update SA interval accordingly

#### Scenario: Skip covered regions

- **WHEN** a SMEM is found
- **THEN** skip to end of SMEM for next search
- **AND** allow overlapping SMEMs at different positions

### Requirement: Chain Scoring Model

The system SHALL score seed chains using dynamic programming with gap penalties.

#### Scenario: Score collinear seeds

- **WHEN** two seeds are collinear (same direction, consistent order)
- **THEN** apply linear gap penalty based on distance
- **AND** add seed lengths to chain score

#### Scenario: Penalize non-collinear seeds

- **WHEN** two seeds are non-collinear
- **THEN** apply heavy inconsistency penalty
- **AND** allow chain to continue if still viable

#### Scenario: Compute chain score

- **WHEN** evaluating a chain
- **THEN** sum seed lengths minus gap penalties
- **AND** compare against minimum score threshold

### Requirement: DP Chain Building Algorithm

The system SHALL build chains using dynamic programming and greedy peeling.

#### Scenario: Sort seeds by reference position

- **WHEN** building chains
- **THEN** sort seeds by reference position
- **AND** process in sorted order

#### Scenario: Find best chain via DP

- **WHEN** building a chain
- **THEN** use DP to find optimal seed combination
- **AND** mark used seeds to prevent reuse

#### Scenario: Extract multiple chains

- **WHEN** best chain is extracted
- **THEN** repeat with remaining seeds
- **AND** stop when chain score falls below threshold

### Requirement: Banded Smith-Waterman Implementation

The system SHALL implement banded affine-gap Smith-Waterman alignment.

#### Scenario: Initialize DP matrix

- **WHEN** starting SW alignment
- **THEN** allocate 2*band_width+1 columns per row
- **AND** initialize first row/column to 0

#### Scenario: Compute affine gap scores

- **WHEN** computing DP cell
- **THEN** consider match/mismatch, gap open, and gap extend
- **AND** use separate matrices for gap tracking

#### Scenario: Traceback for CIGAR

- **WHEN** DP computation completes
- **THEN** traceback from maximum score
- **AND** generate CIGAR string (M, I, D, S operations)

### Requirement: Semi-Global Refinement

The system SHALL perform semi-global refinement for edge cases.

#### Scenario: Refine partial alignments

- **WHEN** chain covers partial read
- **THEN** extend alignment to read boundaries
- **AND** apply soft clipping where appropriate

#### Scenario: Handle clipping penalty

- **WHEN** sorting candidates
- **THEN** apply clip penalty to alignment score
- **AND** prefer alignments with less clipping

### Requirement: Candidate Management

The system SHALL manage alignment candidates with deduplication and sorting.

#### Scenario: Deduplicate by position

- **WHEN** collecting candidates
- **THEN** deduplicate by (ref_id, ref_pos, is_reverse)
- **AND** keep best score for each position

#### Scenario: Sort by quality

- **WHEN** outputting alignments
- **THEN** sort by score minus clip penalty
- **AND** mark primary and secondary alignments

#### Scenario: Limit output count

- **WHEN** multiple alignments exist
- **THEN** limit to `max_alignments_per_read` (default: 5)
- **AND** mark excess as secondary

### Requirement: MAPQ Estimation

The system SHALL estimate Mapping Quality from alignment scores.

#### Scenario: Compute MAPQ from score difference

- **WHEN** primary and secondary alignments exist
- **THEN** estimate MAPQ from score difference
- **AND** cap MAPQ at maximum value (default: 60)

#### Scenario: Handle unique alignment

- **WHEN** only one alignment exists
- **THEN** assign high MAPQ (default: 60)
- **AND** adjust based on alignment score

## Why

These algorithmic specifications ensure bwa-rust produces accurate alignments comparable to BWA-MEM while maintaining predictable memory usage and performance characteristics.
