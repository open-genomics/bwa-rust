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
      text: GitHub
      link: https://github.com/LessUp/bwa-rust

features:
  - icon: 🧬
    title: FM 索引构建
    details: 后缀数组 + BWT + 稀疏 SA 采样，序列化为单一 .fm 文件
  - icon: 🎯
    title: BWA-MEM 风格比对
    details: SMEM 种子查找 → 种子链构建与过滤 → 带状仿射间隙 Smith-Waterman 局部对齐
  - icon: 📄
    title: 标准 SAM 输出
    details: 含 CIGAR、MAPQ、AS/XS/NM 标签、主/次要比对 FLAG
  - icon: ⚡
    title: 多线程并行
    details: 基于 rayon 的 reads 级并行处理，充分利用多核性能
---
