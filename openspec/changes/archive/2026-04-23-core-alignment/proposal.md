---
## Why

Implement BWA-MEM style DNA short-read alignment using FM-index based seeding, chaining, and Smith-Waterman extension. This is the core functionality of bwa-rust.

## What Changes

- Implement SMEM (Super-Maximal Exact Match) seed finding algorithm
- Implement seed chain building with DP scoring
- Implement banded Smith-Waterman alignment
- Implement candidate management and deduplication
- Implement MAPQ estimation
- Support multi-threading via rayon
- Output SAM format alignment results

## Capabilities

### New Capabilities

- `alignment`: Core alignment capability with SMEM, chaining, SW, and SAM output

### Modified Capabilities

None (initial implementation)

## Impact

- `src/align/` module - All alignment algorithms
- `src/io/sam.rs` - SAM output formatting
- `src/main.rs` - CLI integration
