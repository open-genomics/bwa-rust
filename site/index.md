---
layout: home

hero:
  name: bwa-rust
  text: Rust 版 BWA 序列比对器
  tagline: 受 BWA/BWA-MEM 启发，使用 Rust 实现的高性能 DNA 序列比对工具
  actions:
    - theme: brand
      text: 快速开始
      link: /guide/getting-started
    - theme: alt
      text: 架构设计
      link: /guide/architecture
    - theme: alt
      text: GitHub
      link: https://github.com/LessUp/bwa-rust

features:
  - icon: 🧬
    title: FM 索引构建
    details: 后缀数组 + BWT + 稀疏 SA 采样，序列化为单一 .fm 文件，含 magic number 与版本兼容性检查
  - icon: 🎯
    title: BWA-MEM 风格比对
    details: SMEM 种子查找 → 种子链构建与过滤 → 带状仿射间隙 Smith-Waterman 局部对齐
  - icon: 📄
    title: 标准 SAM 输出
    details: 含 @HD/@SQ/@PG header、CIGAR、MAPQ、AS/XS/NM 标签、主/次要比对 FLAG
  - icon: ⚡
    title: 多线程并行
    details: 基于 rayon 的 reads 级并行处理，充分利用多核性能
  - icon: 🦀
    title: Rust 内存安全
    details: 零 unsafe 代码，编译期保证内存安全；jemalloc 分配器提升多线程吞吐
  - icon: 🧪
    title: 133 项测试全通过
    details: 121 单元测试 + 11 集成测试 + 1 文档测试，criterion 基准测试，GitHub Actions CI 持续集成
---
