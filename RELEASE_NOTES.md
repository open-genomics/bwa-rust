# bwa-rust v0.2.0 Release Notes

## 🌏 双语文档支持 / Bilingual Documentation

### English

We are excited to announce bwa-rust v0.2.0 with comprehensive bilingual documentation support!

**Highlights:**
- Complete documentation suite in English and Chinese (中文)
- Professional architecture documentation: Overview, Index Building, Alignment Algorithms
- Step-by-step tutorials: Getting Started, Algorithm Tutorial
- Library usage guides with API reference
- Reorganized docs structure for better navigation

**Key Improvements:**
- Memory protection with configurable limits (`max_occ`, `max_chains`, `max_alignments`)
- Alignment quality fixes: semi-global refinement, candidate filtering, clip penalty
- Input validation: empty sequences, duplicate contig names, thread count
- Error handling improvements with proper error propagation

### 中文

我们很高兴发布 bwa-rust v0.2.0，提供完整的双语文档支持！

**更新亮点：**
- 完整的中英文双语文档
- 专业架构文档：概述、索引构建、比对算法
- 分步教程：快速入门、算法详解
- 库使用指南及 API 参考
- 重新组织的文档结构，便于导航

**主要改进：**
- 内存防护机制：可配置限制 (`max_occ`、`max_chains`、`max_alignments`)
- 比对质量修复：半全局细化、候选过滤、剪切惩罚
- 输入验证：空序列、重复 contig 名称、线程数检查
- 错误处理改进：正确的错误传播

---

## 📋 变更日志 / Changelog

### ✨ Added
- Bilingual documentation (English + Chinese)
- BwaError::validate() for AlignOpt parameter validation
- Memory protection limits: max_occ, max_chains_per_contig, max_alignments_per_read

### 🐛 Fixed  
- Candidate filtering: premature threshold issue fixed
- Semi-global refinement for better CIGAR accuracy
- Clip penalty introduced for candidate ranking
- FASTA header validation
- Empty sequence and duplicate contig name rejection
- Thread pool error handling

### 🔧 Changed
- Code quality: extracted named constants, Copy trait optimizations
- Documentation: comprehensive doc comments added

### ⚡ Performance
- Optimized read/qual string construction

---

## 📦 安装 / Installation

```bash
git clone https://github.com/LessUp/bwa-rust.git
cd bwa-rust
cargo build --release
```

## 🚀 快速开始 / Quick Start

```bash
# Build index
bwa-rust index reference.fa -o ref

# Align reads
bwa-rust mem reference.fa reads.fq -o output.sam

# Multi-threaded
bwa-rust mem ref.fa reads.fq -t 4 -o output.sam
```

---

## 📚 文档链接 / Documentation Links

- [Documentation Home](docs/)
- [Architecture Overview (EN)](docs/architecture/overview.md) / [中文](docs/architecture/overview.zh-CN.md)
- [Getting Started (EN)](docs/tutorial/getting-started.md) / [中文](docs/tutorial/getting-started.zh-CN.md)
- [API Reference (EN)](docs/api/library-usage.md) / [中文](docs/api/library-usage.zh-CN.md)
- [Changelog](CHANGELOG.md)

---

## 🙏 致谢 / Acknowledgments

Inspired by [BWA](https://github.com/lh3/bwa) by Heng Li.
