# RustBwa

RustBwa 是一个 **受 BWA 启发的 Rust 版序列比对器项目**。本仓库同时包含：

- Rust 实现的比对器原型（`bwa-rust/`）
- 作为参考实现的 C 版 BWA 源码（`bwa-0.7.19/`）
- 项目路线图与规划文档（`ROADMAP.md`）

> 说明：本项目 **不追求与 C 版 BWA 完全兼容**，而是在整体结构和算法思想上向 BWA/BWA-MEM 靠拢。

---

## 仓库结构

- `bwa-rust/`
  - Rust 版 BWA 的主要代码所在目录
  - 已实现：
    - `index` 子命令：读取 FASTA，构建基于后缀数组 + FM 的索引，并序列化为 `.fm` 文件
    - `align` 子命令：加载 `.fm`，对 FASTQ 进行精确匹配，对齐结果输出为简化 SAM
  - 更多使用说明与开发计划见 `bwa-rust/README.md`

- `bwa-0.7.19/`
  - 原始 C 版 BWA 0.7.19 源码，仅作为算法与结构的参考实现
  - 可用于对比测试与行为参考

- `ROADMAP.md`
  - 项目整体的阶段划分与 TODO 列表
  - 包含从索引模块、对齐 MVP 到 BWA-MEM 风格对齐的详细规划

---

## 快速开始（Rust 版）

进入 `bwa-rust/` 目录后：

```bash
# 构建
cargo build

# 运行 index 子命令（构建索引）
cargo run -- index /path/to/ref.fa -o ref

# 运行 align 子命令（使用已有索引进行精确匹配对齐）
# 具体参数与功能以 bwa-rust/README.md 为准
cargo run -- align ref.fm /path/to/reads.fq -o out.sam
```

更详细的使用说明和开发路线请查看：

- `bwa-rust/README.md`
- `ROADMAP.md`

---

## 开发与贡献

- 建议先阅读 `ROADMAP.md` 了解当前阶段目标和 TODO
- 开发主要集中在 `bwa-rust/` 目录
- 可参考 `bwa-0.7.19/` 中的实现细节来设计或优化 Rust 版本的算法与数据结构
