# Alignment Pipeline

## 1. Input Normalization

FASTA and FASTQ inputs are uniformly mapped to uppercase DNA letters. Unknown bases map to `N`, with `$` reserved as sentinel in the internal alphabet.

## 2. Forward/Reverse Candidates

Each read is tried in both forward and reverse complement orientations. If the output FLAG contains `0x10`, SAM `SEQ` is the reverse complement sequence, with `QUAL` reversed accordingly.

## 3. Seeds and Chains

SMEM seeds pass through occurrence filtering before chain building. Chain building groups by contig and keeps a limited number of high-scoring chains, preventing candidate explosion from repetitive sequences.

## 4. Extension and Refinement

Each chain is first converted to approximate alignment, then semi-global refinement is attempted. Candidate sorting uses raw score, soft-clip penalty, NM, contig, position, and orientation for stable tie-breaking.

## 5. Output Control

`max_alignments_per_read` controls output record count per read. Candidates below `score_threshold` are not output; no valid candidates result in an unmapped record.

## 6. SAM Auxiliary Tags

- `AS:i`: Current alignment score.
- `XS:i`: Suboptimal alignment score.
- `NM:i`: Edit distance-like mismatch/indel count.
- `MD:Z`: Reference-side mismatch/deletion description.
- `SA:Z`: Chimeric/supplementary alignment description.

For soft-clipped reads, MD:Z generation uses query slices consistent with complete CIGAR coordinates, avoiding coordinate misalignment from soft clips.
