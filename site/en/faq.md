# FAQ

## Can bwa-rust read BWA indices directly?

No. bwa-rust uses its own single-file `.fm` index format, which must be built with `bwa-rust index`.

## Is output identical to BWA?

Not guaranteed. bwa-rust adopts BWA-MEM style seeding, chaining, and extension ideas, but the index format, MAPQ, heuristics, and some tie-breaking are project-specific implementations.

## Is paired-end supported?

The current stable CLI only supports single-end FASTQ. The repository contains paired-end reader and insert-size infrastructure, but this should not be treated as delivered capability.

## Is BAM or CRAM supported?

No. Current output format is SAM.

## Why keep complete reference text in the index?

Alignment extension requires reference sequence fragments. The current version saves encoded text for implementation clarity and simple reading; future versions may evaluate compression or on-demand loading.

## Where can I see default parameters?

`src/align/mod.rs` `AlignOpt::default()` is the single source of truth. `align` and `mem` standard defaults should match it.

## Why was the documentation site Chinese-only?

Previously, Pages was the Chinese public portal, with English entry points in `README.md` and docs.rs. Unmaintained English content was not disguised as existing. Now bilingual support is provided.
