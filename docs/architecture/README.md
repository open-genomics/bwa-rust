# Architecture Documentation

> Technical documentation on bwa-rust's module design, algorithms, and implementation.

---

## Documents

| Document | Description | 中文版本 |
|----------|-------------|----------|
| [Overview](./overview.md) | Module architecture, data flow, tech stack | [中文](./overview.zh-CN.md) |
| [Index Building](./index-building.md) | SA construction, BWT, FM-index, serialization | [中文](./index-building.zh-CN.md) |
| [Alignment](./alignment.md) | SMEM seeds, chaining, Smith-Waterman, pipeline | [中文](./alignment.zh-CN.md) |

---

## Quick Navigation

### For Users
- Start with [Getting Started](../tutorial/getting-started.md)
- Learn about [Algorithms](../tutorial/algorithms.md)

### For Developers
- Read [Overview](./overview.md) for architecture
- Study [Index Building](./index-building.md) for indexing internals
- Review [Alignment](./alignment.md) for alignment algorithms

---

## Architecture Diagram

```
┌──────────────────────────────────────────────────────────────┐
│                      CLI Layer (main.rs)                     │
│               Command parsing with clap + dispatch           │
├──────────────┬──────────────┬───────────────┬────────────────┤
│     io/      │    index/    │    align/     │     util/      │
│     I/O      │   Indexing   │   Alignment   │   Utilities    │
├──────────────┼──────────────┼───────────────┼────────────────┤
│   FASTA      │     SA       │    Seed       │  DNA encoding  │
│   FASTQ      │     BWT      │    Chain      │  Rev comp      │
│   SAM        │     FM       │    SW/Extend  │                │
│              │   Builder    │    Candidate  │                │
│              │              │    MAPQ       │                │
│              │              │    Pipeline   │                │
└──────────────┴──────────────┴───────────────┴────────────────┘
```

---

## Language Support

- **English**: Default documentation language
- **中文**: 完整的中文版本文档，见 [overview.zh-CN.md](./overview.zh-CN.md) 等

---

## See Also

- [API Documentation](../api/) — Library usage guide
- [Development Guide](../development/) — Contributing and development
