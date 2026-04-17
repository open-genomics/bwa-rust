# bwa-rust Documentation

> Comprehensive documentation for bwa-rust, available in English and Chinese (中文).

---

## 📖 Quick Links

| Document | Description | 中文 |
|----------|-------------|------|
| [Specifications](../specs/) | **Single Source of Truth** (SDD) | [规范](../specs/README.md) |
| [Getting Started](tutorial/getting-started.md) | Installation and basic usage | [快速入门](tutorial/getting-started.zh-CN.md) |
| [Architecture](architecture/) | Module design and implementation | [架构](architecture/overview.zh-CN.md) |
| [Algorithms](tutorial/algorithms.md) | Core algorithm tutorial | [算法教程](tutorial/algorithms.zh-CN.md) |
| [API Reference](api/) | Library API documentation | [API 文档](api/library-usage.zh-CN.md) |
| [Development](development/) | Development guide | - |

---

## 📂 Documentation Structure

```
docs/
├── README.md              # This index
├── assets/                # Images, diagrams, UML
│
├── tutorial/              # User tutorials
│   ├── README.md
│   ├── getting-started.md
│   ├── getting-started.zh-CN.md
│   ├── algorithms.md
│   └── algorithms.zh-CN.md
│
├── architecture/          # Architecture documentation
│   ├── README.md
│   ├── overview.md
│   ├── overview.zh-CN.md
│   ├── index-building.md
│   ├── index-building.zh-CN.md
│   ├── alignment.md
│   └── alignment.zh-CN.md
│
├── api/                   # API documentation
│   ├── README.md
│   ├── library-usage.md
│   └── library-usage.zh-CN.md
│
└── development/           # Development guides
    ├── README.md
    └── bwa-full-reimplementation-plan.md
```

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

| Language | Status |
|----------|--------|
| **English** | Default documentation |
| **中文 (Chinese)** | Complete translation |

Each document is available in both languages where indicated.

---

## 🤝 Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for contribution guidelines.

For development setup, see [Development Guide](development/README.md).

---

## 📄 License

[MIT License](../LICENSE)
