# Product Specification: Paired-End Alignment

> **Version**: v0.3.0
> **Status**: Draft
> **Last Updated**: 2026-04-22

## Overview

This document defines the product features and acceptance criteria for paired-end (PE) read alignment support in bwa-rust.

## Feature List

### F-011: Paired-End FASTQ Input

**Description**: Support paired-end FASTQ input files (R1/R2 pairs).

**Acceptance Criteria**:
- [ ] Parse paired FASTQ files (reads_1.fq, reads_2.fq)
- [ ] Validate read name pairing (same prefix before `/1` or `/2`)
- [ ] Handle interleaved FASTQ format
- [ ] Support gzip compressed FASTQ files
- [ ] Report error on read count mismatch between R1 and R2

**CLI Interface**:
```bash
# Separate files
bwa-rust mem ref.fa reads_1.fq reads_2.fq -o output.sam

# Interleaved format
bwa-rust mem ref.fa reads_interleaved.fq -p -o output.sam
```

### F-012: Insert Size Estimation

**Description**: Estimate insert size distribution from properly paired alignments.

**Acceptance Criteria**:
- [ ] Collect insert sizes from properly paired alignments
- [ ] Calculate median insert size
- [ ] Calculate insert size standard deviation (MAD-based)
- [ ] Update estimates iteratively during alignment
- [ ] Use median + 3*MAD as maximum insert size threshold
- [ ] Default maximum insert size: 500 bp (before estimation)

**Output Format**:
```
@PG	ID:bwa-rust	VN:0.3.0	CL:bwa-rust mem ...
@CO	Median insert size: 350
@CO	Insert size MAD: 50
```

### F-013: Pairing Logic

**Description**: Implement proper pairing rules for PE alignments.

**Acceptance Criteria**:
- [ ] Find best pair within insert size constraints
- [ ] Consider orientation: FR (forward-reverse, most common)
- [ ] Support RF and FF orientations (rare cases)
- [ ] Calculate pairing score combining both mates
- [ ] Mark improperly paired alignments as secondary

**Pairing Rules**:
| Orientation | Valid | Description |
|-------------|-------|-------------|
| FR | ✅ | Forward (R1) ← Reverse (R2), most common |
| RF | ✅ | Reverse (R1) → Forward (R2), Mate-pair libraries |
| FF | ⚠️ | Both forward, rare, mark as improper |
| RR | ⚠️ | Both reverse, rare, mark as improper |

### F-014: Mate Rescue

**Description**: Rescue unmapped mates by searching near the mapped mate's position.

**Acceptance Criteria**:
- [ ] Detect unmapped mate with mapped mate
- [ ] Search for unmapped mate within rescue window
- [ ] Use relaxed alignment parameters for rescue
- [ ] Mark rescued alignments with appropriate FLAG
- [ ] Set MAPQ to lower value for rescued mates

**Rescue Parameters**:
| Parameter | Default | Description |
|-----------|---------|-------------|
| `--rescue-window` | 1000 | Search window around mapped mate |
| `--rescue-min-score` | 15 | Minimum score for rescued alignment |

### F-015: PE SAM Output

**Description**: Format paired-end alignments in SAM format with proper flags.

**Acceptance Criteria**:
- [ ] Set proper FLAG bits for paired reads
- [ ] Include RNEXT, PNEXT, TLEN fields
- [ ] Handle proper/improper pairing flags
- [ ] Output both mapped and unmapped mates
- [ ] Support secondary and supplementary alignments

**FLAG Bits**:
| Bit | Description |
|-----|-------------|
| 0x1 | Paired |
| 0x2 | Properly paired |
| 0x4 | Unmapped |
| 0x8 | Mate unmapped |
| 0x10 | Reverse complemented |
| 0x20 | Mate reverse complemented |
| 0x40 | First in pair (R1) |
| 0x80 | Second in pair (R2) |
| 0x100 | Secondary alignment |
| 0x800 | Supplementary alignment |

**SAM Fields Example**:
```
read1	99	chr1	100	60	50M	=	200	150	ACGT...	IIII...	AS:i:50	NM:i:0
read1	147	chr1	200	60	50M	=	100	-150	ACGT...	IIII...	AS:i:48	NM:i:1
```

### F-016: Pair Scoring

**Description**: Score alignment pairs to select the best pair.

**Acceptance Criteria**:
- [ ] Combine alignment scores from both mates
- [ ] Apply penalty for improper orientation
- [ ] Apply penalty for insert size deviation from median
- [ ] Select pair with highest combined score
- [ ] Handle one mate unmapped scenarios

**Scoring Formula**:
```
pair_score = score_R1 + score_R2
           - orientation_penalty
           - insert_size_deviation_penalty

where:
  orientation_penalty = 0 if proper (FR), 5 otherwise
  insert_size_deviation_penalty = |insert_size - median| / (2 * MAD)
```

### F-017: PE CLI Options

**Description**: Add paired-end specific CLI options.

**Acceptance Criteria**:
- [ ] `-p, --interleaved`: Input is interleaved FASTQ
- [ ] `-I, --insert-size`: Expected insert size (default: auto)
- [ ] `--max-insert`: Maximum insert size (default: 500)
- [ ] `--rescue-window`: Mate rescue window size
- [ ] `--no-rescue`: Disable mate rescue

**CLI Examples**:
```bash
# Basic PE alignment
bwa-rust mem ref.fa reads_1.fq reads_2.fq -o output.sam

# With custom insert size
bwa-rust mem ref.fa reads_1.fq reads_2.fq -I 300 --max-insert 600

# Interleaved input
bwa-rust mem ref.fa reads_interleaved.fq -p -o output.sam

# Disable mate rescue
bwa-rust mem ref.fa reads_1.fq reads_2.fq --no-rescue
```

## Non-Functional Requirements

### NFR-PE-001: Performance

| Metric | Target |
|--------|--------|
| PE alignment overhead vs SE | < 20% |
| Mate rescue overhead | < 10% |
| Memory overhead | < 50 MB for insert size tracking |

### NFR-PE-002: Correctness

| Metric | Target |
|--------|--------|
| Properly paired rate | > 90% for good data |
| False pairing rate | < 1% |
| Insert size estimation error | < 5% |

## Testing Requirements

### Test Cases

1. **Basic Pairing**
   - Simple FR orientation pairs
   - Verify proper FLAG settings
   - Verify TLEN calculation

2. **Insert Size Estimation**
   - Known insert size dataset
   - Verify median within 5% of true value

3. **Mate Rescue**
   - One mate unmapped
   - Verify rescue finds correct position
   - Verify lower MAPQ for rescued mates

4. **Edge Cases**
   - Both mates unmapped
   - Both mates map to different chromosomes
   - Insert size exceeds maximum
   - Interleaved FASTQ parsing

5. **Stress Testing**
   - 1M PE reads
   - Multi-threaded correctness
   - Memory stability

## Migration from v0.2.0

**Breaking Changes**: None

**Compatibility**:
- Single-end alignment continues to work unchanged
- Existing CLI commands remain valid
- `.fm` index format unchanged (version 2)

**New Dependencies**:
- `flate2` for gzip support (optional)

## Timeline

| Phase | Duration | Tasks |
|-------|----------|-------|
| Phase 1 | 1 week | Spec finalization, RFC design |
| Phase 2 | 2 weeks | PE parsing, insert size estimation |
| Phase 3 | 1 week | Pairing logic, mate rescue |
| Phase 4 | 1 week | SAM output, testing |
| Phase 5 | 1 week | Documentation, release |

**Total**: 6 weeks
