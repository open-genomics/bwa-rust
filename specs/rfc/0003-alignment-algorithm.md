# RFC-0003: Alignment Algorithm

> **Status**: Accepted
> **Created**: 2026-02-13
> **Updated**: 2026-04-16
> **Version**: v0.2.0

## Summary

This document specifies the bwa-rust alignment algorithm workflow, including SMEM seed finding, chain building, Smith-Waterman alignment, and the complete pipeline.

## Overall Workflow

```
Read (FASTQ)
    │
    ▼
┌─────────────────────────────────────┐
│ 1. Sequence Normalization           │
│    • DNA encoding (A/C/G/T/N)       │
│    • Reverse complement             │
└─────────────────────────────────────┘
    │
    ├─ Forward ────────┐
    │                  │
    ├─ Rev-comp ───────┤
    │                  │
    ▼                  ▼
┌─────────────────────────────────────┐
│ 2. SMEM Seed Finding                │
│    • Maximal exact matches          │
│    • Left extension until no longer │
│    • max_occ filtering              │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│ 3. Seed Chain Building              │
│    • DP scoring                     │
│    • Greedy peeling for multi-chain │
│    • Low-score chain filtering      │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│ 4. Chain Extension Alignment        │
│    • Reference window extraction    │
│    • Banded Smith-Waterman          │
│    • CIGAR generation               │
│    • Semi-global refinement         │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│ 5. Candidate Management             │
│    • Position/direction dedup       │
│    • Score-based sorting            │
│    • Primary/secondary marking      │
└─────────────────────────────────────┘
    │
    ▼
Output (SAM)
```

## SMEM Seed Finding

### Definition

SMEM (Super-Maximal Exact Match) is defined as:
> For each position in the read, find the **longest** exact match covering that position, which is not contained by any other exact match.

### Algorithm

```rust
pub fn find_smem_seeds(
    fm: &FMIndex,
    read: &[u8],
    min_len: usize,
    max_occ: usize,
) -> Vec<MemSeed> {
    let mut seeds = Vec::new();
    let mut i = 0;

    while i < read.len() {
        // From position i, extend right to find longest match
        let (seed_len, sa_interval) = extend_right(fm, &read[i..]);

        if seed_len >= min_len && sa_interval.occ() <= max_occ {
            // Try to extend left
            let (left_ext, final_interval) = extend_left(
                fm, &read[..i], sa_interval
            );

            let seed = MemSeed {
                qb: i - left_ext,     // query begin
                qe: i + seed_len,     // query end
                sa_l: final_interval.l,
                sa_r: final_interval.r,
                occ: final_interval.occ(),
            };

            seeds.push(seed);
            i += seed_len;  // Skip covered region
        } else {
            i += 1;
        }
    }

    seeds
}
```

### Example

Read: `ACGTACGTACGT`

```
Forward strand:
Position: 0  1  2  3  4  5  6  7  8  9  10 11
Read:     A  C  G  T  A  C  G  T  A  C  G  T
          └──────── SMEM 1 ──────┘
Seed 1: read[0:8] → ref[1000:1008], occ=1
                         └──── SMEM 2 ────┘
Seed 2: read[4:12] → ref[2000:2008], occ=2

Seeds may overlap, but each is maximal at its position.
```

### Memory Guard: max_occ

```rust
// Filter seeds with too many occurrences (e.g., repetitive sequences)
const DEFAULT_MAX_OCC: usize = 500;

if seed.occ > max_occ {
    skip!();  // Prevent poly-A etc. from exploding
}
```

## Seed Chain Building

### Goal

Combine multiple seeds into a coherent "chain" representing where the read likely aligns to the reference.

### Scoring Model

```rust
// Chain score calculation
fn chain_score(seeds: &[MemSeed]) -> i32 {
    let mut score = 0i32;

    for window in seeds.windows(2) {
        let (s1, s2) = (&window[0], &window[1]);

        // Seed score
        score += s1.len() as i32;

        // Gap penalty
        let gap_cost = if collinear(s1, s2) {
            // Collinear: linear penalty based on distance
            gap_penalty_linear(s1, s2)
        } else {
            // Non-collinear: heavy penalty
            GAP_PENALTY_INCONSISTENT
        };

        score -= gap_cost;
    }

    score
}
```

### DP Chain Building

```rust
pub fn build_chains(seeds: &[MemSeed], read_len: usize) -> Vec<Chain> {
    let mut chains = Vec::new();
    let mut used = vec![false; seeds.len()];

    // Sort seeds by reference position
    let mut ordered: Vec<_> = seeds.iter().enumerate().collect();
    ordered.sort_by_key(|(_, s)| s.rb);

    // Greedy extraction of best chains
    while !used.iter().all(|&x| x) {
        // For each unused seed, DP to find best chain
        let best_chain = dp_find_best_chain(&ordered, &used);

        if best_chain.score < MIN_CHAIN_SCORE {
            break;
        }

        // Mark used
        for &idx in &best_chain.seed_indices {
            used[idx] = true;
        }

        chains.push(best_chain);
    }

    chains
}
```

### Chain Filtering

```rust
pub fn filter_chains(chains: &mut Vec<Chain>, ratio: f32) {
    if chains.is_empty() { return; }

    // Sort by score descending
    chains.sort_by_key(|c| -c.score);

    let best_score = chains[0].score;
    let threshold = (best_score as f32 * ratio) as i32;

    // Filter low-score chains
    chains.retain(|c| c.score >= threshold);
}
```

## Smith-Waterman Alignment

### Banded Implementation

Standard Smith-Waterman requires O(m×n) space and time. bwa-rust uses banded optimization:

```rust
pub struct SwParams {
    pub match_score: i32,       // default 2
    pub mismatch_penalty: i32,  // default 1
    pub gap_open: i32,          // default 2
    pub gap_extend: i32,        // default 1
    pub band_width: usize,      // default 16
}

pub fn banded_sw(
    query: &[u8],
    reference: &[u8],
    params: SwParams,
) -> SwResult {
    let w = params.band_width;
    let mut dp = vec![vec![0i32; 2 * w + 1]; query.len() + 1];

    for i in 1..=query.len() {
        // Only compute cells in band
        let center = best_col_from_prev(&dp[i-1]);
        let col_start = center.saturating_sub(w);
        let col_end = (center + w).min(reference.len());

        for j in col_start..=col_end {
            // Affine gap DP
            let match_score = if query[i-1] == reference[j-1] {
                params.match_score
            } else {
                -params.mismatch_penalty
            };

            dp[i][j - col_start] = max(
                dp[i-1][j - col_start - 1] + match_score,  // match/mismatch
                dp[i-1][j - col_start] - params.gap_extend,  // vertical gap
                dp[i][j - col_start - 1] - params.gap_extend,  // horizontal gap
                0,  // Smith-Waterman: allow local alignment
            );
        }
    }

    // Backtrack to generate CIGAR
    let cigar = traceback(&dp);
    let score = dp.iter().flatten().max().copied().unwrap_or(0);

    SwResult { score, cigar }
}
```

### Complexity

| Implementation | Time | Space |
|----------------|------|-------|
| Standard SW | O(m×n) | O(m×n) |
| Banded SW | O(m×w) | O(m×w) |

Where w is band width (default 16), achieving 99%+ savings for long sequences.

## Candidate Management

### Workflow

```rust
pub fn collect_candidates(
    chains: &[Chain],
    align_opt: &AlignOpt,
) -> Vec<Candidate> {
    let mut candidates = Vec::new();

    for chain in chains {
        // 1. Extract reference window
        let ref_window = extract_ref_window(chain, &align_opt);

        // 2. Perform SW alignment
        let sw_result = banded_sw(&chain.query, &ref_window, align_opt.sw_params);

        // 3. Filter low-score alignments
        if sw_result.score < align_opt.score_threshold {
            continue;
        }

        // 4. Semi-global refinement
        let refined = semi_global_refinement(
            &chain.query, &ref_window, sw_result
        );

        candidates.push(Candidate {
            ref_id: chain.ref_id,
            ref_pos: chain.ref_pos,
            score: refined.score,
            cigar: refined.cigar,
            nm: refined.nm,
            is_reverse: chain.is_reverse,
        });
    }

    candidates
}
```

### Deduplication and Sorting

```rust
pub fn dedup_candidates(candidates: &mut Vec<Candidate>) {
    // Deduplicate by (ref_id, ref_pos, is_reverse)
    candidates.sort_by_key(|c| (c.ref_id, c.ref_pos, c.is_reverse));
    candidates.dedup_by_key(|c| (c.ref_id, c.ref_pos, c.is_reverse));

    // Sort by alignment quality (score - clip_penalty)
    candidates.sort_by(|a, b| {
        let score_a = a.score - a.clipped_bases;
        let score_b = b.score - b.clipped_bases;
        score_b.cmp(&score_a)
    });
}
```

## Configuration: AlignOpt

```rust
pub struct AlignOpt {
    // Scoring parameters
    pub match_score: i32,
    pub mismatch_penalty: i32,
    pub gap_open: i32,
    pub gap_extend: i32,
    pub clip_penalty: i32,

    // Alignment parameters
    pub band_width: usize,
    pub score_threshold: i32,
    pub min_seed_len: usize,

    // Parallel parameters
    pub threads: usize,

    // Memory protection
    pub max_occ: usize,
    pub max_chains_per_contig: usize,
    pub max_alignments_per_read: usize,
}

impl AlignOpt {
    pub fn validate(&self) -> Result<(), BwaError> {
        // Validate parameter values
        if self.band_width == 0 {
            return Err(BwaError::Align("band_width must be > 0".into()));
        }
        // ... other validations
        Ok(())
    }
}
```

## Pipeline Entry

```rust
pub fn align_reads(
    fm: &FMIndex,
    reads: &[FastqRecord],
    opt: &AlignOpt,
) -> Vec<SamRecord> {
    let pool = ThreadPoolBuilder::new()
        .num_threads(opt.threads)
        .build()
        .expect("Failed to create thread pool");

    pool.install(|| {
        reads.par_iter().map(|read| {
            align_single_read(fm, read, opt)
        }).collect()
    })
}

fn align_single_read(
    fm: &FMIndex,
    read: &FastqRecord,
    opt: &AlignOpt,
) -> SamRecord {
    // Forward
    let forward_seeds = find_smem_seeds(fm, &read.seq, opt.min_seed_len, opt.max_occ);
    let forward_chains = build_chains(&forward_seeds, read.seq.len());
    let forward_candidates = collect_candidates(&forward_chains, opt);

    // Reverse complement
    let revcomp_seq = revcomp(&read.seq);
    let reverse_seeds = find_smem_seeds(fm, &revcomp_seq, opt.min_seed_len, opt.max_occ);
    let reverse_chains = build_chains(&reverse_seeds, read.seq.len());
    let mut reverse_candidates = collect_candidates(&reverse_chains, opt);
    reverse_candidates.iter_mut().for_each(|c| c.is_reverse = true);

    // Merge, dedup, sort
    let mut all = forward_candidates;
    all.extend(reverse_candidates);
    dedup_candidates(&mut all);

    // Select primary and secondary
    select_primary_and_secondary(&all, opt.max_alignments_per_read)
}
```

## Memory Protection

bwa-rust implements three-level memory protection to prevent memory explosion from repetitive sequences (e.g., poly-A):

```
┌────────────────────────────────────────────────────────────┐
│                  Three-Level Memory Protection              │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ 1. max_occ = 500                                     │  │
│  │    Skip seeds with SA interval > 500                  │  │
│  │    Prevent poly-A etc. from expanding to 10K+ positions│ │
│  └──────────────────────────────────────────────────────┘  │
│                         ↓                                  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ 2. max_chains_per_contig = 5                         │  │
│  │    Max 5 chains per contig                           │  │
│  │    Prevent repetitive regions from generating        │  │
│  │    too many candidates                               │  │
│  └──────────────────────────────────────────────────────┘  │
│                         ↓                                  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ 3. max_alignments_per_read = 5                       │  │
│  │    Max 5 alignments per read                         │  │
│  │    Control final output size                         │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

## Performance Optimizations

### Key Optimizations

| Optimization | Method | Effect |
|--------------|--------|--------|
| Buffer reuse | `SwBuffer` pool | Reduce hot-path allocations |
| Batch processing | Batch seeds by chromosome | Improve cache hit rate |
| Parallelization | Rayon parallel iterator | Near-linear multi-core speedup |
| Early filtering | max_occ + chain score threshold | Reduce wasted computation |

### Performance Benchmark

On small test dataset (toy.fa, 1K reads):

| Operation | Single-thread | 4 threads | 8 threads |
|-----------|---------------|-----------|-----------|
| Index build | 0.1s | - | - |
| Alignment | 2.1s | 0.6s | 0.4s |
