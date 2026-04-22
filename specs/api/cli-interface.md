# API Specification: CLI Interface

> **Version**: v0.2.0
> **Status**: Stable
> **Last Updated**: 2026-04-16

## Overview

This document defines the command-line interface (CLI) API for bwa-rust.

## Commands

### `index` — Build FM-Index

Build an FM-index from a reference FASTA file.

```bash
bwa-rust index <ref.fa> -o <prefix>
```

**Arguments**:
| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<ref.fa>` | Path | Yes | Reference genome FASTA file |
| `-o, --output` | String | Yes | Output index file prefix |

**Output**:
- `<prefix>.fm` — FM-index file (binary, version 2)

**Exit Codes**:
- `0` — Success
- `1` — Error (invalid input, I/O error, etc.)

**Examples**:
```bash
# Build index with default settings
bwa-rust index ref.fa -o ref

# Output to specific directory
bwa-rust index ref.fa -o /tmp/index/ref
```

---

### `align` — Align Reads to Index

Align FASTQ reads to an existing FM-index.

```bash
bwa-rust align -i <prefix>.fm <reads.fq>
```

**Arguments**:
| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<reads.fq>` | Path | Yes | Input FASTQ file |
| `-i, --index` | Path | Yes | FM-index file path |

**Options**:
| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `-t, --threads` | usize | 1 | Number of threads for parallel alignment |
| `-o, --output` | Path | stdout | Output SAM file path |
| `--max-occ` | usize | 500 | Maximum seed occurrence count |
| `--max-chains` | usize | 5 | Maximum chains per contig |
| `--max-alignments` | usize | 5 | Maximum alignments per read |

**Output**:
- SAM format to stdout or specified output file

**Exit Codes**:
- `0` — Success
- `1` — Error

**Examples**:
```bash
# Single-thread alignment
bwa-rust align -i ref.fm reads.fq

# Multi-thread with 8 threads
bwa-rust align -i ref.fm reads.fq -t 8

# Output to file
bwa-rust align -i ref.fm reads.fq -o output.sam
```

---

### `mem` — One-Step Index and Align

BWA-MEM style: build index and align in one command.

```bash
bwa-rust mem <ref.fa> <reads.fq>
```

**Arguments**:
| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<ref.fa>` | Path | Yes | Reference genome FASTA file |
| `<reads.fq>` | Path | Yes | Input FASTQ file |

**Options**:
| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `-t, --threads` | usize | 1 | Number of threads |
| `-o, --output` | Path | stdout | Output SAM file path |
| `-i, --index-prefix` | String | auto | Index file prefix (auto-generates if not exists) |
| `--max-occ` | usize | 500 | Maximum seed occurrence count |
| `--max-chains` | usize | 5 | Maximum chains per contig |
| `--max-alignments` | usize | 5 | Maximum alignments per read |

**Output**:
- SAM format to stdout or specified output file
- Index file (if not exists): `<auto>.fm`

**Exit Codes**:
- `0` — Success
- `1` — Error

**Examples**:
```bash
# One-step alignment (auto-generate index)
bwa-rust mem ref.fa reads.fq

# With multi-threading
bwa-rust mem ref.fa reads.fq -t 4

# Specify index prefix
bwa-rust mem ref.fa reads.fq -i custom_prefix
```

---

## Scoring Parameters (Currently Not CLI-Configurable)

These parameters are defined in code and may be made configurable in future versions:

| Parameter | Default | Description |
|-----------|---------|-------------|
| `match_score` | 2 | Score for matching bases |
| `mismatch_penalty` | 1 | Penalty for mismatches |
| `gap_open` | 2 | Gap opening penalty |
| `gap_extend` | 1 | Gap extension penalty |
| `band_width` | 16 | SW band width |
| `clip_penalty` | 5 | Penalty for soft-clipped bases |
| `min_seed_len` | 19 | Minimum SMEM seed length |

---

## SAM Output Format

### Header Lines

```
@HD	VN:1.6	SO:unsorted
@SQ	SN:<contig_name>	LN:<contig_length>
@PG	ID:bwa-rust	VN:<version>	CL:<command_line>
```

### Alignment Record Format

```
<qname>	<flag>	<rname>	<pos>	<mapq>	<cigar>	<rnext>	<pnext>	<tlen>	<seq>	<qual>	<tags>
```

**FLAG Values**:
| Flag | Meaning |
|------|---------|
| 0 | Forward alignment, primary |
| 16 | Reverse complement alignment, primary |
| 256 | Secondary alignment |

**Optional Tags**:
| Tag | Type | Description |
|-----|------|-------------|
| `AS:i` | int | Alignment score |
| `XS:i` | int | Suboptimal alignment score |
| `NM:i` | int | Edit distance (number of mismatches + indels) |

---

## Error Handling

All commands return structured error messages:

```
Error: invalid FM index file: BWT symbol out of range at 1234
Error: empty FASTA entry: sequence 3 has no bases
Error: duplicate contig names: chr1 appears twice
Error: invalid thread count: must be >= 1
```

---

## Library API

For programmatic usage, see the [Library Usage Guide](../../docs/api/library-usage.md).

```rust
use bwa_rust::{index::fm::FMIndex, io::fasta::parse_fasta, align::pipeline};

// Load reference
let fasta = parse_fasta("ref.fa")?;

// Build index
let fm = FMIndex::build(&fasta, 64)?;

// Align reads
let opt = AlignOpt::default();
let results = pipeline::align_reads(&fm, &reads, &opt);
```
