# Index Format

## Overview

bwa-rust uses a **single-file FM-index** format (`.fm`) that combines all index components into one portable file. This design simplifies deployment and archiving compared to BWA's multi-file approach.

## File Structure

The `.fm` file uses [bincode](https://docs.rs/bincode) serialization with the following layout:

```
┌─────────────────────────────────────────────────────────────┐
│  Header (Magic + Version)                                   │
├─────────────────────────────────────────────────────────────┤
│  Encoded Reference Text                                     │
│  (DNA alphabet: {0:$, 1:A, 2:C, 3:G, 4:T, 5:N})            │
├─────────────────────────────────────────────────────────────┤
│  Burrows-Wheeler Transform (BWT)                            │
├─────────────────────────────────────────────────────────────┤
│  C Table (cumulative character counts)                      │
├─────────────────────────────────────────────────────────────┤
│  Occ Samples (rank statistics at intervals)                 │
├─────────────────────────────────────────────────────────────┤
│  Sparse Suffix Array (sampled SA values)                    │
├─────────────────────────────────────────────────────────────┤
│  Contig Metadata (names, lengths, offsets)                  │
└─────────────────────────────────────────────────────────────┘
```

## Key Components

### Magic Number and Version

- **Magic**: `bwa-rust-fm-index` (identifies file type)
- **Version**: Semantic version for compatibility checking

### Encoded Reference Text

Reference sequences are concatenated and encoded using a 3-bit alphabet:
- `0`: Sentinel character `$` (sequence terminator)
- `1-4`: A, C, G, T
- `5`: N (ambiguous base)

### BWT (Burrows-Wheeler Transform)

The transformed text enables efficient backward search. Storage: O(n) bytes where n = reference length.

### C Table

Cumulative counts for each character in the alphabet. Enables O(1) rank queries.

### Occ Samples

Rank statistics stored at regular intervals (default: every 4 positions). This **sparse sampling** reduces memory by ~75% compared to storing full rank matrices.

**Trade-off**: O(interval) query time instead of O(1), but the constant is small and memory savings are significant.

### Sparse Suffix Array

Only every k-th SA value is stored (default: k=4). Missing values are reconstructed on demand using the BWT and Occ samples.

## Comparison with BWA

| Aspect | bwa-rust `.fm` | BWA |
|--------|---------------|-----|
| File count | 1 | 5+ (`.sa`, `.bwt`, `.pac`, `.ann`, `.amb`) |
| Deployment | Single file copy | Multiple file sync |
| Archive | Simple tar/zip | Must preserve all files |
| Portability | High | Medium |
| Index size | Similar | Similar |

## Usage

### Building an Index

```bash
bwa-rust index reference.fasta -o reference.fm
```

### Using the Index

```bash
bwa-rust align reference.fm reads.fastq -o output.sam
```

## Implementation Details

See [FM-index Builder](/en/architecture/algorithms#fm-index-construction) for the construction algorithm.

::: tip Memory Efficiency
The sparse sampling strategy makes bwa-rust suitable for environments with limited memory, while maintaining reasonable query performance.
:::
