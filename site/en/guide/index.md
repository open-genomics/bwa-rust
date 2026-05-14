# Guide

This guide covers only the delivered single-end workflow: FASTA reference sequences, FASTQ reads, `.fm` index, and SAM output.

## Standard Path

```bash
bwa-rust index reference.fa -o ref
bwa-rust align -i ref.fm reads.fq -o output.sam
```

You can also use `mem` to build the index in memory and align immediately:

```bash
bwa-rust mem reference.fa reads.fq -t 4 -o output.sam
```

## Parameter Ground Truth

CLI defaults match `src/align/mod.rs` `AlignOpt::default()`. Common parameters:

| Parameter | Default | CLI |
|-----------|---------|-----|
| match score | `2` | `--match` / `-A` |
| mismatch penalty | `1` | `--mismatch` / `-B` |
| gap open | `2` | `--gap-open` / `-O` |
| gap extend | `1` | `--gap-ext` / `-E` |
| band width | `16` | `--band-width` / `-w` |
| score threshold | `20` | `--score-threshold` / `-T` |
| min seed length | `19` | `--min-seed-len` / `-k` |
| z-drop | `100` | `--z-drop` / `-d` |

## Next Steps

- [Installation](/en/guide/installation)
- [Quick Start](/en/guide/quickstart)
- [Alignment Pipeline](/en/architecture/pipeline)
