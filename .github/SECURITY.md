# Security Policy

## Supported Versions

| Version | Supported |
| ------- | --------- |
| 0.1.x   | ✅ Active development |
| < 0.1   | ❌ Not supported |

## Reporting a Vulnerability

We take security seriously. If you discover a security vulnerability, please follow these steps:

### 🔒 Private Reporting (Recommended)

1. **Do NOT open a public issue**
2. Email the maintainer directly or use GitHub's private vulnerability reporting:
   - Go to [Security Advisories](https://github.com/LessUp/bwa-rust/security/advisories)
   - Click "Report a vulnerability"
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

### ⏱️ Response Timeline

| Stage | Timeline |
|-------|----------|
| Initial response | Within 48 hours |
| Vulnerability confirmation | Within 7 days |
| Fix development | Depends on severity |
| Security advisory | After fix is released |

### 🏆 Recognition

Contributors who responsibly disclose security vulnerabilities will be:
- Listed in the security advisory (if desired)
- Credited in CHANGELOG.md (if desired)

## Security Best Practices

When using bwa-rust:

- **Input validation**: Always validate input FASTA/FASTQ files
- **Memory limits**: Use `--max-occ`, `--max-chains`, `--max-alignments` for untrusted input
- **Resource limits**: Consider ulimit for processing untrusted data

## Known Security Considerations

1. **Memory exhaustion**: Large or maliciously crafted input files can cause memory issues
   - Mitigation: Use memory protection parameters

2. **No network access**: bwa-rust does not make network connections
   - Safe to run in isolated environments

3. **No unsafe code**: Project forbids `unsafe` Rust code
   - Memory safety guaranteed by Rust compiler
