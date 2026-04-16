# bwa-rust Documentation

> Comprehensive documentation for bwa-rust, available in English and Chinese (中文).

---

## 📖 Quick Links

| Document | Description | 中文 |
|----------|-------------|------|
| [Getting Started](tutorial/getting-started.md) | Installation and basic usage | [快速入门](tutorial/getting-started.zh-CN.md) |
| [Architecture](architecture/) | Module design and implementation | [架构](architecture/overview.zh-CN.md) |
| [Algorithms](tutorial/algorithms.md) | Core algorithm tutorial | [算法教程](tutorial/algorithms.zh-CN.md) |
| [API Reference](api/) | Library API documentation | [API 文档](api/library-usage.zh-CN.md) |

---

## 📂 Documentation Structure

### Tutorial (教程)

| Document | EN | ZH-CN |
|----------|:--:|:-----:|
| Getting Started | [📄](tutorial/getting-started.md) | [📄](tutorial/getting-started.zh-CN.md) |
| Algorithms | [📄](tutorial/algorithms.md) | [📄](tutorial/algorithms.zh-CN.md) |

### Architecture (架构)

| Document | EN | ZH-CN |
|----------|:--:|:-----:|
| Overview | [📄](architecture/overview.md) | [📄](architecture/overview.zh-CN.md) |
| Index Building | [📄](architecture/index-building.md) | [📄](architecture/index-building.zh-CN.md) |
| Alignment | [📄](architecture/alignment.md) | [📄](architecture/alignment.zh-CN.md) |

### API (应用程序接口)

| Document | EN | ZH-CN |
|----------|:--:|:-----:|
| Library Usage | [📄](api/library-usage.md) | [📄](api/library-usage.zh-CN.md) |

### Development (开发)

| Document | EN |
|----------|:--:|
| Development Guide | [📄](development/README.md) |

---

## 🚀 Quick Start

### Installation

```bash
git clone https://github.com/LessUp/bwa-rust.git
cd bwa-rust
cargo build --release
```

### Build Index

```bash
bwa-rust index reference.fa -o ref
```

### Align Reads

```bash
# One-step (BWA-MEM style)
bwa-rust mem ref.fa reads.fq -o output.sam

# Or two-step
bwa-rust align -i ref.fm reads.fq -o output.sam
```

---

## 📊 Project Overview

bwa-rust is a BWA-MEM style short-read DNA sequence aligner written in Rust.

**Key Features:**
- ✅ FM-index with suffix array and BWT
- ✅ SMEM seed finding
- ✅ Seed chain building with DP scoring
- ✅ Banded Smith-Waterman alignment
- ✅ Multi-threaded via rayon
- ✅ Zero unsafe code
- ✅ Single `.fm` index file

---

## 🌍 Language Support

- **English** — Default documentation
- **中文 (Chinese)** — 完整中文文档

Each document is available in both languages where indicated.

---

## 🤝 Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for contribution guidelines.

For development setup, see [Development Guide](development/README.md).

---

## 📄 License

[MIT License](../LICENSE)
