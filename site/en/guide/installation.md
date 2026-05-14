# Installation

## Download Release

Download the appropriate platform artifact from GitHub Releases.

| Platform | File |
|----------|------|
| Linux static | `bwa-rust-linux-amd64-static.tar.gz` |
| Linux glibc | `bwa-rust-linux-amd64.tar.gz` |
| macOS Intel | `bwa-rust-macos-amd64.tar.gz` |
| macOS Apple Silicon | `bwa-rust-macos-arm64.tar.gz` |
| Windows | `bwa-rust-windows-amd64.zip` |

Linux example:

```bash
curl -sL https://github.com/LessUp/bwa-rust/releases/latest/download/bwa-rust-linux-amd64-static.tar.gz | tar xz
./bwa-rust --version
```

## Build from Source

```bash
git clone https://github.com/LessUp/bwa-rust.git
cd bwa-rust
cargo build --release
target/release/bwa-rust --version
```

## Unsupported Installation Methods

No Docker image is published, and no Homebrew, Conda, or apt repository is declared. If documentation or third-party pages claim support for these methods, this page takes precedence.
