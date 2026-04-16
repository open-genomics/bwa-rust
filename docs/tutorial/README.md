# Tutorial Documentation

> Step-by-step guides for using and understanding bwa-rust.

---

## Documents

| Document | Description | 中文版本 |
|----------|-------------|----------|
| [Getting Started](./getting-started.md) | Installation, CLI usage, quick examples | [中文](./getting-started.zh-CN.md) |
| [Algorithms](./algorithms.md) | Deep dive into FM-index, SMEM, and alignment | [中文](./algorithms.zh-CN.md) |

---

## Learning Path

### Beginner
1. Read [Getting Started](./getting-started.md)
2. Try the examples in `data/` directory
3. Run `cargo run --example simple_align`

### Intermediate
1. Read [Algorithms](./algorithms.md) for core concepts
2. Study [Architecture Overview](../architecture/) for module design
2. Explore the source code starting from `src/main.rs`

### Advanced
1. Read [Index Building](../architecture/index-building.md) and [Alignment](../architecture/alignment.md)
2. Review [Library Usage](../api/library-usage.md) for programmatic API
3. Check [Development Guide](../development/) for contributing

---

## Quick Links

- **CLI Reference**: See [Getting Started](./getting-started.md#cli-commands)
- **Algorithm Details**: See [Algorithms](./algorithms.md)
- **API Reference**: See [API Documentation](../api/)
