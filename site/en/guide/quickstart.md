# Quick Start

The repository includes toy data to verify CLI functionality and output format.

## Build Index

```bash
cargo run --release -- index data/toy.fa -o toy
```

Output:

```text
toy.fm
```

## Align Reads

```bash
cargo run --release -- align -i toy.fm data/toy_reads.fq -o toy.sam
```

Or in one step:

```bash
cargo run --release -- mem data/toy.fa data/toy_reads.fq -t 4 -o toy.sam
```

## View SAM

```bash
grep -v '^@' toy.sam
```

Output records contain standard SAM fields plus `AS:i`, `XS:i`, `NM:i`, and optionally `MD:Z` and `SA:Z` tags.

## Cleanup

```bash
rm -f toy.fm toy.sam
```

## Parameter Tuning Example

```bash
bwa-rust align -i toy.fm data/toy_reads.fq \
  --min-seed-len 15 \
  --max-occ 200 \
  --band-width 32 \
  --z-drop 80 \
  -o tuned.sam
```

All defaults follow `AlignOpt::default()`; `mem` and `align` use consistent standard defaults.
