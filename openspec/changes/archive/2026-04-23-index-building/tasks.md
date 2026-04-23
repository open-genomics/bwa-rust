## 1. FASTA Parsing

- [x] 1.1 Parse multi-contig FASTA files
- [x] 1.2 Normalize sequences (uppercase, filter non-standard)
- [x] 1.3 Detect duplicate contig names
- [x] 1.4 Reject empty sequences

## 2. Suffix Array Construction

- [x] 2.1 Implement doubling algorithm O(n log²n)
- [x] 2.2 Handle DNA alphabet (6 symbols)
- [x] 2.3 Optimize memory usage

## 3. BWT Construction

- [x] 3.1 Generate BWT from SA
- [x] 3.2 O(n) time complexity

## 4. FM-Index Construction

- [x] 4.1 Build C-table
- [x] 4.2 Build Occ sampling table (block size 64)
- [x] 4.3 Implement backward search
- [x] 4.4 Implement sparse SA sampling

## 5. Index Serialization

- [x] 5.1 Define .fm file format with magic header
- [x] 5.2 Serialize with bincode
- [x] 5.3 Include version number for compatibility
- [x] 5.4 Store contig metadata

## 6. Query Interface

- [x] 6.1 SA position lookup (with backtracking)
- [x] 6.2 Occ query with sampling
- [x] 6.3 LF-mapping
