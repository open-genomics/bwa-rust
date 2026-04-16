# Core Algorithm Tutorial

> This tutorial explains bwa-rust's core algorithms: FM-index, SMEM seed finding, and Smith-Waterman alignment.

---

## Table of Contents

- [FM-Index Principles](#fm-index-principles)
- [SMEM Seed Finding](#smem-seed-finding)
- [Seed Chain Building](#seed-chain-building)
- [Smith-Waterman Alignment](#smith-waterman-alignment)
- [Complete Alignment Pipeline](#complete-alignment-pipeline)

---

## FM-Index Principles

### What is FM-Index?

The FM-index (Ferragini-Manzini Index) is a full-text index data structure based on BWT (Burrows-Wheeler Transform):

- **Efficient substring search**: O(m) time to find pattern of length m
- **Memory efficient**: Reduced space via sampling
- **Reversible**: Supports recovering original text from BWT

### Construction Pipeline

```
Original text: ACGTACGT$
    │
    ▼
┌─────────────────────────────┐
│ 1. Suffix Array             │
│    Sort all suffixes         │
│    [8, 0, 4, 1, 5, 2, 6, 3, 7]│
└─────────────────────────────┘
    │
    ▼
┌─────────────────────────────┐
│ 2. BWT Transform            │
│    Take preceding char       │
│    BWT = "T$TCAGAGC"         │
└─────────────────────────────┘
    │
    ▼
┌─────────────────────────────┐
│ 3. C-table + Occ table      │
│    C: starting positions     │
│    Occ: cumulative counts    │
└─────────────────────────────┘
```

### Backward Search

Core operation of FM-index, searching from right to left:

```rust
// Search pattern "CGT"
// Step 1: Start with 'T'
SA_range = [C['T'], C['T']+1) = [6, 8)

// Step 2: Extend with 'G'
new_l = C['G'] + Occ('G', l) = 3 + 1 = 4
new_r = C['G'] + Occ('G', r) = 3 + 2 = 5
SA_range = [4, 5)

// Step 3: Extend with 'C'
final_l = C['C'] + Occ('C', 4) = 1 + 1 = 2
final_r = C['C'] + Occ('C', 5) = 1 + 2 = 3
SA_range = [2, 3)  // Found 1 match

// Match position = SA[2] = 4
// "CGT" is at position 4 in original text
```

**Time complexity: O(m)** — Independent of sequence length!

### Code Example

```rust
use bwa_rust::index::{sa, bwt, fm};
use bwa_rust::util::dna;

fn main() {
    let reference = b"ACGTACGT";
    
    // Encode
    let norm = dna::normalize_seq(reference);
    let mut text: Vec<u8> = norm.iter().map(|&b| dna::to_alphabet(b)).collect();
    text.push(0);  // sentinel
    
    // Build index
    let sa_arr = sa::build_sa(&text);
    let bwt_arr = bwt::build_bwt(&text, &sa_arr);
    let contigs = vec![fm::Contig { name: "ref".into(), len: 8, offset: 0 }];
    let fm_idx = fm::FMIndex::build(text, bwt_arr, sa_arr, contigs, 6, 4);
    
    // Search
    let pattern: Vec<u8> = b"CGT".iter().map(|&b| dna::to_alphabet(b)).collect();
    if let Some((l, r)) = fm_idx.backward_search(&pattern) {
        println!("'CGT' occurs {} times", r - l);
        
        for pos in fm_idx.sa_interval_positions(l, r) {
            println!("  At position: {}", pos);
        }
    }
}
```

---

## SMEM Seed Finding

### What is SMEM?

**SMEM** (Super-Maximal Exact Match) is the core concept of BWA-MEM:

> For each position in the read, find the **longest** exact match covering that position, which is not contained by any other exact match.

### Algorithm Steps

```
Read: ACGTACGTACGT

1. Start at position 0, extend right to find longest match
   "ACGTACGT" matches reference at position 1000
   → Try extending left, cannot extend further
   
   Seed 1: read[0:8] → ref[1000:1008]

2. Jump to position 4, repeat process
   "ACGTACGT" matches reference at position 2000
   
   Seed 2: read[4:12] → ref[2000:2008]

Note: Seeds may overlap, but each is maximal at its position.
```

### Memory Protection

For repetitive sequences (e.g., poly-A), use `max_occ` limit:

```rust
// Only keep seeds with ≤ 500 occurrences
let seeds = find_smem_seeds_with_max_occ(&fm_idx, &read, min_len, 500);
```

---

## Seed Chain Building

### Goal

Combine multiple seeds into a "chain" representing where the read likely aligns.

### Scoring Formula

```
Chain score = Σ(seed lengths) - Gap penalties

Gap penalty rules:
- Collinear seeds (reasonable distance): linear penalty
- Non-collinear seeds: heavy penalty
```

### DP Algorithm

```rust
// Sort seeds by reference position
seeds.sort_by_key(|s| s.rb);

// Dynamic programming for best chain
for i in 0..seeds.len() {
    best_chain[i] = seeds[i].len;  // single seed
    
    for j in 0..i {
        if collinear(seeds[j], seeds[i]) {
            let candidate = best_chain[j] + gap_score(seeds[j], seeds[i]);
            if candidate > best_chain[i] {
                best_chain[i] = candidate;
                prev[i] = j;
            }
        }
    }
}

// Backtrack to find chain
reconstruct_chain(prev, argmax(best_chain));
```

---

## Smith-Waterman Alignment

### Problem

Perform local alignment in chain gaps to handle:
- Mismatches
- Insertions/deletions (indels)
- Soft clipping

### Banded Optimization

Standard SW requires O(m×n) time and space. Banded optimization:

```
Standard matrix (m×n):
┌────────────┐
│ ██████████ │  Time/Space: O(m×n)
│ ██████████ │
│ ██████████ │
└────────────┘

Banded matrix (m×w):
┌────────────┐
│     ██     │  Time/Space: O(m×w)
│    ████    │  w = band width (default 16)
│     ██     │
└────────────┘
```

### DP Formulas

```
M[i][j] = match score
X[i][j] = vertical gap score (insertion)
Y[i][j] = horizontal gap score (deletion)

M[i][j] = max(
    M[i-1][j-1] + match_score(query[i], ref[j]),
    X[i-1][j-1] + match_score(query[i], ref[j]),
    Y[i-1][j-1] + match_score(query[i], ref[j]),
    0  // SW allows local alignment
)

X[i][j] = max(
    M[i-1][j] - gap_open,
    X[i-1][j] - gap_extend
)

Y[i][j] = max(
    M[i][j-1] - gap_open,
    Y[i][j-1] - gap_extend
)
```

### Complexity Comparison

| Implementation | Time | Space | 100bp × 1000bp |
|----------------|------|-------|----------------|
| Standard SW | O(m×n) | O(m×n) | 100K cells |
| Banded SW | O(m×w) | O(m×w) | 1.6K cells (w=16) |

**98%+ savings**

---

## Complete Alignment Pipeline

```
┌──────────────────────────────────────────────────────────┐
│                    Input: FASTQ Read                      │
└──────────────────────────────────────────────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        ▼                   ▼                   ▼
   ┌─────────┐        ┌─────────┐         ┌─────────┐
   │ Forward │        │ Forward │         │ Reverse │
   │ Sequence│        │  Seeds  │         │ Seeds   │
   └────┬────┘        └────┬────┘         └────┬────┘
        │                   │                   │
        │              ┌────┴────┐         ┌────┴────┐
        │              │ Chains  │         │ Chains  │
        │              └────┬────┘         └────┬────┘
        │                   │                   │
        │              ┌────┴────┐         ┌────┴────┐
        │              │   SW    │         │   SW    │
        │              └────┬────┘         └────┬────┘
        │                   │                   │
        └───────────────────┼───────────────────┘
                            ▼
              ┌─────────────────────┐
              │   Merge & Dedup     │
              └──────────┬──────────┘
                         ▼
              ┌─────────────────────┐
              │ Sort by Quality     │
              └──────────┬──────────┘
                         ▼
              ┌─────────────────────┐
              │ Select Primary/Sec  │
              └──────────┬──────────┘
                         ▼
              ┌─────────────────────┐
              │   Output: SAM      │
              └─────────────────────┘
```

### AlignOpt Configuration

```rust
use bwa_rust::align::AlignOpt;

let opt = AlignOpt {
    // Scoring parameters
    match_score: 2,
    mismatch_penalty: 1,
    gap_open: 2,
    gap_extend: 1,
    clip_penalty: 1,
    
    // Alignment parameters
    band_width: 16,
    score_threshold: 20,
    min_seed_len: 19,
    
    // Parallel parameters
    threads: 4,
    
    // Memory protection
    max_occ: 500,
    max_chains_per_contig: 5,
    max_alignments_per_read: 5,
};

opt.validate().expect("Invalid AlignOpt");
```

---

## Related Documentation

- [Getting Started](./getting-started.md) — Installation and basic usage
- [Architecture Overview](../architecture/overview.md) — Module design
- [Index Building](../architecture/index-building.md) — Index construction details
- [Alignment Algorithms](../architecture/alignment.md) — Complete alignment workflow
