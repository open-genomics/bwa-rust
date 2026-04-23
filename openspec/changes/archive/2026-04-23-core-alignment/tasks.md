## 1. SMEM Seed Finding

- [x] 1.1 Implement backward search for exact matches
- [x] 1.2 Implement right extension for longest match
- [x] 1.3 Implement left extension for maximality
- [x] 1.4 Add max_occ filtering for repetitive seeds

## 2. Chain Building

- [x] 2.1 Implement DP chain scoring
- [x] 2.2 Implement greedy peeling for multi-chain extraction
- [x] 2.3 Add chain filtering by score threshold
- [x] 2.4 Add max_chains_per_contig limiting

## 3. Smith-Waterman Alignment

- [x] 3.1 Implement banded SW with affine gap
- [x] 3.2 Generate CIGAR string from traceback
- [x] 3.3 Compute NM (edit distance) tag
- [x] 3.4 Implement SwBuffer for memory reuse

## 4. Candidate Management

- [x] 4.1 Implement position-based deduplication
- [x] 4.2 Implement score-based sorting
- [x] 4.3 Mark primary and secondary alignments
- [x] 4.4 Add max_alignments_per_read limiting

## 5. MAPQ Estimation

- [x] 5.1 Implement score-difference based MAPQ
- [x] 5.2 Handle unique alignment case

## 6. SAM Output

- [x] 6.1 Generate SAM header (@HD, @SQ, @PG)
- [x] 6.2 Format alignment records with all fields
- [x] 6.3 Include AS:i, XS:i, NM:i tags
- [x] 6.4 Handle unmapped reads

## 7. Multi-threading

- [x] 7.1 Implement rayon-based parallel processing
- [x] 7.2 Add thread count configuration
