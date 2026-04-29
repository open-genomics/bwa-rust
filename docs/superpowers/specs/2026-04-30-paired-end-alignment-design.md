# Paired-End Alignment Design

**Version:** v0.3.0  
**Date:** 2026-04-30  
**Status:** Draft

## Summary

Implement BWA-MEM compatible paired-end alignment by extending the existing single-end pipeline. This includes proper pair scoring, mate rescue, insert size estimation, and full SAM paired-end flag support.

## Goals

- BWA-MEM behavior-compatible paired-end alignment
- Support both separate R1/R2 files and interleaved FASTQ
- Mate rescue for unmapped mates
- Proper pair detection with insert size constraints
- Correct SAM paired flags (0x1, 0x2, 0x8, 0x20, 0x40, 0x80)

## Non-Goals

- Exact BWA-MEM output equivalence (edge cases may differ)
- BAM/CRAM output (v0.5.0)
- BWA index compatibility

## Architecture

### Module Structure

```
src/align/
├── pairing.rs (new)     - Pairing logic, mate rescue, proper pair detection
├── pipeline.rs (modify) - Paired-end pipeline entry points
├── insert_size.rs (existing) - Already implemented, will be integrated
├── mod.rs (modify)      - Export PairingOpt and pairing module

src/io/
├── sam.rs (modify)      - Paired SAM output formatting

src/main.rs (modify)     - Extend mem command for paired-end
```

### Data Structures

#### PairedAlignment

```rust
pub struct PairedAlignment {
    pub r1: Option<AlignCandidate>,
    pub r2: Option<AlignCandidate>,
    pub is_proper_pair: bool,
    pub insert_size: i32,
    pub pair_score: i32,
}
```

#### PairScore

```rust
pub struct PairScore {
    pub score: i32,
    pub is_proper: bool,
    pub insert_size: i32,
}
```

## Core Algorithms

### 1. Pair Scoring

**FR Orientation Check:**
- R1 forward, R2 reverse on same contig
- R1 position < R2 position
- Insert size = R2_end - R1_start

**Score Calculation:**
```
pair_score = r1.score + r2.score
           - insert_size_penalty (if insert_size > median + 3*MAD)
           - orientation_penalty (if not FR orientation)
           - pen_unpaired (from PairingOpt)
```

### 2. Proper Pair Detection

A pair is "proper" if:
1. Both mates mapped
2. FR orientation (forward-reverse)
3. Same contig
4. Insert size within `median ± 3*MAD` (after 100+ samples collected)

Before 100 samples: insert size must be ≤ `max_insert` (default 500bp)

### 3. Mate Rescue

When one mate is unmapped but the other is mapped:
1. Extract region around mapped mate: `[pos - max_insert, pos + max_insert]`
2. Perform local alignment with relaxed parameters:
   - Lower `score_threshold` (50% of normal)
   - Wider band width
3. If rescue succeeds, mark rescued alignment with FLAG bit 0x100 (secondary)

### 4. Insert Size Estimation

Already implemented in `InsertSizeStats`:
- Collect insert sizes from proper pairs
- Calculate median and MAD every 1000 samples
- Update `max_insert = median + 3*MAD`
- Cap at 100,000 samples

## CLI Design

### Command Extension

```bash
# Single-end (existing)
bwa-rust mem ref.fa reads.fq

# Paired-end separate files (new)
bwa-rust mem ref.fa reads_1.fq reads_2.fq

# Paired-end interleaved (new)
bwa-rust mem ref.fa reads.fq -p
```

### New Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `reads2` | positional | - | R2 FASTQ file (optional, enables paired-end) |
| `-p, --interleaved` | flag | false | Treat input as interleaved FASTQ |
| `--no-mate-rescue` | flag | false | Disable mate rescue |
| `--max-insert` | int | 500 | Maximum insert size |

### Parameter Priority

When both R2 file and `-p` are provided: **error** (mutually exclusive)

## SAM Output

### Flag Bits for Paired-End

| Bit | Hex | Description |
|-----|-----|-------------|
| 1 | 0x1 | Paired |
| 2 | 0x2 | Proper pair |
| 4 | 0x4 | Unmapped |
| 8 | 0x8 | Mate unmapped |
| 16 | 0x10 | Reverse |
| 32 | 0x20 | Mate reverse |
| 64 | 0x40 | First in pair (R1) |
| 128 | 0x80 | Second in pair (R2) |
| 256 | 0x100 | Secondary |
| 2048 | 0x800 | Supplementary |

### Flag Examples

- R1 mapped (forward), R2 mapped (reverse), proper pair: `67` (0x1|0x2|0x40)
- R1 mapped (reverse), R2 mapped (forward), proper pair: `131` (0x1|0x2|0x10|0x80)
- R1 mapped, R2 unmapped: `73` (0x1|0x8|0x40)
- R1 unmapped, R2 mapped: `141` (0x1|0x4|0x80)

### Additional Fields

- `RNEXT`: Mate reference name (or "=" if same contig)
- `PNEXT`: Mate position (1-based)
- `TLEN`: Insert size (signed: positive if R1 < R2, negative otherwise)

## Pipeline Flow

```
1. Parse CLI args → determine paired-end mode (separate/interleaved)
2. Load FM-index
3. Initialize InsertSizeStats with default max_insert=500
4. Create PairedFastqReader (separate or interleaved mode)
5. Batch read ReadPairs (batch_size=1000)
6. Parallel process each ReadPair:
   a. Align R1 → collect candidates
   b. Align R2 → collect candidates
   c. If both have candidates:
      - Find best pairing (score all combinations)
      - Determine if proper pair
   d. If only one has candidates:
      - Attempt mate rescue
   e. If rescue succeeds:
      - Update alignment pair
   f. Update InsertSizeStats (only proper pairs)
   g. Generate SAM lines with paired flags
7. Output SAM to stdout or file
```

## Testing Strategy

### Unit Tests (src/align/pairing.rs)

- `score_pair_fr_orientation` - FR orientation correct scoring
- `score_pair_rf_orientation` - RF orientation with penalty
- `score_pair_same_contig` - Same contig pairs
- `score_pair_different_contig` - Different contig pairs
- `is_proper_pair_valid_insert` - Insert size within range
- `is_proper_pair_invalid_insert` - Insert size out of range
- `is_proper_pair_wrong_orientation` - Wrong orientation not proper
- `rescue_mate_success` - Mate rescue succeeds
- `rescue_mate_failure` - Mate rescue fails
- `rescue_mate_no_region` - No valid rescue region

### Integration Tests (tests/integration.rs)

- `e2e_paired_fastq_separate_files` - R1/R2 separate file input
- `e2e_paired_fastq_interleaved` - Interleaved FASTQ input
- `e2e_paired_proper_pair_flags` - Proper pair FLAG bits correct
- `e2e_paired_improper_pair_flags` - Improper pair FLAG bits
- `e2e_paired_mate_rescue` - Mate rescue activates and succeeds
- `e2e_paired_insert_size_stats` - Insert size stats update correctly
- `e2e_paired_both_unmapped` - Both mates unmapped
- `e2e_paired_rna_flags` - RNEXT/PNEXT/TLEN correct
- `e2e_single_end_unchanged` - Single-end behavior unchanged

### Validation Gates

- New test count ≥ 12
- Total tests pass (204 + new)
- `cargo fmt --all -- --check` passes
- `cargo clippy --all-targets --all-features -- -D warnings` passes
- `cargo test --all-targets --all-features` passes

## File Changes

| File | Change | Lines (est.) |
|------|--------|--------------|
| `src/align/pairing.rs` | New | ~300 |
| `src/align/pipeline.rs` | Modify | +150 |
| `src/align/mod.rs` | Modify | +5 |
| `src/io/sam.rs` | Modify | +100 |
| `src/main.rs` | Modify | +80 |
| `tests/integration.rs` | Modify | +200 |
| `openspec/specs/cli/spec.md` | Modify | +30 |

**Total estimated:** ~865 new lines

## Implementation Order

1. **Phase 1: Core Pairing Logic**
   - Create `src/align/pairing.rs` with score_pair, is_proper_pair
   - Add unit tests for pairing functions

2. **Phase 2: SAM Paired Output**
   - Extend `src/io/sam.rs` with format_paired_records
   - Add FLAG calculation helpers

3. **Phase 3: Pipeline Integration**
   - Extend `src/align/pipeline.rs` with align_paired_fastq
   - Integrate InsertSizeStats
   - Add mate rescue logic

4. **Phase 4: CLI Extension**
   - Modify `src/main.rs` for paired-end args
   - Add validation for mutually exclusive options

5. **Phase 5: Integration Tests**
   - Add comprehensive e2e tests
   - Update openspec/specs/cli/spec.md

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Insert size estimation instability | Cap samples at 100k, use MAD for robustness |
| Mate rescue performance | Limit search region, use relaxed parameters |
| Paired flag complexity | Comprehensive test matrix for all combinations |
| Single-end regression | Add regression test ensuring single-end unchanged |

## Success Criteria

- [ ] Paired-end alignment produces correct SAM flags
- [ ] Proper pair detection matches BWA-MEM behavior (FR orientation + insert size)
- [ ] Mate rescue recovers unmapped mates within expected window
- [ ] Insert size statistics update from proper pairs
- [ ] Both separate and interleaved FASTQ supported
- [ ] Single-end alignment behavior unchanged
- [ ] All tests pass (existing + new ≥ 216)
- [ ] No clippy warnings
- [ ] Code formatted correctly

## References

- OpenSpec: `openspec/specs/paired-end-alignment/spec.md`
- BWA-MEM manual: http://bio-bwa.sourceforge.net/bwa.shtml
- SAM specification: https://samtools.github.io/hts-specs/SAMv1.pdf
