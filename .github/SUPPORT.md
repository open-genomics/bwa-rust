# Support

## 📖 Documentation

- [Online Docs](https://lessup.github.io/bwa-rust/)
- [Architecture](https://github.com/LessUp/bwa-rust/tree/main/docs/architecture)
- [Tutorial](https://github.com/LessUp/bwa-rust/tree/main/docs/tutorial)
- [OpenSpec Specifications](https://github.com/LessUp/bwa-rust/tree/main/openspec/specs)

## 💬 Community

- **Questions**: [GitHub Discussions](https://github.com/LessUp/bwa-rust/discussions)
- **Bug Reports**: Use [Bug Report template](https://github.com/LessUp/bwa-rust/issues/new?template=bug_report.md)
- **Feature Requests**: Use [Feature Request template](https://github.com/LessUp/bwa-rust/issues/new?template=feature_request.md)

## 🔒 Security

For security vulnerabilities, **do not** open a public issue.

Report privately via [Security Advisories](https://github.com/LessUp/bwa-rust/security/advisories). See [SECURITY.md](SECURITY.md) for details.

## 🛠️ Common Issues

**Build failures on jemalloc**: jemalloc is disabled on Windows. On Linux/macOS, ensure build tools are installed.

**FASTA parse errors**: Check file format, sequence names, and for duplicate contig names.

**Memory issues**: Use memory protection parameters:
```bash
bwa-rust mem ref.fa reads.fq --max-occ 200 --max-chains 3 --max-alignments 3
```

## 📋 Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md).

---

**Note**: This is an open-source project maintained in spare time. Please be patient.
