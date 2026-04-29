# Support

## Documentation

- Public docs: <https://lessup.github.io/bwa-rust/>
- Repository README: <https://github.com/LessUp/bwa-rust>
- OpenSpec requirements: <https://github.com/LessUp/bwa-rust/tree/master/openspec/specs>

## Questions And Bugs

- Questions: GitHub Discussions.
- Bugs: GitHub Issues with input files, command, expected behavior, actual behavior, OS, and Rust version.
- Feature requests: label planned capabilities clearly; paired-end and BAM/CRAM are not shipped yet.

## Security

Do not open public issues for vulnerabilities. Use GitHub Security Advisories and see `SECURITY.md`.

## Common Issues

- BWA index files are not compatible; build `.fm` with `bwa-rust index`.
- This is single-end only; paired FASTQ input is not a shipped CLI workflow.
- Use `--max-occ`, `--max-chains`, and `--max-alignments` to reduce repetitive-sequence blowups.
