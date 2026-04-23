---
## Why

Build FM-index from FASTA reference genome for efficient O(m) exact pattern matching and SMEM seeding.

## What Changes

- Implement FASTA parsing with multi-contig support
- Implement suffix array construction (doubling algorithm)
- Implement BWT construction from SA
- Implement FM-index with C-table and Occ sampling
- Implement backward search for pattern matching
- Serialize index to single `.fm` file format

## Capabilities

### New Capabilities

- `index-building`: FM-index construction from FASTA

### Modified Capabilities

None (initial implementation)

## Impact

- `src/io/fasta.rs` - FASTA parsing
- `src/index/sa.rs` - Suffix array construction
- `src/index/bwt.rs` - BWT construction
- `src/index/fm.rs` - FM-index core
- `src/index/builder.rs` - Build entry point
