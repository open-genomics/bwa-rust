# Paired-End Alignment Design

## Context

Paired-end (PE) alignment extends the single-end alignment pipeline to handle mate-pair relationships, insert size constraints, and mate rescue scenarios.

## Goals / Non-Goals

**Goals:**
- Support standard FR orientation paired-end libraries
- Estimate insert size distribution automatically
- Rescue unmapped mates when possible
- Output proper SAM flags for paired reads

**Non-Goals:**
- Support for long-insert mate-pair libraries (RF orientation)
- Optical/PCR duplicate detection
- Advanced insert size modeling (bimodal distributions)

## Decisions

### Insert Size Estimation Strategy

Use median and MAD (Median Absolute Deviation) instead of mean and standard deviation.

**Rationale:** Robust to outliers from chimeric reads and incorrectly mapped pairs. MAD-based thresholds are more reliable for genomic data.

```rust
// MAD calculation
fn mad(values: &[i32]) -> f32 {
    let median = median(values);
    let deviations: Vec<i32> = values.iter()
        .map(|&v| (v - median).abs())
        .collect();
    median(&deviations) as f32
}
```

### Pair Scoring Formula

```
pair_score = score_R1 + score_R2
           - orientation_penalty
           - insert_size_deviation_penalty

where:
  orientation_penalty = 0 if proper (FR), 5 otherwise
  insert_size_deviation_penalty = |insert_size - median| / (2 * MAD)
```

**Rationale:** Simple but effective scoring that balances alignment quality with pairing constraints.

### Mate Rescue Algorithm

1. Identify unmapped mate with mapped mate
2. Extract reference window around mapped mate position
3. Perform banded SW with relaxed parameters
4. Accept rescue if score exceeds threshold

**Rationale:** Localized search is faster than genome-wide alignment for unmapped mates.

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| Insert size estimation fails on chimera-rich data | Use iterative estimation with outlier removal |
| Mate rescue may produce false positives | Use higher score threshold for rescue |
| PE alignment slower than SE | Process pairs together, share index access |

## Implementation Notes

### Data Flow Extension

```
PE Input (R1/R2)
    │
    ├── Align R1 independently
    │
    ├── Align R2 independently
    │
    ├── Pair candidates
    │       └── Find best pair within constraints
    │
    ├── Mate rescue (if needed)
    │       └── Search near mapped mate for unmapped mate
    │
    └── Output paired SAM records
```

### FLAG Bits for PE

| Bit | Meaning | Set When |
|-----|---------|----------|
| 0x1 | Paired | Always for PE input |
| 0x2 | Properly paired | FR orientation, valid insert size |
| 0x4 | Unmapped | No alignment found |
| 0x8 | Mate unmapped | Mate has no alignment |
| 0x10 | Reverse | Alignment on reverse strand |
| 0x20 | Mate reverse | Mate on reverse strand |
| 0x40 | First in pair | R1 |
| 0x80 | Second in pair | R2 |
