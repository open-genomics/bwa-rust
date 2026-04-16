# Index Building Details

> This document details the FM-index construction process, including suffix array construction, BWT transformation, FM-index structure, and serialization.

---

## Table of Contents

- [Overview](#overview)
- [Data Flow](#data-flow)
- [Suffix Array Construction](#suffix-array-construction)
- [BWT Construction](#bwt-construction)
- [FM-Index Construction](#fm-index-construction)
- [Memory Optimizations](#memory-optimizations)
- [Index File Format](#index-file-format)
- [Performance Analysis](#performance-analysis)

---

## Overview

The index building pipeline in bwa-rust:

```
FASTA file
    │
    ▼
┌─────────────────┐
│ Sequence norm    │ ← Uppercase, filter non-standard chars
│ Multi-contig     │ ← Add $ separator between contigs
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ Suffix Array     │ ← O(n log²n) doubling algorithm
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ BWT transform    │ ← Generate BWT from SA
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ FM-index build   │ ← C-table + Occ sampling
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ Serialization    │ ← bincode encoding → .fm file
└─────────────────┘
```

---

## Data Flow

### 1. FASTA Parsing

**Pipeline:**
```rust
// 1. Read FASTA file
let fasta = parse_fasta(file)?;

// 2. Sequence normalization
text = normalize_seq(seq);  // A/C/G/T/N, uppercase

// 3. Alphabet encoding
for byte in text {
    encoded.push(to_alphabet(byte));  // A→1, C→2, G→3, T→4, N→5
}

// 4. Add sentinel
encoded.push(0);  // $ → 0
```

### 2. Input Validation

| Check | Strategy |
|-------|----------|
| Empty sequence | Reject with error |
| Duplicate contig names | Reject with error |
| Non-standard characters | Convert to N (5) |
| Different line endings | Auto-handle LF/CRLF |

---

## Suffix Array Construction

### Doubling Algorithm

**Principle:** Sort suffixes by increasingly longer prefixes.

```rust
// Round k: Sort by first 2^k characters
fn doubling_sort(k: usize, sa: &mut [usize], rank: &[i32]) {
    // Use (rank[i], rank[i+k]) as sort key
    sa.sort_by_key(|&i| (rank[i], rank.get(i + k).copied().unwrap_or(-1)));
}

// Full algorithm
pub fn build_sa(text: &[u8]) -> Vec<usize> {
    let n = text.len();
    let mut sa: Vec<usize> = (0..n).collect();
    let mut rank: Vec<i32> = text.iter().map(|&c| c as i32).collect();
    
    let mut k = 1;
    loop {
        // Sort by (rank[i], rank[i+k])
        sa.sort_by_key(|&i| {
            let r1 = rank[i];
            let r2 = if i + k < n { rank[i + k] } else { -1 };
            (r1, r2)
        });
        
        // Update ranks
        let mut new_rank = vec![0i32; n];
        new_rank[sa[0]] = 0;
        for i in 1..n {
            new_rank[sa[i]] = new_rank[sa[i-1]] 
                + if is_different(&sa, i, k, &rank) { 1 } else { 0 };
        }
        
        rank = new_rank;
        if rank[sa[n-1]] == (n - 1) as i32 { break; }
        k *= 2;
    }
    
    sa
}
```

**Complexity Analysis:**
- Time: O(n log²n) — O(n log n) sort per round, O(log n) rounds
- Space: O(n) — SA and rank arrays

### Example

Text: `"banana$"` (n=7)

| Round (k) | Key Length | SA State |
|-----------|------------|----------|
| 1 | 2 | [6, 5, 3, 1, 0, 4, 2] |
| 2 | 4 | [6, 5, 3, 1, 0, 4, 2] |
| 4 | 8 | [6, 5, 3, 1, 0, 4, 2] ✓ |

Final SA = [6, 5, 3, 1, 0, 4, 2] corresponds to suffixes:
- SA[0]=6 → "$"
- SA[1]=5 → "a$"
- SA[2]=3 → "ana$"
- SA[3]=1 → "anana$"
- SA[4]=0 → "banana$"
- SA[5]=4 → "na$"
- SA[6]=2 → "nana$"

---

## BWT Construction

### Algorithm

Burrows-Wheeler Transform from SA:

```
BWT[i] = text[(SA[i] - 1) mod n]
```

**Rust implementation:**
```rust
pub fn build_bwt(text: &[u8], sa: &[usize]) -> Vec<u8> {
    let n = text.len();
    let mut bwt = Vec::with_capacity(n);
    
    for &sa_i in sa {
        let pos = if sa_i == 0 { n - 1 } else { sa_i - 1 };
        bwt.push(text[pos]);
    }
    
    bwt
}
```

### Example

Text: `"abracadabra$"` (n=12)

| i | SA[i] | BWT[i] = text[(SA[i]-1)%n] | Suffix |
|---|-------|---------------------------|--------|
| 0 | 11 | $ | "abracadabra$" |
| 1 | 10 | a | "$" |
| 2 | 7 | a | "abra$" |
| 3 | 0 | $ | "abracadabra$" |
| 4 | 3 | a | "acadabra$" |
| 5 | 5 | c | "adabra$" |
| 6 | 8 | d | "abra$" |
| 7 | 1 | b | "bracadabra$" |
| 8 | 4 | a | "cadabra$" |
| 9 | 6 | a | "dabra$" |
| 10 | 9 | r | "ra$" |
| 11 | 2 | r | "racadabra$" |

BWT = "$aa$acdbraara"`

### FM-Index Construction

The FM-index consists of two main components:
- **C-table**: Records the starting position of each character in BWT
- **Occ-table**: Records the cumulative count of each character in BWT prefix

#### C-Table

```rust
// C[c] = first position of character c in BWT
pub fn build_c(bwt: &[u8], sigma: usize) -> Vec<u32> {
    let mut count = vec![0u32; sigma];
    
    // Count character frequencies
    for &c in bwt {
        count[c as usize] += 1;
    }
    
    // Compute cumulative frequencies
    let mut c = vec![0u32; sigma + 1];
    for i in 0..sigma {
        c[i + 1] = c[i] + count[i];
    }
    
    c  // c[0]=0, c[1]=count($), c[2]=count($)+count(A), ...
}
```

#### Occ Sampling Table

To reduce memory usage, use block-level sampling:

```rust
pub struct FMIndex {
    block: u32,            // Sampling interval (default 64)
    occ_samples: Vec<u32>, // Cumulative counts at sample points
}

impl FMIndex {
    // Compute Occ(c, pos) = count of char c in BWT[0..pos]
    pub fn occ(&self, c: u8, pos: usize) -> u32 {
        let block_idx = pos / self.block as usize;
        let sample_idx = block_idx * self.sigma as usize + c as usize;
        let base = self.occ_samples[sample_idx];
        
        // Count from last sample point
        let start = block_idx * self.block as usize;
        let count = self.bwt[start..pos]
            .iter()
            .filter(|&&b| b == c)
            .count() as u32;
        
        base + count
    }
}
```

**Space optimization comparison:**

| Method | Space | Query Time |
|--------|-------|------------|
| Full Occ table | O(n × σ) | O(1) |
| Sampled Occ (rate=64) | O(n × σ / 64) | O(64) ≈ O(1) |

For DNA sequences (σ=6), sampling at rate 64 reduces memory by ~94%.

#### Backward Search

The core operation of FM-index:

```rust
pub fn backward_search(&self, pattern: &[u8]) -> Option<(usize, usize)> {
    let mut l: usize = 0;
    let mut r: usize = self.bwt.len();
    
    for &c in pattern.iter().rev() {
        l = self.c[c as usize] as usize + self.occ(c, l) as usize;
        r = self.c[c as usize] as usize + self.occ(c, r) as usize;
        
        if l >= r {
            return None;  // No match
        }
    }
    
    Some((l, r))  // SA interval [l, r)
}
```

**Time complexity: O(m)** where m is pattern length

---

## Memory Optimizations

### 1. Sparse SA Sampling

```rust
pub struct FMIndex {
    sa: Vec<u32>,          // Only store sample points
    sa_sample_rate: u32,   // Sampling interval (default 4)
}

impl FMIndex {
    // Get SA value at any position
    pub fn sa(&self, i: usize) -> Option<u32> {
        if i % self.sa_sample_rate as usize == 0 {
            // Direct sample value
            Some(self.sa[i / self.sa_sample_rate as usize])
        } else {
            // Backtrack via LF-mapping
            self.lf_mapping_backtrack(i)
        }
    }
}
```

**Memory savings:**
- Full SA: n × 4 bytes
- Sparse SA (rate=4): n × 1 byte (75% reduction)

### 2. Occ Block Sampling

- Block size: 64 (configurable)
- Store u32 count per character
- Memory: n × 6 × 4 / 64 ≈ n × 0.375 bytes

### 3. Total Memory Estimate

For reference sequence of length n:

| Component | Memory Usage |
|-----------|--------------|
| BWT | n bytes |
| C-table | 28 bytes (7 × u32) |
| Occ samples | n × 0.375 bytes |
| SA samples | n bytes |
| Text | n bytes |
| Contig metadata | ~KB level |
| **Total** | **≈ 2.4n bytes** |

For human genome (≈3G bases): ~7.2 GB (raw FASTA ~3 GB)

---

## Index File Format

### File Structure

```
.magic: [u8; 8]       = b"BWAFM_RS" (0x424D4146_4D5F5253)
.version: u32         = 2

.sigma: u8            = 6 ($, A, C, G, T, N)
.block: u32           = 64

.c: Vec<u32>          = length σ+1 = 7
.bwt: Vec<u8>         = length n
.occ_samples: Vec<u32> = length (n/block) × σ

.sa: Vec<u32>         = length n/sa_sample_rate
.sa_sample_rate: u32  = 4

.contigs: Vec<Contig>  # contig metadata
.text: Vec<u8>         # original encoded text (optional)

.meta: Option<IndexMeta>  # build metadata
```

### Contig Structure

```rust
pub struct Contig {
    pub name: String,   // chromosome name
    pub len: u32,       // length
    pub offset: u64,    // start position in concatenated sequence
}
```

### IndexMeta Structure

```rust
pub struct IndexMeta {
    pub version: String,
    pub build_time: String,  // ISO 8601 format
    pub sequences: u32,      // number of contigs
    pub total_bases: u64,    // total base count
}
```

---

## Performance Analysis

### Build Time

| Operation | Complexity | Typical Time (100M bp) |
|-----------|------------|----------------------|
| FASTA parsing | O(n) | ~1s |
| SA construction | O(n log²n) | ~30s |
| BWT construction | O(n) | ~1s |
| FM-index construction | O(n) | ~2s |
| Serialization | O(n) | ~3s |
| **Total** | - | **~37s** |

### Query Performance

| Operation | Complexity | Typical Time |
|-----------|------------|--------------|
| Backward Search | O(m) | <1µs (100bp read) |
| SA position query | O(rate) | <1µs |
| Full alignment | O(read_len + ref_window) | 0.1-1ms |

### Comparison with BWA

| Metric | bwa-rust | BWA (C) |
|--------|----------|---------|
| SA algorithm | Doubling O(n log²n) | SA-IS O(n) |
| Build time | Slower | 3-5x faster |
| Index size | Similar | Similar |
| Query speed | Similar | Similar |

---

## Related Documentation

- [Architecture Overview](./overview.md) — Module architecture overview
- [Alignment Algorithm Details](./alignment.md) — Complete alignment workflow
- [Getting Started Tutorial](../tutorial/getting-started.md) — Usage guide
