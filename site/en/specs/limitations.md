# Limitations

## Overview

This page explicitly states what bwa-rust does **not** support. Honest scope declaration is a core project value.

## Current Limitations

### Single-End Only

**Status**: Planned for v0.3.0

bwa-rust currently only supports single-end reads:

- ✅ FASTQ single-end input
- ❌ FASTQ paired-end input
- ❌ Proper pair inference
- ❌ Insert size estimation

**Workaround**: For paired-end data, align each file separately and merge manually.

### SAM Output Only

**Status**: Planned for v0.5.0

Current output format:

- ✅ SAM text output
- ❌ BAM binary output
- ❌ CRAM compressed output
- ❌ Coordinate sorting

**Workaround**: Pipe SAM through `samtools view -bS`:

```bash
bwa-rust align index.fm reads.fq | samtools view -bS -o output.bam -
```

### No BWA Compatibility

**Status**: Non-goal

bwa-rust is **not** designed for BWA output compatibility:

- Different index format (single-file vs multi-file)
- Different algorithmic decisions
- Different tie-breaking rules
- Different MAPQ calculation

**Implication**: Do not use for BWA compatibility testing.

### No Large-Reference Optimization

**Status**: Planned for v0.4.0

Current limitations for large references:

- Full in-memory index
- No streaming reference
- No chromosome-level chunking

**Practical limit**: Works well up to ~1 Gbp. Human genome (3 Gbp) may require substantial RAM.

### No GPU Acceleration

**Status**: Non-goal

GPU acceleration is not planned:

- Focus on portable CPU implementation
- Use Rayon for multi-threading
- Consider external GPU wrappers if needed

## Non-Goals

These are explicitly **not** planned:

| Feature | Reason |
|---------|--------|
| BWA index compatibility | Different design philosophy |
| Bit-level output match | Not pursuing compatibility |
| Real-time streaming | Batch processing model |
| GUI interface | CLI-focused design |
| Windows-first support | Unix-first, Windows best-effort |

## When to Use Alternatives

### Use BWA-MEM When

- Need paired-end alignment now
- Require BAM/CRAM output
- Processing human genome at scale
- Need BWA compatibility

### Use minimap2 When

- Aligning long reads (PacBio, ONT)
- Need spliced alignment (RNA-seq)
- Cross-species mapping

### Use bwa-rust When

- Learning alignment algorithms
- Developing Rust bioinformatics tools
- Need memory-safe implementation
- Processing single-end data
- Prototyping new methods

## Future Plans

See [ROADMAP.md](https://github.com/LessUp/bwa-rust/blob/master/ROADMAP.md) for planned features.

---

[← Validation](/en/specs/validation) | [Guide →](/en/guide/)
