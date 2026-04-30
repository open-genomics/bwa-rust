# Paired-End Alignment Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement BWA-MEM compatible paired-end alignment with proper pair scoring, mate rescue, insert size estimation, and correct SAM paired flags.

**Architecture:** Extend existing single-end pipeline by adding a new `pairing.rs` module for pair scoring and mate rescue logic. The `mem` CLI command is extended to accept optional R2 file or `-p` flag for interleaved input. Reuse existing `PairedFastqReader` and `InsertSizeStats` infrastructure.

**Tech Stack:** Rust 2021, Rayon for parallelism, existing bwa-rust alignment pipeline

---

## File Structure

```
src/align/
├── pairing.rs (new)      - Pair scoring, mate rescue, proper pair detection
├── pipeline.rs (modify)  - Add align_paired_fastq, align_read_pair
├── insert_size.rs (existing) - Already implemented
├── mod.rs (modify)       - Export pairing module

src/io/
├── sam.rs (modify)       - Paired SAM output, FLAG helpers

src/main.rs (modify)      - Extend mem command

tests/
├── integration.rs (modify) - Paired-end e2e tests

openspec/specs/cli/spec.md (modify) - CLI spec update
```

---

## Phase 1: Core Pairing Logic

### Task 1: Create pairing.rs with PairScore and PairedAlignment structs

**Files:**
- Create: `src/align/pairing.rs`

- [ ] **Step 1: Create pairing.rs with core data structures**

```rust
use crate::align::{AlignCandidate, AlignOpt, PairingOpt};
use crate::align::insert_size::InsertSizeStats;
use crate::index::fm::FMIndex;
use crate::align::sw::SwParams;

/// Result of paired-end alignment for a read pair
#[derive(Debug, Clone)]
pub struct PairedAlignment {
    pub r1: Option<AlignCandidate>,
    pub r2: Option<AlignCandidate>,
    pub is_proper_pair: bool,
    pub insert_size: i32,
    pub pair_score: i32,
}

impl PairedAlignment {
    pub fn new() -> Self {
        Self {
            r1: None,
            r2: None,
            is_proper_pair: false,
            insert_size: 0,
            pair_score: 0,
        }
    }
}

/// Score for a potential pairing of two alignments
#[derive(Debug, Clone, Copy)]
pub struct PairScore {
    pub score: i32,
    pub is_proper: bool,
    pub insert_size: i32,
}

impl Default for PairedAlignment {
    fn default() -> Self {
        Self::new()
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add src/align/pairing.rs
git commit -m "feat(align): add PairedAlignment and PairScore structs"
```

---

### Task 2: Implement FR orientation detection

**Files:**
- Modify: `src/align/pairing.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::align::AlignCandidate;

    fn make_candidate(contig_idx: usize, pos1: i32, is_rev: bool, score: i32) -> AlignCandidate {
        AlignCandidate {
            contig_idx,
            pos1,
            is_rev,
            score,
            cigar: "100M".to_string(),
            rname: format!("chr{}", contig_idx),
            ref_seq: vec![b'A'; 100],
            query_seq: vec![b'A'; 100],
            nm: 0,
            sort_score: score,
        }
    }

    #[test]
    fn is_fr_orientation_forward_reverse() {
        let r1 = make_candidate(0, 100, false, 50);
        let r2 = make_candidate(0, 200, true, 50);
        assert!(is_fr_orientation(&r1, &r2));
    }

    #[test]
    fn is_fr_orientation_reverse_forward() {
        let r1 = make_candidate(0, 200, true, 50);
        let r2 = make_candidate(0, 100, false, 50);
        assert!(!is_fr_orientation(&r1, &r2));
    }

    #[test]
    fn is_fr_orientation_same_strand() {
        let r1 = make_candidate(0, 100, false, 50);
        let r2 = make_candidate(0, 200, false, 50);
        assert!(!is_fr_orientation(&r1, &r2));
    }

    #[test]
    fn is_fr_orientation_different_contig() {
        let r1 = make_candidate(0, 100, false, 50);
        let r2 = make_candidate(1, 200, true, 50);
        assert!(!is_fr_orientation(&r1, &r2));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test is_fr_orientation -- --nocapture`
Expected: Compilation error (function not defined)

- [ ] **Step 3: Implement is_fr_orientation**

```rust
/// Check if two alignments are in FR (forward-reverse) orientation
/// FR: R1 forward at lower position, R2 reverse at higher position on same contig
pub fn is_fr_orientation(r1: &AlignCandidate, r2: &AlignCandidate) -> bool {
    if r1.contig_idx != r2.contig_idx {
        return false;
    }
    
    !r1.is_rev && r2.is_rev && r1.pos1 < r2.pos1
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test is_fr_orientation -- --nocapture`
Expected: All 4 tests pass

- [ ] **Step 5: Commit**

```bash
git add src/align/pairing.rs
git commit -m "feat(align): add is_fr_orientation for pair orientation detection"
```

---

### Task 3: Implement insert size calculation

**Files:**
- Modify: `src/align/pairing.rs`

- [ ] **Step 1: Write the failing test**

Add to the tests module:

```rust
    #[test]
    fn calculate_insert_size_fr_orientation() {
        let r1 = make_candidate(0, 100, false, 50);
        let mut r2 = make_candidate(0, 200, true, 50);
        r2.cigar = "50M".to_string();
        let insert = calculate_insert_size(&r1, &r2);
        assert_eq!(insert, 150);
    }

    #[test]
    fn calculate_insert_size_negative_for_rf() {
        let r1 = make_candidate(0, 200, true, 50);
        let mut r2 = make_candidate(0, 100, false, 50);
        r2.cigar = "50M".to_string();
        let insert = calculate_insert_size(&r1, &r2);
        assert!(insert < 0);
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test calculate_insert_size -- --nocapture`
Expected: Compilation error

- [ ] **Step 3: Implement calculate_insert_size**

```rust
/// Calculate insert size (TLEN) for a pair of alignments
/// Returns positive if R1 is leftmost, negative if R2 is leftmost
pub fn calculate_insert_size(r1: &AlignCandidate, r2: &AlignCandidate) -> i32 {
    let r1_end = r1.pos1 + parse_cigar_length(&r1.cigar) as i32;
    let r2_end = r2.pos1 + parse_cigar_length(&r2.cigar) as i32;
    
    if r1.pos1 <= r2.pos1 {
        r2_end - r1.pos1
    } else {
        r1_end - r2.pos1
    }
}

fn parse_cigar_length(cigar: &str) -> usize {
    let mut len = 0;
    let mut num = 0;
    for c in cigar.chars() {
        if c.is_ascii_digit() {
            num = num * 10 + c.to_digit(10).unwrap() as usize;
        } else {
            match c {
                'M' | 'I' | 'D' | 'N' | '=' | 'X' => len += num,
                _ => {}
            }
            num = 0;
        }
    }
    len
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test calculate_insert_size -- --nocapture`
Expected: All 2 tests pass

- [ ] **Step 5: Commit**

```bash
git add src/align/pairing.rs
git commit -m "feat(align): add calculate_insert_size for TLEN calculation"
```

---

### Task 4: Implement proper pair detection

**Files:**
- Modify: `src/align/pairing.rs`

- [ ] **Step 1: Write the failing test**

Add to tests module:

```rust
    #[test]
    fn is_proper_pair_valid() {
        let mut stats = InsertSizeStats::new(500);
        for i in 100..200 {
            stats.add_sample(i);
        }
        
        let r1 = make_candidate(0, 100, false, 50);
        let r2 = make_candidate(0, 200, true, 50);
        assert!(is_proper_pair(&r1, &r2, &stats));
    }

    #[test]
    fn is_proper_pair_wrong_orientation() {
        let stats = InsertSizeStats::new(500);
        
        let r1 = make_candidate(0, 100, false, 50);
        let r2 = make_candidate(0, 200, false, 50);
        assert!(!is_proper_pair(&r1, &r2, &stats));
    }

    #[test]
    fn is_proper_pair_insert_too_large() {
        let stats = InsertSizeStats::new(500);
        
        let r1 = make_candidate(0, 100, false, 50);
        let r2 = make_candidate(0, 1000, true, 50);
        assert!(!is_proper_pair(&r1, &r2, &stats));
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test is_proper_pair -- --nocapture`
Expected: Compilation error

- [ ] **Step 3: Implement is_proper_pair**

```rust
/// Check if a pair is a "proper pair" (FR orientation, valid insert size)
pub fn is_proper_pair(
    r1: &AlignCandidate,
    r2: &AlignCandidate,
    insert_stats: &InsertSizeStats,
) -> bool {
    if !is_fr_orientation(r1, r2) {
        return false;
    }
    
    let insert_size = calculate_insert_size(r1, r2);
    insert_stats.is_valid_insert(insert_size.abs())
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test is_proper_pair -- --nocapture`
Expected: All 3 tests pass

- [ ] **Step 5: Commit**

```bash
git add src/align/pairing.rs
git commit -m "feat(align): add is_proper_pair for proper pair detection"
```

---

### Task 5: Implement pair scoring

**Files:**
- Modify: `src/align/pairing.rs`

- [ ] **Step 1: Write the failing test**

Add to tests module:

```rust
    #[test]
    fn score_pair_fr_no_penalty() {
        let mut stats = InsertSizeStats::new(500);
        for i in 100..200 {
            stats.add_sample(i);
        }
        let opt = PairingOpt::default();
        
        let r1 = make_candidate(0, 100, false, 60);
        let r2 = make_candidate(0, 200, true, 50);
        let score = score_pair(&r1, &r2, &stats, &opt);
        
        assert_eq!(score.score, 110);
        assert!(score.is_proper);
    }

    #[test]
    fn score_pair_rf_with_penalty() {
        let stats = InsertSizeStats::new(500);
        let opt = PairingOpt::default();
        
        let r1 = make_candidate(0, 200, true, 60);
        let r2 = make_candidate(0, 100, false, 50);
        let score = score_pair(&r1, &r2, &stats, &opt);
        
        assert!(score.score < 110);
        assert!(!score.is_proper);
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test score_pair -- --nocapture`
Expected: Compilation error

- [ ] **Step 3: Implement score_pair**

```rust
/// Score a potential pairing of two alignments
pub fn score_pair(
    r1: &AlignCandidate,
    r2: &AlignCandidate,
    insert_stats: &InsertSizeStats,
    pairing_opt: &PairingOpt,
) -> PairScore {
    let is_proper = is_proper_pair(r1, r2, insert_stats);
    let insert_size = calculate_insert_size(r1, r2);
    
    let mut score = r1.score + r2.score;
    
    if !is_proper {
        score -= pairing_opt.pen_unpaired;
    }
    
    let deviation_penalty = insert_stats.insert_size_deviation_penalty(insert_size.abs());
    score -= deviation_penalty;
    
    PairScore {
        score,
        is_proper,
        insert_size,
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test score_pair -- --nocapture`
Expected: All 2 tests pass

- [ ] **Step 5: Commit**

```bash
git add src/align/pairing.rs
git commit -m "feat(align): add score_pair for pair scoring"
```

---

### Task 6: Implement mate rescue

**Files:**
- Modify: `src/align/pairing.rs`

- [ ] **Step 1: Write the failing test**

Add to tests module:

```rust
    use crate::testutil::build_test_fm;
    use crate::align::sw::SwParams;
    use crate::align::AlignOpt;

    #[test]
    fn rescue_mate_success() {
        let fm = build_test_fm(b"ACGTACGTACGTACGTACGTACGTACGTACGTACGT");
        let mapped = make_candidate(0, 5, false, 50);
        let unmapped_seq = b"ACGTACGT";
        
        let sw_params = SwParams::default();
        let opt = AlignOpt::default();
        
        let result = rescue_mate(&fm, &mapped, unmapped_seq, sw_params, &opt);
        assert!(result.is_some());
    }

    #[test]
    fn rescue_mate_far_away() {
        let fm = build_test_fm(b"ACGTACGTACGT");
        let mapped = make_candidate(0, 1000, false, 50);
        let unmapped_seq = b"ACGTACGT";
        
        let sw_params = SwParams::default();
        let opt = AlignOpt::default();
        
        let result = rescue_mate(&fm, &mapped, unmapped_seq, sw_params, &opt);
        assert!(result.is_none());
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test rescue_mate -- --nocapture`
Expected: Compilation error

- [ ] **Step 3: Implement rescue_mate**

```rust
/// Attempt to rescue an unmapped mate near a mapped mate
pub fn rescue_mate(
    fm: &FMIndex,
    mapped_mate: &AlignCandidate,
    unmapped_seq: &[u8],
    sw_params: SwParams,
    opt: &AlignOpt,
) -> Option<AlignCandidate> {
    let rescue_window = 500;
    let contig = &fm.contigs[mapped_mate.contig_idx];
    
    let contig_start = contig.offset as i32;
    let contig_end = (contig.offset + contig.len) as i32;
    
    let search_start = (mapped_mate.pos1 - rescue_window).max(contig_start) as usize;
    let search_end = (mapped_mate.pos1 + rescue_window).min(contig_end) as usize;
    
    if search_end <= search_start {
        return None;
    }
    
    let ref_seq: Vec<u8> = fm.text[search_start..search_end].to_vec();
    
    let relaxed_threshold = opt.score_threshold / 2;
    
    if unmapped_seq.len() < opt.min_seed_len {
        return None;
    }
    
    use crate::align::sw::banded_sw;
    let result = banded_sw(
        &ref_seq,
        unmapped_seq,
        sw_params.match_score,
        sw_params.mismatch_penalty,
        sw_params.gap_open,
        sw_params.gap_extend,
        sw_params.band_width,
    );
    
    if result.score >= relaxed_threshold {
        Some(AlignCandidate {
            contig_idx: mapped_mate.contig_idx,
            pos1: search_start as i32 + result.ref_start as i32 + 1,
            is_rev: false,
            score: result.score,
            cigar: result.cigar.clone(),
            rname: mapped_mate.rname.clone(),
            ref_seq: result.ref_seq,
            query_seq: result.query_seq,
            nm: result.edit_distance,
            sort_score: result.score,
        })
    } else {
        None
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test rescue_mate -- --nocapture`
Expected: All 2 tests pass

- [ ] **Step 5: Commit**

```bash
git add src/align/pairing.rs
git commit -m "feat(align): add rescue_mate for mate rescue"
```

---

### Task 7: Export pairing module from mod.rs

**Files:**
- Modify: `src/align/mod.rs`

- [ ] **Step 1: Add pairing module and exports**

Add after line 1:

```rust
pub mod pairing;
```

Add after line 18:

```rust
pub use pairing::{calculate_insert_size, is_fr_orientation, is_proper_pair, PairScore, PairedAlignment, rescue_mate, score_pair};
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check`
Expected: No errors

- [ ] **Step 3: Commit**

```bash
git add src/align/mod.rs
git commit -m "feat(align): export pairing module and public API"
```

---

## Phase 2: SAM Paired Output

### Task 8: Add paired FLAG helper functions to sam.rs

**Files:**
- Modify: `src/io/sam.rs`

- [ ] **Step 1: Write the failing test**

Add to tests module in sam.rs:

```rust
    #[test]
    fn calculate_paired_flag_r1_mapped_r2_mapped_proper() {
        let flag = calculate_paired_flag(true, true, false, false, true, true);
        assert_eq!(flag, 0x1 | 0x2 | 0x40);
    }

    #[test]
    fn calculate_paired_flag_r1_mapped_r2_unmapped() {
        let flag = calculate_paired_flag(true, false, false, false, false, true);
        assert_eq!(flag, 0x1 | 0x8 | 0x40);
    }

    #[test]
    fn calculate_paired_flag_r1_unmapped_r2_mapped() {
        let flag = calculate_paired_flag(false, true, false, false, false, false);
        assert_eq!(flag, 0x1 | 0x4 | 0x80);
    }

    #[test]
    fn calculate_paired_flag_r1_reverse_r2_forward() {
        let flag = calculate_paired_flag(true, true, true, false, false, true);
        assert_eq!(flag, 0x1 | 0x10 | 0x20 | 0x40);
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test calculate_paired_flag -- --nocapture`
Expected: Compilation error

- [ ] **Step 3: Implement calculate_paired_flag**

Add after existing flag constants:

```rust
pub const FLAG_PAIRED: u16 = 0x1;
pub const FLAG_PROPER_PAIR: u16 = 0x2;
pub const FLAG_UNMAPPED: u16 = 0x4;
pub const FLAG_MATE_UNMAPPED: u16 = 0x8;
pub const FLAG_REVERSE: u16 = 0x10;
pub const FLAG_MATE_REVERSE: u16 = 0x20;
pub const FLAG_FIRST_IN_PAIR: u16 = 0x40;
pub const FLAG_SECOND_IN_PAIR: u16 = 0x80;
pub const FLAG_SECONDARY: u16 = 0x100;
pub const FLAG_SUPPLEMENTARY: u16 = 0x800;

/// Calculate FLAG for paired-end alignment
pub fn calculate_paired_flag(
    r1_mapped: bool,
    r2_mapped: bool,
    r1_reverse: bool,
    r2_reverse: bool,
    is_proper: bool,
    is_r1: bool,
) -> u16 {
    let mut flag = FLAG_PAIRED;
    
    if is_proper {
        flag |= FLAG_PROPER_PAIR;
    }
    
    if !r1_mapped {
        flag |= FLAG_UNMAPPED;
    }
    if !r2_mapped {
        flag |= FLAG_MATE_UNMAPPED;
    }
    
    if r1_reverse {
        flag |= FLAG_REVERSE;
    }
    if r2_reverse {
        flag |= FLAG_MATE_REVERSE;
    }
    
    if is_r1 {
        flag |= FLAG_FIRST_IN_PAIR;
    } else {
        flag |= FLAG_SECOND_IN_PAIR;
    }
    
    flag
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test calculate_paired_flag -- --nocapture`
Expected: All 4 tests pass

- [ ] **Step 5: Commit**

```bash
git add src/io/sam.rs
git commit -m "feat(io): add calculate_paired_flag for FLAG calculation"
```

---

### Task 9: Add format_paired_records function

**Files:**
- Modify: `src/io/sam.rs`

- [ ] **Step 1: Write the failing test**

Add to tests module:

```rust
    #[test]
    fn format_paired_records_basic() {
        let (line1, line2) = format_paired_records(
            "read1",
            "chr1", 100, "50M", false,
            "chr1", 200, "50M", true,
            "ACGT", "IIII",
            "TGCA", "IIII",
            true, 150,
            60, 60,
            50, 50,
            0, 0,
            "", "",
        );
        
        assert!(line1.contains("read1"));
        assert!(line1.contains("chr1"));
        assert!(line2.contains("read1"));
        assert!(line2.contains("="));
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test format_paired_records -- --nocapture`
Expected: Compilation error

- [ ] **Step 3: Implement format_paired_records**

```rust
/// Format a pair of alignments as SAM records
pub fn format_paired_records(
    qname: &str,
    r1_rname: &str, r1_pos: i32, r1_cigar: &str, r1_reverse: bool,
    r2_rname: &str, r2_pos: i32, r2_cigar: &str, r2_reverse: bool,
    seq1: &str, qual1: &str,
    seq2: &str, qual2: &str,
    is_proper: bool,
    insert_size: i32,
    mapq1: u8, mapq2: u8,
    score1: i32, score2: i32,
    nm1: i32, nm2: i32,
    md1: &str, md2: &str,
) -> (String, String) {
    let r1_mapped = r1_cigar != "*" && !r1_cigar.is_empty();
    let r2_mapped = r2_cigar != "*" && !r2_cigar.is_empty();
    
    let flag1 = calculate_paired_flag(r1_mapped, r2_mapped, r1_reverse, r2_reverse, is_proper, true);
    let flag2 = calculate_paired_flag(r2_mapped, r1_mapped, r2_reverse, r1_reverse, is_proper, false);
    
    let rnext1 = if r1_rname == r2_rname { "=" } else { r2_rname };
    let rnext2 = if r1_rname == r2_rname { "=" } else { r1_rname };
    
    let pnext1 = if r2_mapped { r2_pos } else { 0 };
    let pnext2 = if r1_mapped { r1_pos } else { 0 };
    
    let tlen1 = insert_size;
    let tlen2 = -insert_size;
    
    let line1 = format_record_core(
        qname, flag1, r1_rname, r1_pos, mapq1, r1_cigar,
        rnext1, pnext1, tlen1, seq1, qual1,
        score1, score2, nm1, md1,
    );
    
    let line2 = format_record_core(
        qname, flag2, r2_rname, r2_pos, mapq2, r2_cigar,
        rnext2, pnext2, tlen2, seq2, qual2,
        score2, score1, nm2, md2,
    );
    
    (line1, line2)
}

fn format_record_core(
    qname: &str, flag: u16, rname: &str, pos: i32, mapq: u8, cigar: &str,
    rnext: &str, pnext: i32, tlen: i32, seq: &str, qual: &str,
    score: i32, sub_score: i32, nm: i32, md: &str,
) -> String {
    let mut fields = vec![
        qname.to_string(),
        flag.to_string(),
        rname.to_string(),
        pos.to_string(),
        mapq.to_string(),
        cigar.to_string(),
        rnext.to_string(),
        pnext.to_string(),
        tlen.to_string(),
        seq.to_string(),
        qual.to_string(),
    ];
    
    fields.push(format!("AS:i:{}", score));
    fields.push(format!("XS:i:{}", sub_score));
    fields.push(format!("NM:i:{}", nm));
    
    if !md.is_empty() {
        fields.push(format!("MD:Z:{}", md));
    }
    
    fields.join("\t")
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test format_paired_records -- --nocapture`
Expected: Test passes

- [ ] **Step 5: Commit**

```bash
git add src/io/sam.rs
git commit -m "feat(io): add format_paired_records for paired SAM output"
```

---

## Phase 3: Pipeline Integration

### Task 10: Add align_read_pair function to pipeline.rs

**Files:**
- Modify: `src/align/pipeline.rs`

- [ ] **Step 1: Add necessary imports**

Add at top of file after existing imports:

```rust
use crate::io::fastq::{PairedFastqReader, ReadPair};
use crate::align::{PairingOpt, PairScore, PairedAlignment};
use crate::align::pairing::{is_fr_orientation, calculate_insert_size, is_proper_pair, score_pair, rescue_mate};
use std::sync::Mutex;
```

- [ ] **Step 2: Implement align_read_pair**

Add after `align_single_read` function:

```rust
pub(crate) fn align_read_pair(
    fm: &FMIndex,
    pair: &ReadPair,
    sw_params: SwParams,
    align_opt: &AlignOpt,
    pairing_opt: &PairingOpt,
    insert_stats: &mut InsertSizeStats,
) -> (Vec<String>, Vec<String>) {
    let r1_candidates = align_single_read_collect(fm, &pair.seq1, sw_params, align_opt);
    let r2_candidates = align_single_read_collect(fm, &pair.seq2, sw_params, align_opt);
    
    let paired_aln = find_best_pairing(
        fm,
        r1_candidates,
        r2_candidates,
        sw_params,
        align_opt,
        pairing_opt,
        insert_stats,
    );
    
    if paired_aln.is_proper_pair {
        insert_stats.add_sample(paired_aln.insert_size.abs());
    }
    
    let (lines1, lines2) = format_paired_output(
        &pair.name,
        &paired_aln,
        &pair.seq1, &pair.qual1,
        &pair.seq2, &pair.qual2,
        align_opt,
    );
    
    (lines1, lines2)
}

fn align_single_read_collect(
    fm: &FMIndex,
    seq: &[u8],
    sw_params: SwParams,
    opt: &AlignOpt,
) -> Vec<AlignCandidate> {
    use crate::util::dna;
    
    if seq.is_empty() {
        return vec![];
    }
    
    let fwd_norm = dna::normalize_seq(seq);
    let fwd_alpha: Vec<u8> = fwd_norm.iter().map(|&b| dna::to_alphabet(b)).collect();
    let rc_seq = dna::revcomp(seq);
    let rev_norm = dna::normalize_seq(&rc_seq);
    let rev_alpha: Vec<u8> = rev_norm.iter().map(|&b| dna::to_alphabet(b)).collect();
    
    let mut candidates = Vec::new();
    let query_len = seq.len();
    
    collect_candidates(fm, &fwd_norm, &fwd_alpha, sw_params, false, query_len, opt, &mut candidates);
    collect_candidates(fm, &rev_norm, &rev_alpha, sw_params, true, query_len, opt, &mut candidates);
    
    candidates.sort_by(|a, b| b.sort_score.cmp(&a.sort_score));
    dedup_candidates(&mut candidates);
    
    candidates
}

fn find_best_pairing(
    fm: &FMIndex,
    r1_candidates: Vec<AlignCandidate>,
    r2_candidates: Vec<AlignCandidate>,
    sw_params: SwParams,
    align_opt: &AlignOpt,
    pairing_opt: &PairingOpt,
    insert_stats: &InsertSizeStats,
) -> PairedAlignment {
    let mut best = PairedAlignment::new();
    let mut best_score = i32::MIN;
    
    if !r1_candidates.is_empty() && !r2_candidates.is_empty() {
        for r1 in &r1_candidates {
            for r2 in &r2_candidates {
                let pair_score = score_pair(r1, r2, insert_stats, pairing_opt);
                if pair_score.score > best_score {
                    best_score = pair_score.score;
                    best.r1 = Some(r1.clone());
                    best.r2 = Some(r2.clone());
                    best.is_proper_pair = pair_score.is_proper;
                    best.insert_size = pair_score.insert_size;
                    best.pair_score = pair_score.score;
                }
            }
        }
    } else if !r1_candidates.is_empty() && r2_candidates.is_empty() && pairing_opt.mate_rescue {
        if let Some(r1) = r1_candidates.first() {
            if let Some(r2) = rescue_mate(fm, r1, &r1_candidates[0].query_seq, sw_params, align_opt) {
                best.r1 = Some(r1.clone());
                best.r2 = Some(r2);
                best.pair_score = r1.score;
            } else {
                best.r1 = Some(r1.clone());
            }
        }
    } else if r1_candidates.is_empty() && !r2_candidates.is_empty() && pairing_opt.mate_rescue {
        if let Some(r2) = r2_candidates.first() {
            if let Some(r1) = rescue_mate(fm, r2, &r2_candidates[0].query_seq, sw_params, align_opt) {
                best.r1 = Some(r1);
                best.r2 = Some(r2.clone());
                best.pair_score = r2.score;
            } else {
                best.r2 = Some(r2.clone());
            }
        }
    }
    
    best
}

fn format_paired_output(
    qname: &str,
    paired_aln: &PairedAlignment,
    seq1: &[u8], qual1: &[u8],
    seq2: &[u8], qual2: &[u8],
    opt: &AlignOpt,
) -> (Vec<String>, Vec<String>) {
    let seq1_str = std::str::from_utf8(seq1).unwrap_or("");
    let qual1_str = std::str::from_utf8(qual1).unwrap_or("");
    let seq2_str = std::str::from_utf8(seq2).unwrap_or("");
    let qual2_str = std::str::from_utf8(qual2).unwrap_or("");
    
    match (&paired_aln.r1, &paired_aln.r2) {
        (Some(r1), Some(r2)) => {
            let (line1, line2) = sam::format_paired_records(
                qname,
                &r1.rname, r1.pos1, &r1.cigar, r1.is_rev,
                &r2.rname, r2.pos1, &r2.cigar, r2.is_rev,
                seq1_str, qual1_str,
                seq2_str, qual2_str,
                paired_aln.is_proper_pair,
                paired_aln.insert_size,
                compute_mapq(r1.sort_score, 0),
                compute_mapq(r2.sort_score, 0),
                r1.score, r2.score,
                r1.nm, r2.nm,
                "", "",
            );
            (vec![line1], vec![line2])
        }
        (Some(r1), None) => {
            let mut flag = sam::FLAG_PAIRED | sam::FLAG_MATE_UNMAPPED | sam::FLAG_FIRST_IN_PAIR;
            if r1.is_rev {
                flag |= sam::FLAG_REVERSE;
            }
            let line = sam::format_record(
                qname, flag, &r1.rname, r1.pos1,
                compute_mapq(r1.sort_score, 0),
                &r1.cigar, seq1_str, qual1_str,
                r1.score, 0, r1.nm, "",
            );
            let unmapped = sam::format_unmapped_with_flag(qname, seq2_str, qual2_str, sam::FLAG_PAIRED | sam::FLAG_UNMAPPED | sam::FLAG_SECOND_IN_PAIR);
            (vec![line], vec![unmapped])
        }
        (None, Some(r2)) => {
            let unmapped = sam::format_unmapped_with_flag(qname, seq1_str, qual1_str, sam::FLAG_PAIRED | sam::FLAG_UNMAPPED | sam::FLAG_FIRST_IN_PAIR);
            let mut flag = sam::FLAG_PAIRED | sam::FLAG_MATE_UNMAPPED | sam::FLAG_SECOND_IN_PAIR;
            if r2.is_rev {
                flag |= sam::FLAG_REVERSE;
            }
            let line = sam::format_record(
                qname, flag, &r2.rname, r2.pos1,
                compute_mapq(r2.sort_score, 0),
                &r2.cigar, seq2_str, qual2_str,
                r2.score, 0, r2.nm, "",
            );
            (vec![unmapped], vec![line])
        }
        (None, None) => {
            let unmapped1 = sam::format_unmapped_with_flag(qname, seq1_str, qual1_str, sam::FLAG_PAIRED | sam::FLAG_UNMAPPED | sam::FLAG_FIRST_IN_PAIR);
            let unmapped2 = sam::format_unmapped_with_flag(qname, seq2_str, qual2_str, sam::FLAG_PAIRED | sam::FLAG_UNMAPPED | sam::FLAG_SECOND_IN_PAIR);
            (vec![unmapped1], vec![unmapped2])
        }
    }
}
```

- [ ] **Step 3: Add helper function for unmapped with flag**

Add to `src/io/sam.rs`:

```rust
pub fn format_unmapped_with_flag(qname: &str, seq: &str, qual: &str, flag: u16) -> String {
    format!("{}\t{}\t*\t0\t0\t*\t*\t0\t0\t{}\t{}", qname, flag, seq, qual)
}
```

- [ ] **Step 4: Verify compilation**

Run: `cargo check`
Expected: No errors

- [ ] **Step 5: Commit**

```bash
git add src/align/pipeline.rs src/io/sam.rs
git commit -m "feat(align): add align_read_pair for paired-end alignment"
```

---

### Task 11: Add align_paired_fastq entry point

**Files:**
- Modify: `src/align/pipeline.rs`

- [ ] **Step 1: Implement align_paired_fastq**

Add after imports:

```rust
pub fn align_paired_fastq(
    fm: Arc<FMIndex>,
    fastq1_path: &str,
    fastq2_path: Option<&str>,
    out_path: Option<&str>,
    align_opt: AlignOpt,
    pairing_opt: PairingOpt,
) -> Result<()> {
    let mut out_box: Box<dyn Write> = if let Some(p) = out_path {
        Box::new(std::io::BufWriter::new(std::fs::File::create(p)?))
    } else {
        Box::new(std::io::BufWriter::new(std::io::stdout()))
    };
    
    let contig_info: Vec<(&str, u32)> = fm.contigs.iter().map(|c| (c.name.as_str(), c.len)).collect();
    sam::write_header(&mut out_box, &contig_info)?;
    
    let sw_params = SwParams {
        match_score: align_opt.match_score,
        mismatch_penalty: align_opt.mismatch_penalty,
        gap_open: align_opt.gap_open,
        gap_extend: align_opt.gap_extend,
        band_width: align_opt.band_width,
    };
    
    let insert_stats = Mutex::new(InsertSizeStats::new(pairing_opt.max_insert as i32));
    
    let pool = if align_opt.threads > 1 {
        Some(rayon::ThreadPoolBuilder::new()
            .num_threads(align_opt.threads)
            .build()
            .map_err(|e| anyhow::anyhow!("failed to build thread pool: {}", e))?)
    } else {
        None
    };
    
    if let Some(r2_path) = fastq2_path {
        let fq1 = std::fs::File::open(fastq1_path)?;
        let fq2 = std::fs::File::open(r2_path)?;
        let mut reader = PairedFastqReader::new_separate(
            std::io::BufReader::new(fq1),
            std::io::BufReader::new(fq2),
        );
        process_paired_batch(&mut reader, &fm, &mut out_box, sw_params, &align_opt, &pairing_opt, &insert_stats, &pool)?;
    } else {
        let fq = std::fs::File::open(fastq1_path)?;
        let mut reader = PairedFastqReader::new_interleaved(std::io::BufReader::new(fq));
        process_paired_batch(&mut reader, &fm, &mut out_box, sw_params, &align_opt, &pairing_opt, &insert_stats, &pool)?;
    }
    
    Ok(())
}

fn process_paired_batch<R1: BufRead, R2: BufRead>(
    reader: &mut PairedFastqReader<R1, R2>,
    fm: &Arc<FMIndex>,
    out: &mut Box<dyn Write>,
    sw_params: SwParams,
    align_opt: &AlignOpt,
    pairing_opt: &PairingOpt,
    insert_stats: &Mutex<InsertSizeStats>,
    pool: &Option<rayon::ThreadPool>,
) -> Result<()> {
    let batch_size = 1000;
    
    loop {
        let mut batch: Vec<ReadPair> = Vec::with_capacity(batch_size);
        for _ in 0..batch_size {
            match reader.next_pair()? {
                Some(pair) => batch.push(pair),
                None => break,
            }
        }
        
        if batch.is_empty() {
            break;
        }
        
        if let Some(pool) = pool {
            let fm_ref = Arc::clone(fm);
            let results: Vec<(Vec<String>, Vec<String>)> = pool.install(|| {
                batch
                    .iter()
                    .map(|pair| {
                        let mut stats = insert_stats.lock().unwrap().clone();
                        let result = align_read_pair(&fm_ref, pair, sw_params, align_opt, pairing_opt, &mut stats);
                        {
                            let mut global_stats = insert_stats.lock().unwrap();
                            global_stats.sample_count = stats.sample_count;
                            global_stats.median = stats.median;
                            global_stats.mad = stats.mad;
                            global_stats.max_insert = stats.max_insert;
                        }
                        result
                    })
                    .collect()
            });
            
            for (lines1, lines2) in results {
                for line in lines1 {
                    writeln!(out, "{}", line)?;
                }
                for line in lines2 {
                    writeln!(out, "{}", line)?;
                }
            }
        } else {
            let mut stats = insert_stats.lock().unwrap().clone();
            for pair in &batch {
                let (lines1, lines2) = align_read_pair(fm, pair, sw_params, align_opt, pairing_opt, &mut stats);
                for line in lines1 {
                    writeln!(out, "{}", line)?;
                }
                for line in lines2 {
                    writeln!(out, "{}", line)?;
                }
            }
            {
                let mut global_stats = insert_stats.lock().unwrap();
                global_stats.sample_count = stats.sample_count;
                global_stats.median = stats.median;
                global_stats.mad = stats.mad;
                global_stats.max_insert = stats.max_insert;
            }
        }
    }
    
    Ok(())
}
```

- [ ] **Step 2: Add BufRead import**

Add to imports:

```rust
use std::io::{BufRead, Write};
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check`
Expected: No errors

- [ ] **Step 4: Commit**

```bash
git add src/align/pipeline.rs
git commit -m "feat(align): add align_paired_fastq entry point"
```

---

## Phase 4: CLI Extension

### Task 12: Extend mem command for paired-end

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Add paired-end parameters to Mem command**

Find the `Mem` variant and modify:

```rust
    /// BWA-MEM style alignment: build index from FASTA and align FASTQ in one step
    Mem {
        /// Reference FASTA file
        reference: String,
        /// Reads FASTQ file (R1 for paired-end)
        reads: String,
        /// R2 FASTQ file for paired-end alignment (optional)
        reads2: Option<String>,
        /// Output SAM path (stdout if omitted)
        #[arg(short, long)]
        out: Option<String>,
        /// Treat input as interleaved FASTQ (paired-end)
        #[arg(short = 'p', long = "interleaved")]
        interleaved: bool,
        /// Disable mate rescue for unmapped mates
        #[arg(long = "no-mate-rescue")]
        no_mate_rescue: bool,
        /// Maximum insert size for paired-end alignment
        #[arg(long = "max-insert", default_value_t = 500)]
        max_insert: usize,
```

Continue with existing match/mismatch/gap parameters unchanged...

- [ ] **Step 2: Add validation for mutually exclusive options**

In the Mem command handling section, add validation:

```rust
        Commands::Mem {
            reference,
            reads,
            reads2,
            out,
            interleaved,
            no_mate_rescue,
            max_insert,
            // ... other params
        } => {
            if reads2.is_some() && interleaved {
                anyhow::bail!("--interleaved and reads2 are mutually exclusive");
            }
            
            let align_opt = AlignOpt {
                match_score,
                // ... populate all fields
            };
            
            let pairing_opt = PairingOpt {
                max_insert,
                min_insert: 0,
                mate_rescue: !no_mate_rescue,
                pen_unpaired: 17,
            };
            
            if reads2.is_some() || interleaved {
                handle_paired_mem(reference, reads, reads2, interleaved, out, align_opt, pairing_opt)?;
            } else {
                handle_single_mem(reference, reads, out, align_opt)?;
            }
        }
```

- [ ] **Step 3: Add helper functions**

Add after main function:

```rust
fn handle_single_mem(
    reference: String,
    reads: String,
    out: Option<String>,
    opt: AlignOpt,
) -> Result<()> {
    let index_path = format!("{}.fm", reference);
    align::align_fastq_with_opt(&index_path, &reads, out.as_deref(), opt)
}

fn handle_paired_mem(
    reference: String,
    reads: String,
    reads2: Option<String>,
    interleaved: bool,
    out: Option<String>,
    align_opt: AlignOpt,
    pairing_opt: PairingOpt,
) -> Result<()> {
    use std::sync::Arc;
    
    let index_path = format!("{}.fm", reference);
    
    if !std::path::Path::new(&index_path).exists() {
        index::build_fm_index(&reference, &reference)?;
    }
    
    let fm = Arc::new(index::fm::FMIndex::load_from_file(&index_path)?);
    
    let r2_path = if interleaved { None } else { reads2.as_deref() };
    align::align_paired_fastq(fm, &reads, r2_path, out.as_deref(), align_opt, pairing_opt)
}
```

- [ ] **Step 4: Add PairingOpt import**

Add to imports:

```rust
use bwa_rust::align::PairingOpt;
```

- [ ] **Step 5: Verify compilation**

Run: `cargo check`
Expected: No errors

- [ ] **Step 6: Commit**

```bash
git add src/main.rs
git commit -m "feat(cli): extend mem command for paired-end alignment"
```

---

## Phase 5: Integration Tests

### Task 13: Add paired-end integration tests

**Files:**
- Modify: `tests/integration.rs`

- [ ] **Step 1: Add test helper for paired FASTQ**

Add at top of file:

```rust
use std::io::Write;

fn create_paired_fastq_files() -> (tempfile::NamedTempFile, tempfile::NamedTempFile) {
    let mut r1 = tempfile::NamedTempFile::new().unwrap();
    let mut r2 = tempfile::NamedTempFile::new().unwrap();
    
    writeln!(r1, "@read1/1\nACGTACGT\n+\nIIIIIIII").unwrap();
    writeln!(r1, "@read2/1\nGGGGCCCC\n+\nIIIIIIII").unwrap();
    
    writeln!(r2, "@read1/2\nTTTTGGGG\n+\nIIIIIIII").unwrap();
    writeln!(r2, "@read2/2\nAAAACCCC\n+\nIIIIIIII").unwrap();
    
    (r1, r2)
}

fn create_interleaved_fastq() -> tempfile::NamedTempFile {
    let mut fq = tempfile::NamedTempFile::new().unwrap();
    
    writeln!(fq, "@read1/1\nACGTACGT\n+\nIIIIIIII").unwrap();
    writeln!(fq, "@read1/2\nTTTTGGGG\n+\nIIIIIIII").unwrap();
    writeln!(fq, "@read2/1\nGGGGCCCC\n+\nIIIIIIII").unwrap();
    writeln!(fq, "@read2/2\nAAAACCCC\n+\nIIIIIIII").unwrap();
    
    fq
}
```

- [ ] **Step 2: Add e2e_paired_fastq_separate test**

```rust
#[test]
fn e2e_paired_fastq_separate_files() {
    let ref_file = create_test_fasta();
    let (r1, r2) = create_paired_fastq_files();
    let output = tempfile::NamedTempFile::new().unwrap();
    
    let status = Command::new(env!("CARGO_BIN_EXE_bwa-rust"))
        .args(["mem", ref_file.path().to_str().unwrap(), r1.path().to_str().unwrap(), r2.path().to_str().unwrap()])
        .arg("-o").arg(output.path())
        .status()
        .expect("Failed to run bwa-rust");
    
    assert!(status.success());
    
    let content = std::fs::read_to_string(output.path()).unwrap();
    assert!(content.contains("@HD"));
    assert!(content.contains("@SQ"));
    assert!(content.contains("read1"));
    assert!(content.contains("read2"));
}
```

- [ ] **Step 3: Add e2e_paired_interleaved test**

```rust
#[test]
fn e2e_paired_fastq_interleaved() {
    let ref_file = create_test_fasta();
    let fq = create_interleaved_fastq();
    let output = tempfile::NamedTempFile::new().unwrap();
    
    let status = Command::new(env!("CARGO_BIN_EXE_bwa-rust"))
        .args(["mem", ref_file.path().to_str().unwrap(), fq.path().to_str().unwrap()])
        .arg("-p")
        .arg("-o").arg(output.path())
        .status()
        .expect("Failed to run bwa-rust");
    
    assert!(status.success());
    
    let content = std::fs::read_to_string(output.path()).unwrap();
    assert!(content.contains("read1"));
    assert!(content.contains("read2"));
}
```

- [ ] **Step 4: Add proper pair flags test**

```rust
#[test]
fn e2e_paired_proper_pair_flags() {
    let ref_file = create_test_fasta();
    let (r1, r2) = create_paired_fastq_files();
    let output = tempfile::NamedTempFile::new().unwrap();
    
    let status = Command::new(env!("CARGO_BIN_EXE_bwa-rust"))
        .args(["mem", ref_file.path().to_str().unwrap(), r1.path().to_str().unwrap(), r2.path().to_str().unwrap()])
        .arg("-o").arg(output.path())
        .status()
        .expect("Failed to run bwa-rust");
    
    assert!(status.success());
    
    let content = std::fs::read_to_string(output.path()).unwrap();
    for line in content.lines().skip_while(|l| l.starts_with('@')) {
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() > 1 {
            let flag: u16 = fields[1].parse().unwrap();
            assert!(flag & 0x1 != 0, "FLAG should have paired bit set");
        }
    }
}
```

- [ ] **Step 5: Add single-end unchanged regression test**

```rust
#[test]
fn e2e_single_end_unchanged() {
    let ref_file = create_test_fasta();
    let fq = create_test_fastq();
    let output = tempfile::NamedTempFile::new().unwrap();
    
    let status = Command::new(env!("CARGO_BIN_EXE_bwa-rust"))
        .args(["mem", ref_file.path().to_str().unwrap(), fq.path().to_str().unwrap()])
        .arg("-o").arg(output.path())
        .status()
        .expect("Failed to run bwa-rust");
    
    assert!(status.success());
    
    let content = std::fs::read_to_string(output.path()).unwrap();
    for line in content.lines().skip_while(|l| l.starts_with('@')) {
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() > 1 {
            let flag: u16 = fields[1].parse().unwrap();
            assert!(flag & 0x1 == 0, "Single-end should not have paired bit set");
        }
    }
}
```

- [ ] **Step 6: Run tests**

Run: `cargo test e2e_paired --test integration -- --nocapture`
Expected: All tests pass

- [ ] **Step 7: Commit**

```bash
git add tests/integration.rs
git commit -m "test: add paired-end integration tests"
```

---

### Task 14: Update CLI spec

**Files:**
- Modify: `openspec/specs/cli/spec.md`

- [ ] **Step 1: Add paired-end mem command documentation**

Add after existing mem command section:

```markdown
### Requirement: Paired-End Alignment via mem Command

The mem command SHALL support paired-end alignment when provided with R1/R2 file pairs or interleaved FASTQ.

#### Scenario: Align paired-end reads with separate files

- **WHEN** `bwa-rust mem ref.fa reads_1.fq reads_2.fq` is executed
- **THEN** both R1 and R2 reads SHALL be aligned as pairs
- **AND** SAM output SHALL include paired flags (0x1, 0x40, 0x80)
- **AND** mate information SHALL be populated (RNEXT, PNEXT, TLEN)

#### Scenario: Align interleaved FASTQ

- **WHEN** `bwa-rust mem ref.fa reads.fq -p` is executed
- **THEN** adjacent read pairs SHALL be treated as R1/R2
- **AND** output SHALL be identical to separate file mode

#### Scenario: Reject conflicting paired options

- **WHEN** both R2 file and `-p` are provided
- **THEN** the command SHALL fail with error message
- **AND** no alignment SHALL be performed

### Requirement: Mate Rescue

The mem command SHALL attempt to rescue unmapped mates when one mate is mapped.

#### Scenario: Rescue unmapped R2

- **WHEN** R1 is mapped but R2 has no alignment
- **THEN** the aligner SHALL search for R2 within rescue window around R1
- **AND** rescued alignments SHALL be marked with appropriate flags
- **AND** rescue SHALL be skipable via `--no-mate-rescue` flag

### Requirement: Insert Size Constraints

The mem command SHALL apply insert size constraints for proper pair detection.

#### Scenario: Configure maximum insert size

- **WHEN** `--max-insert 1000` is specified
- **THEN** pairs with insert size > 1000 SHALL not be marked as proper pairs
- **AND** default maximum SHALL be 500 bp
```

- [ ] **Step 2: Commit**

```bash
git add openspec/specs/cli/spec.md
git commit -m "docs(specs): add paired-end CLI requirements"
```

---

### Task 15: Run full verification suite

**Files:**
- None

- [ ] **Step 1: Run format check**

Run: `cargo fmt --all -- --check`
Expected: No output (passes)

- [ ] **Step 2: Run clippy**

Run: `cargo clippy --all-targets --all-features -- -D warnings`
Expected: No errors

- [ ] **Step 3: Run all tests**

Run: `cargo test --all-targets --all-features`
Expected: All tests pass (204 + new paired tests)

- [ ] **Step 4: Verify test count**

Run: `cargo test -- --list | grep -c "test$"`
Expected: Count >= 216

---

### Task 16: Final commit and push

**Files:**
- None

- [ ] **Step 1: Push to master**

```bash
git push origin master
```

Expected: Push succeeds

---

## Success Criteria Checklist

- [ ] `pairing.rs` module with score_pair, is_proper_pair, rescue_mate
- [ ] SAM paired output with correct FLAG bits
- [ ] `align_paired_fastq` pipeline entry point
- [ ] `mem` command supports R1/R2 and interleaved
- [ ] Mate rescue functional
- [ ] Insert size estimation integrated
- [ ] ≥ 12 new tests passing
- [ ] Single-end behavior unchanged
- [ ] All verification passes
