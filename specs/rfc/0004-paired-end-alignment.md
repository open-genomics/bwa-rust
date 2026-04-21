# RFC-0004: Paired-End Alignment Implementation

> **Status**: Draft  
> **Author**: LessUp  
> **Created**: 2026-04-22  
> **Target Version**: v0.3.0

## Summary

This RFC proposes the implementation of paired-end (PE) read alignment support for bwa-rust. The design extends the existing single-end alignment pipeline to handle paired FASTQ input, estimate insert size distributions, and properly pair alignments with correct SAM flags.

## Motivation

Paired-end sequencing is the dominant approach in modern DNA sequencing. Supporting PE alignment is essential for:

1. **Standard workflows**: Most NGS pipelines require PE support
2. **Better accuracy**: Pairing constraints improve mapping accuracy
3. **Structural variant detection**: Insert size deviations indicate SVs
4. **Mate rescue**: Recover alignments for reads where one mate fails to map

## Design Overview

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    PE Alignment Pipeline                      │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  FASTQ R1/R2 → Parse Pairs → Align Separately → Pair Scoring │
│       ↓                                        ↓              │
│  Insert Size                              Mate Rescue          │
│  Estimation                                    ↓              │
│       ↓                                    Pair Selection      │
│  Update Threshold                               ↓              │
│                                           SAM PE Output        │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

### Data Structures

#### 1. PairedRead

```rust
pub struct PairedRead {
    pub name: String,
    pub r1: Read,
    pub r2: Read,
}

pub struct Read {
    pub seq: Vec<u8>,
    pub qual: Vec<u8>,
    pub is_reverse: bool,
}
```

#### 2. PairedAlignment

```rust
pub struct PairedAlignment {
    pub r1: Option<Alignment>,
    pub r2: Option<Alignment>,
    pub insert_size: i32,
    pub is_proper_pair: bool,
    pub pair_score: i32,
}

pub struct Alignment {
    pub contig: usize,
    pub pos: u32,
    pub flag: u16,
    pub mapq: u8,
    pub cigar: Vec<CigarOp>,
    pub score: i32,
}
```

#### 3. InsertSizeStats

```rust
pub struct InsertSizeStats {
    pub median: f64,
    pub mad: f64,  // Median absolute deviation
    pub max_insert: i32,
    pub sample_count: usize,
}

impl InsertSizeStats {
    pub fn update(&mut self, insert_size: i32) { ... }
    pub fn is_valid_insert(&self, insert_size: i32) -> bool { ... }
}
```

### Pipeline Components

#### 1. PE FASTQ Parser (io/fastq.rs)

```rust
pub fn parse_paired_fastq(
    r1_path: &Path,
    r2_path: &Path,
) -> impl Iterator<Item = Result<PairedRead>> {
    // Parse both files in parallel
    // Validate read name pairing
    // Yield paired reads
}

pub fn parse_interleaved_fastq(
    path: &Path,
) -> impl Iterator<Item = Result<PairedRead>> {
    // Parse interleaved format
    // Validate /1 and /2 suffixes
}
```

#### 2. Pair Alignment Engine (align/pair.rs)

```rust
pub fn align_paired(
    r1: &Read,
    r2: &Read,
    fm: &FMIndex,
    opt: &AlignOpt,
    insert_stats: &mut InsertSizeStats,
) -> PairedAlignment {
    // 1. Align both mates independently
    let alignments_r1 = align_single_read(r1, fm, opt);
    let alignments_r2 = align_single_read(r2, fm, opt);
    
    // 2. Score all possible pairs
    let pairs = score_pairs(alignments_r1, alignments_r2, insert_stats);
    
    // 3. Select best pair
    let best_pair = select_best_pair(pairs);
    
    // 4. Attempt mate rescue if needed
    let rescued_pair = rescue_unmapped_mate(best_pair, fm, opt);
    
    rescued_pair
}
```

#### 3. Pair Scoring (align/pair.rs)

```rust
fn score_pairs(
    alignments_r1: Vec<Alignment>,
    alignments_r2: Vec<Alignment>,
    insert_stats: &InsertSizeStats,
) -> Vec<PairedAlignment> {
    // Cartesian product of R1 and R2 alignments
    // Calculate pair score for each combination
    // Filter pairs outside insert size constraints
    // Sort by pair score
}

fn calculate_pair_score(
    a1: &Alignment,
    a2: &Alignment,
    insert_stats: &InsertSizeStats,
) -> i32 {
    let base_score = a1.score + a2.score;
    
    // Orientation penalty
    let orientation_penalty = match get_orientation(a1, a2) {
        Orientation::FR => 0,
        Orientation::RF => 5,
        _ => 10,
    };
    
    // Insert size deviation penalty
    let insert_size = calculate_insert_size(a1, a2);
    let deviation_penalty = if insert_stats.sample_count > 100 {
        let expected = insert_stats.median as i32;
        let tolerance = 3.0 * insert_stats.mad;
        let diff = (insert_size - expected).abs() as f64;
        if diff > tolerance {
            (diff / (2.0 * insert_stats.mad)) as i32
        } else {
            0
        }
    } else {
        0
    };
    
    base_score - orientation_penalty - deviation_penalty
}
```

#### 4. Mate Rescue (align/rescue.rs)

```rust
pub fn rescue_unmapped_mate(
    pair: PairedAlignment,
    fm: &FMIndex,
    opt: &AlignOpt,
) -> PairedAlignment {
    match (&pair.r1, &pair.r2) {
        (Some(mapped), None) => {
            // R1 mapped, rescue R2
            let rescued = rescue_near_position(
                &pair.r2_original_read,
                mapped.contig,
                mapped.pos,
                opt.rescue_window,
                fm,
                opt,
            );
            PairedAlignment { r2: rescued, ..pair }
        }
        (None, Some(mapped)) => {
            // R2 mapped, rescue R1
            let rescued = rescue_near_position(
                &pair.r1_original_read,
                mapped.contig,
                mapped.pos,
                opt.rescue_window,
                fm,
                opt,
            );
            PairedAlignment { r1: rescued, ..pair }
        }
        _ => pair,
    }
}

fn rescue_near_position(
    read: &Read,
    contig: usize,
    pos: u32,
    window: u32,
    fm: &FMIndex,
    opt: &AlignOpt,
) -> Option<Alignment> {
    // Extract reference region around position
    // Perform local alignment with relaxed parameters
    // Return alignment if score exceeds rescue threshold
}
```

#### 5. Insert Size Estimation (align/insert_size.rs)

```rust
pub struct InsertSizeEstimator {
    samples: Vec<i32>,
    median: f64,
    mad: f64,
    max_insert: i32,
}

impl InsertSizeEstimator {
    pub fn new(initial_max: i32) -> Self { ... }
    
    pub fn add_sample(&mut self, insert_size: i32) {
        if self.samples.len() < MAX_SAMPLES {
            self.samples.push(insert_size);
        }
        
        // Periodically update statistics
        if self.samples.len() % UPDATE_INTERVAL == 0 {
            self.update_stats();
        }
    }
    
    fn update_stats(&mut self) {
        // Calculate median
        self.samples.sort();
        let mid = self.samples.len() / 2;
        self.median = self.samples[mid] as f64;
        
        // Calculate MAD (median absolute deviation)
        let deviations: Vec<f64> = self.samples.iter()
            .map(|&x| (x as f64 - self.median).abs())
            .collect();
        let mut sorted_devs = deviations;
        sorted_devs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        self.mad = sorted_devs[sorted_devs.len() / 2];
        
        // Update max insert size
        self.max_insert = (self.median + 3.0 * self.mad) as i32;
    }
}
```

### SAM Output Changes (io/sam.rs)

```rust
pub fn format_pe_record(
    name: &str,
    alignment: &Alignment,
    mate: &MateInfo,
    is_first: bool,
) -> String {
    let flag = calculate_pe_flag(alignment, mate, is_first);
    let rnext = if mate.contig == alignment.contig { "=" } else { &mate.contig_name };
    let pnext = mate.pos;
    let tlen = calculate_tlen(alignment, mate);
    
    format!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        name, flag, alignment.contig_name, alignment.pos,
        alignment.mapq, format_cigar(&alignment.cigar),
        rnext, pnext, tlen,
        alignment.seq, alignment.qual,
        format_tags(&alignment.tags)
    )
}

pub struct MateInfo {
    pub contig: usize,
    pub contig_name: String,
    pub pos: u32,
    pub is_reverse: bool,
    pub is_unmapped: bool,
}

fn calculate_pe_flag(
    alignment: &Alignment,
    mate: &MateInfo,
    is_first: bool,
) -> u16 {
    let mut flag = 0u16;
    
    flag |= 0x1;  // Paired
    
    if !alignment.is_unmapped && !mate.is_unmapped {
        flag |= 0x2;  // Properly paired
    }
    
    if alignment.is_unmapped {
        flag |= 0x4;
    }
    
    if mate.is_unmapped {
        flag |= 0x8;
    }
    
    if alignment.is_reverse {
        flag |= 0x10;
    }
    
    if mate.is_reverse {
        flag |= 0x20;
    }
    
    if is_first {
        flag |= 0x40;  // First in pair
    } else {
        flag |= 0x80;  // Second in pair
    }
    
    flag
}

fn calculate_tlen(alignment: &Alignment, mate: &MateInfo) -> i32 {
    if alignment.is_unmapped || mate.is_unmapped {
        return 0;
    }
    
    if alignment.contig != mate.contig {
        return 0;
    }
    
    // TLEN = distance from leftmost to rightmost base
    let left_pos = alignment.pos.min(mate.pos) as i32;
    let right_pos = (alignment.pos.max(mate.pos) + alignment.len) as i32;
    
    if alignment.pos <= mate.pos {
        right_pos - left_pos
    } else {
        -(right_pos - left_pos)
    }
}
```

### CLI Changes (main.rs)

```rust
#[derive(Parser)]
#[command(name = "bwa-rust")]
enum Command {
    #[command(about = "Build FM index")]
    Index {
        // ... existing ...
    },
    
    #[command(about = "Align single-end reads")]
    Align {
        // ... existing ...
    },
    
    #[command(about = "BWA-MEM style alignment (supports PE)")]
    Mem {
        #[arg(help = "Reference FASTA")]
        reference: String,
        
        #[arg(help = "Reads file (FASTQ)")]
        reads: String,
        
        #[arg(help = "Second reads file (for PE)")]
        reads2: Option<String>,
        
        #[arg(short = 'p', long, help = "Input is interleaved FASTQ")]
        interleaved: bool,
        
        #[arg(short = 'I', long, help = "Expected insert size")]
        insert_size: Option<i32>,
        
        #[arg(long, default_value = "500", help = "Maximum insert size")]
        max_insert: i32,
        
        #[arg(long, default_value = "1000", help = "Mate rescue window")]
        rescue_window: u32,
        
        #[arg(long, help = "Disable mate rescue")]
        no_rescue: bool,
        
        // ... existing options ...
    },
}
```

## Implementation Plan

### Phase 1: Parsing (Week 1)

- [ ] Add `PairedRead` struct to `io/fastq.rs`
- [ ] Implement `parse_paired_fastq()` 
- [ ] Implement `parse_interleaved_fastq()`
- [ ] Add gzip support via `flate2` (optional feature)
- [ ] Add unit tests for PE parsing

### Phase 2: Insert Size (Week 2)

- [ ] Add `InsertSizeEstimator` to `align/insert_size.rs`
- [ ] Implement median and MAD calculation
- [ ] Integrate with alignment pipeline
- [ ] Add unit tests for insert size estimation

### Phase 3: Pairing Logic (Week 3)

- [ ] Add `PairedAlignment` struct
- [ ] Implement `align_paired()` in `align/pair.rs`
- [ ] Implement pair scoring algorithm
- [ ] Implement mate rescue logic
- [ ] Add unit tests for pairing

### Phase 4: SAM Output (Week 4)

- [ ] Update `io/sam.rs` for PE format
- [ ] Implement PE flag calculation
- [ ] Implement TLEN calculation
- [ ] Add integration tests with PE test data

### Phase 5: Documentation & Release (Week 5)

- [ ] Update README with PE examples
- [ ] Add PE tutorial documentation
- [ ] Update CHANGELOG.md
- [ ] Create v0.3.0 release

## Performance Considerations

### Memory

- Insert size estimator: ~1MB for 100K samples
- No significant memory overhead for PE alignment

### Speed

- PE alignment ~1.5x slower than SE (expected)
- Mate rescue only for unmapped mates (~5-10% of reads)
- Insert size estimation is incremental, negligible overhead

### Parallelism

- PE alignment is naturally parallelizable at read pair level
- Insert size stats updated atomically (use `AtomicU64` for counters)
- No lock contention expected

## Testing Strategy

### Unit Tests

- PE FASTQ parsing correctness
- Insert size estimation accuracy
- Pair scoring algorithm
- Mate rescue logic
- SAM flag calculation

### Integration Tests

- Full PE alignment pipeline
- Properly paired rate validation
- Insert size estimation validation
- Mate rescue effectiveness

### Test Data

- Synthetic PE reads with known insert sizes
- Real PE data from public datasets
- Edge cases: unmapped mates, chimeric pairs

## Backward Compatibility

- Single-end alignment unchanged
- Existing CLI commands continue to work
- No breaking changes to API
- `.fm` index format unchanged

## Future Enhancements

- Support for RF/FF orientation (Mate-pair libraries)
- Advanced insert size models (Gaussian fitting)
- Multi-threaded mate rescue
- BAM output with PE sorting

## References

- SAM Format Specification: https://samtools.github.io/hts-specs/SAMv1.pdf
- BWA-MEM Paper: Li H. (2013) Aligning sequence reads with BWA-MEM
- Insert Size Estimation: Statistical methods for NGS data
