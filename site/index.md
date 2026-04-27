---
layout: home

hero:
  name: bwa-rust
  text: BWA-MEM 算法的 Rust 实现
  tagline: 零 unsafe 代码 · 清晰架构 · 内存安全基线
  image:
    src: /logo.svg
    alt: bwa-rust logo
  actions:
    - theme: brand
      text: 快速开始
      link: /guide/quickstart
    - theme: alt
      text: 架构文档
      link: /architecture/
    - theme: alt
      text: GitHub
      link: https://github.com/LessUp/bwa-rust

features:
  - icon: 🛡️
    title: 内存安全
    details: 100% Rust 实现，编译期内存安全保证，零 unsafe 代码，forbid(unsafe_code) lint 强制验证
  - icon: 📖
    title: 可读代码
    details: 教育级代码质量，清晰的模块架构，适合学习 BWA-MEM 算法实现细节与生物信息学
  - icon: ⚡
    title: 多线程
    details: 基于 Rayon 的 reads 级并行，jemalloc 分配器，充分利用现代多核 CPU
  - icon: 📦
    title: 单文件索引
    details: 索引存储为单个 .fm 文件，内置版本管理和元数据，便于分发与管理
  - icon: 🧬
    title: 标准输出
    details: 完整 SAM 格式输出（CIGAR, MAPQ, AS/XS/NM），与下游工具无缝集成
  - icon: 🔧
    title: 可编程
    details: 既可作为命令行工具使用，也可作为 Rust 库集成到自定义流程中
---

## 项目定位

bwa-rust 是 BWA-MEM 算法的 Rust 重实现，提供：

- **稳定的单端比对基线** — 当前专注于 SE reads，PE 支持计划中
- **内存安全保证** — 零 unsafe 代码，适合处理不可信数据源
- **Rust 库集成** — 作为依赖嵌入到 Rust 生物信息学流程
- **算法学习资源** — 教育级代码，理解 FM-index、SMEM、Smith-Waterman

**不适合场景**：需要 100% BWA 兼容性或配对端支持的生产流程（使用原版 BWA）

---

## 安装

### 从源码构建（推荐）

```bash
git clone https://github.com/LessUp/bwa-rust.git
cd bwa-rust && cargo build --release
./target/release/bwa-rust --version
```

### Cargo 安装

```bash
cargo install bwa-rust
```

### 预编译二进制

从 [GitHub Releases](https://github.com/LessUp/bwa-rust/releases) 下载对应平台的二进制文件。

---

## 快速使用

```bash
# 1. 构建索引
bwa-rust index reference.fa -o ref

# 2. 比对序列（BWA-MEM 风格）
bwa-rust mem ref.fa reads.fq -t 8 -o output.sam
```

---

## 性能概览

基于 E. coli K-12（4.6Mbp）的小规模测试：

| 指标 | BWA-MEM | bwa-rust | 相对性能 |
|------|---------|----------|---------|
| 索引构建 | ~2s | ~3s | ~67% |
| 单线程比对 | ~10K reads/s | ~7K reads/s | ~70% |
| 8线程比对 | ~35K reads/s | ~25K reads/s | ~71% |

> 小规模测试数据，人类基因组规模测试计划中。实际性能取决于数据特征和硬件配置。

---

## 技术对比

| 特性 | BWA (C) | bwa-rust |
|------|---------|----------|
| 实现语言 | C | Rust |
| 内存安全 | 手动管理 | 编译器验证（零 unsafe） |
| 索引格式 | 多文件 (.bwt/.sa/.pac) | 单文件 (.fm) |
| 线程库 | pthread | rayon |
| 代码可读性 | 优化为主 | 教育为主 |
| 配对端支持 | ✅ | 🚧 计划中 |

**算法**：遵循 BWA-MEM 核心思想（SMEM 种子查找、DP 链构建、Smith-Waterman 对齐），但不追求 100% 行为兼容。

---

## 项目状态

| 版本 | v0.2.0 |
|------|--------|
| 单元测试 | 188 个 ✅ |
| 集成测试 | 11 个 ✅ |
| 文档测试 | 2 个 ✅ |
| CI/CD | GitHub Actions ✅ |
| 文档 | 架构文档、规范文档、VitePress 站点 |
| 配对端支持 | 🚧 v0.3.0 计划中 |

---

## 了解更多

<div style="margin-top: 2rem; display: flex; gap: 1rem; flex-wrap: wrap;">
  <a href="/bwa-rust/architecture/" style="padding: 0.625rem 1.25rem; background: var(--vp-c-brand-1); color: white; border-radius: 8px; text-decoration: none; font-weight: 500;">架构文档</a>
  <a href="/bwa-rust/guide/" style="padding: 0.625rem 1.25rem; border: 1px solid var(--vp-c-border); border-radius: 8px; text-decoration: none; font-weight: 500;">使用指南</a>
  <a href="/bwa-rust/benchmarks" style="padding: 0.625rem 1.25rem; border: 1px solid var(--vp-c-border); border-radius: 8px; text-decoration: none; font-weight: 500;">性能基准</a>
  <a href="/bwa-rust/faq" style="padding: 0.625rem 1.25rem; border: 1px solid var(--vp-c-border); border-radius: 8px; text-decoration: none; font-weight: 500;">常见问题</a>
</div>
