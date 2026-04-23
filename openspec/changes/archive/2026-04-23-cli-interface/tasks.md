## 1. Index Command

- [x] 1.1 Implement `bwa-rust index <ref.fa> -o <prefix>`
- [x] 1.2 Validate input FASTA file
- [x] 1.3 Output .fm index file
- [x] 1.4 Handle errors with descriptive messages

## 2. Align Command

- [x] 2.1 Implement `bwa-rust align -i <index.fm> <reads.fq>`
- [x] 2.2 Support `-t` thread count option
- [x] 2.3 Support `-o` output file option
- [x] 2.4 Output SAM to stdout or file

## 3. Mem Command

- [x] 3.1 Implement `bwa-rust mem <ref.fa> <reads.fq>`
- [x] 3.2 Auto-build index if not exists
- [x] 3.3 Reuse existing index if available
- [x] 3.4 Support all align options

## 4. Alignment Parameters

- [x] 4.1 Add `--max-occ` option
- [x] 4.2 Add `--max-chains` option
- [x] 4.3 Add `--max-alignments` option

## 5. Error Handling

- [x] 5.1 Report invalid FASTA errors
- [x] 5.2 Report invalid index errors
- [x] 5.3 Report I/O errors
- [x] 5.4 Exit with appropriate codes

## 6. Help and Version

- [x] 6.1 Implement `--help` for all commands
- [x] 6.2 Implement `--version`
