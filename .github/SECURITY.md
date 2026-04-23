# Security Policy

## Supported Versions

| Version | Supported |
| ------- | --------- |
| 0.2.x   | ✅ Current |
| < 0.2   | ❌ Not supported |

## Reporting a Vulnerability

**Do NOT open a public issue for security vulnerabilities.**

1. Report privately via [Security Advisories](https://github.com/LessUp/bwa-rust/security/advisories/new)
2. Include:
   - Description and impact
   - Steps to reproduce
   - Suggested fix (if any)

**Response timeline:**
- Initial response: 48 hours
- Confirmation: 7 days
- Fix: Depends on severity

Contributors who responsibly disclose vulnerabilities will be credited in the security advisory and CHANGELOG (if desired).

## Security Best Practices

When using bwa-rust:
- **Input validation**: Validate input FASTA/FASTQ files
- **Memory limits**: Use `--max-occ`, `--max-chains`, `--max-alignments` for untrusted input
- **Resource limits**: Consider ulimit for untrusted data

## Security Considerations

1. **Memory exhaustion**: Large or malicious input can cause OOM → Use memory protection parameters
2. **No network access**: bwa-rust does not make network connections
3. **No unsafe code**: Project forbids `unsafe` Rust → Memory safety guaranteed by compiler
