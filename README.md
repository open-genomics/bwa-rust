# bwa-rust

[![CI](https://github.com/LessUp/bwa-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/LessUp/bwa-rust/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](bwa-rust/LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

**bwa-rust** 是一个受 [BWA](https://github.com/lh3/bwa) 启发的 **Rust 版 DNA 序列比对器**。本项目在整体结构和算法思想上接近 BWA/BWA-MEM，但**不追求与 C 版 BWA 100% 行为兼容**（索引格式、MAPQ 模型等允许不同）。

### 核心特性

- **FM 索引构建**：后缀数组 + BWT + 稀疏 SA 采样，序列化为单一 `.fm` 文件
- **BWA-MEM 风格比对**：SMEM 种子查找 → 种子链构建与过滤 → 带状仿射间隙 Smith-Waterman 局部对齐
- **标准 SAM 输出**：含 CIGAR、MAPQ、AS/XS/NM 标签、主/次要比对 FLAG
- **多线程并行**：基于 rayon 的 reads 级并行处理

---

## 仓库结构

```
.
├── bwa-rust/          # Rust 版比对器（主要代码）
│   ├── src/           #   源代码
│   ├── docs/          #   架构文档、教程
│   ├── data/          #   测试数据（toy.fa / toy_reads.fq）
│   ├── examples/      #   示例代码
│   ├── benches/       #   基准测试
│   └── README.md      #   详细使用说明
├── bwa-0.7.19/        # C 版 BWA 源码（算法参考实现）
├── ROADMAP.md         # 开发路线图（v0.1.0 已全部完成）
└── README.md          # 本文件
```

---

## 快速开始

进入 `bwa-rust/` 目录后：

```bash
# 构建
cargo build --release

# 构建索引
cargo run --release -- index data/toy.fa -o data/toy

# 对齐 reads（SMEM + SW 局部对齐，输出 SAM）
cargo run --release -- align -i data/toy.fm data/toy_reads.fq

# 多线程 + 自定义打分参数
cargo run --release -- align -i data/toy.fm data/toy_reads.fq -t 4 \
    --match 2 --mismatch 1 --gap-open 2 --gap-ext 1
```

更详细的使用说明见 [`bwa-rust/README.md`](bwa-rust/README.md)。

---

## 文档

| 文档 | 说明 |
|------|------|
| [`bwa-rust/README.md`](bwa-rust/README.md) | 使用说明、安装、项目结构 |
| [`bwa-rust/docs/architecture.md`](bwa-rust/docs/architecture.md) | 模块架构、索引格式、算法流程 |
| [`bwa-rust/docs/tutorial.md`](bwa-rust/docs/tutorial.md) | 教程：从 0 实现 BWA 风格对齐器 |
| [`ROADMAP.md`](ROADMAP.md) | 开发路线图（v0.1.0 已完成） |
| [`bwa-rust/CHANGELOG.md`](bwa-rust/CHANGELOG.md) | 版本变更日志 |
| [`bwa-rust/VERSIONING.md`](bwa-rust/VERSIONING.md) | 版本策略与演进路线 |

---

## 开发与贡献

- 开发主要集中在 `bwa-rust/` 目录，详见 [`CONTRIBUTING.md`](bwa-rust/CONTRIBUTING.md)
- `ROADMAP.md` 记录了完整的开发阶段与任务清单
- `bwa-0.7.19/` 内含 C 版 BWA 源码，可作为算法与数据结构的参考

---

## 许可证

本项目采用 [MIT 许可证](bwa-rust/LICENSE) 发布。
