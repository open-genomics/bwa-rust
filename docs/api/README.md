# API Documentation

> API reference for using bwa-rust as a library.

---

## Documents

| Document | Description | 中文版本 |
|----------|-------------|----------|
| [Library Usage](./library-usage.md) | Using bwa-rust programmatically in Rust | [中文](./library-usage.zh-CN.md) |

---

## Quick Reference

### Core Modules

| Module | Purpose |
|--------|---------|
| `index` | Index building and operations |
| `align` | Alignment algorithms |
| `io` | FASTA/FASTQ/SAM parsing |
| `util` | DNA utilities |

### Main Types

| Type | Module | Description |
|------|--------|-------------|
| `FMIndex` | `index::fm` | FM-index data structure |
| `AlignOpt` | `align` | Alignment configuration |
| `Contig` | `index::fm` | Contig metadata |

---

## Example

```rust
use bwa_rust::index::fm::FMIndex;
use bwa_rust::util::dna;

fn main() {
    let fm = FMIndex::load("ref.fm").unwrap();
    
    let pattern: Vec<u8> = b"ACGT".iter()
        .map(|&b| dna::to_alphabet(b))
        .collect();
    
    if let Some((l, r)) = fm.backward_search(&pattern) {
        println!("Found {} occurrences", r - l);
    }
}
```

---

## Module Documentation

For detailed module docs, run:

```bash
cargo doc --open
```

---

## See Also

- [Tutorial](../tutorial/) — User guides
- [Architecture](../architecture/) — Implementation details
