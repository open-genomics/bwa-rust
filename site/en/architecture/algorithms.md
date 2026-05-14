# Core Algorithms

## FM-index

Index construction pipeline:

```mermaid
flowchart LR
    REF[Reference FASTA] --> NORM[Normalize]
    NORM --> SA[Suffix Array]
    SA --> BWT[BWT Transform]
    BWT --> FM[FM-index]
    FM --> SAVE[.fm File]
```

Project alphabet: `{0:$, 1:A, 2:C, 3:G, 4:T, 5:N}`.

The `.fm` file contains:

- magic/version;
- BWT;
- C table;
- Occ samples;
- SA or sparse SA;
- contig metadata;
- encoded reference text.

## SMEM Seeding

`seed.rs` uses FM-index backward search to find exact matches, controlling seed count via `min_seed_len` and `max_occ`. `max_occ` is crucial for repetitive regions, preventing highly repetitive seeds from overwhelming downstream chain building.

## Chain Building

`chain.rs` organizes seeds on the same contig with consistent orientation and reasonable coordinates into chains. Chain scoring prefers approximately collinear seeds, with maximum chain count limiting candidate explosion.

## Smith-Waterman Extension

`sw.rs` provides banded affine-gap SW, semi-global alignment, and chain-end extension. `extend.rs` converts chains to complete CIGAR:

- Left and right ends use configurable `zdrop` for extension termination;
- Intra-chain gaps are filled using global alignment, avoiding mid-chain indel clipping;
- Read bases unalignable at ends are represented as soft clips.

## SAM Tags

`pipeline.rs` sorts candidates, classifies primary/secondary/supplementary, and generates SAM records. `sam.rs` generates MD:Z; `supplementary.rs` generates SA:Z.
