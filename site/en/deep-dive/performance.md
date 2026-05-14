# Performance Analysis

## Overview

This page documents the complexity analysis, benchmark methodology, and performance characteristics of bwa-rust.

## Complexity Analysis

### Index Construction

| Operation | Time Complexity | Space Complexity |
|-----------|-----------------|------------------|
| Suffix Array (Doubling) | O(n log² n) | O(n) |
| BWT Construction | O(n) | O(n) |
| FM-index Serialization | O(n) | O(n) |

Where n = reference genome length.

### Alignment Operations

| Operation | Time Complexity | Notes |
|-----------|-----------------|-------|
| Backward search | O(m) | m = read length |
| SMEM seeding | O(m) | Linear in read length |
| Chain building | O(k²) | k = number of seeds |
| Smith-Waterman | O(w × l) | w = band width, l = alignment length |

## Benchmark Methodology

### Test Datasets

| Dataset | Size | Source |
|---------|------|--------|
| E. coli K-12 | 4.6 Mbp | RefSeq NC_000913.3 |
| S. cerevisiae | 12 Mbp | RefSeq R64 |
| Human chr22 | 51 Mbp | GRCh38 |

### Metrics

- **Throughput**: Reads aligned per second
- **Latency**: Time per read alignment
- **Memory**: Peak RSS during operation
- **Accuracy**: Alignment rate and correctness

### Running Benchmarks

```bash
# Build index
bwa-rust index reference.fasta -o reference.fm

# Align with timing
time bwa-rust align reference.fm reads.fastq -o output.sam

# Memory profiling (Linux)
/usr/bin/time -v bwa-rust align reference.fm reads.fastq -o output.sam
```

## Performance Characteristics

### Indexing Speed

Typical indexing times on a modern CPU (single-threaded):

| Reference | Size | Time | Peak Memory |
|-----------|------|------|-------------|
| E. coli | 4.6 Mbp | ~3s | ~50 MB |
| Yeast | 12 Mbp | ~8s | ~120 MB |
| Chr22 | 51 Mbp | ~30s | ~500 MB |

### Alignment Throughput

Single-end alignment (single-threaded):

| Reference | Reads | Reads/sec |
|-----------|-------|-----------|
| E. coli | 1M 100bp | ~50,000 |
| Yeast | 1M 100bp | ~45,000 |
| Chr22 | 1M 100bp | ~40,000 |

With Rayon parallelism (8 threads):

| Reference | Reads | Reads/sec |
|-----------|-------|-----------|
| E. coli | 1M 100bp | ~300,000 |
| Yeast | 1M 100bp | ~280,000 |
| Chr22 | 1M 100bp | ~250,000 |

## Trade-off Decisions

### SA Sampling Interval

Default interval: 4

| Interval | Memory | Query Time |
|----------|--------|------------|
| 1 | 100% | Fastest |
| 4 | 25% | +O(4) overhead |
| 8 | 12.5% | +O(8) overhead |

### Band Width

Default: 16

| Width | Sensitivity | Speed |
|-------|-------------|-------|
| 8 | Lower | Faster |
| 16 | Balanced | Balanced |
| 32 | Higher | Slower |

## Optimization Tips

1. **Use parallelism**: Set `--threads` to match CPU cores
2. **Adjust `max_occ`**: Lower values reduce seed explosion in repetitive regions
3. **Tune `min_seed_len`**: Longer seeds reduce false positives
4. **Consider reference size**: Very large references may need memory optimization

## Comparison with BWA

::: warning Not Bit-Level Compatible
bwa-rust is inspired by BWA-MEM but does not pursue bit-level output compatibility. Results are similar but not identical.
:::

| Aspect | bwa-rust | BWA-MEM |
|--------|----------|---------|
| Safety | Zero unsafe | Contains unsafe |
| Index format | Single file | Multi-file |
| Paired-end | Planned | Supported |
| BAM output | Planned | Supported |

## Profiling

For detailed performance analysis:

```bash
# Build with debug symbols
cargo build --release --features profiling

# Use perf (Linux)
perf record -g target/release/bwa-rust align ...
perf report
```

---

[← Memory Safety](/en/deep-dive/memory-safety) | [Specs →](/en/specs/)
