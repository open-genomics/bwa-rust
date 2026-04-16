---
layout: home

hero:
  name: bwa-rust
  text: Rust 版 BWA 序列比对器
  tagline: 受 BWA/BWA-MEM 启发，使用 Rust 实现的高性能 DNA 序列比对工具
  image:
    src: /logo.svg
    alt: bwa-rust logo
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
    link: /guide/architecture#索引构建
  - icon: 🎯
    title: BWA-MEM 风格比对
    details: SMEM 种子查找 → 种子链构建与过滤 → 带状仿射间隙 Smith-Waterman 局部对齐 → semi-global refinement
    link: /guide/architecture#比对算法
  - icon: 📄
    title: 标准 SAM 输出
    details: 含 @HD/@SQ/@PG header、CIGAR、MAPQ、AS/XS/NM 标签、主/次要比对 FLAG，完全符合 SAM 规范
    link: /guide/getting-started#cli-参数详解
  - icon: ⚡
    title: 多线程并行
    details: 基于 rayon 的 reads 级并行处理，自定义线程池避免全局竞争，充分利用多核性能
    link: /guide/architecture#性能优化
  - icon: 🛡️
    title: 内存安全防护
    details: max_occ 过滤重复种子、max_chains 限制候选数、max_alignments 控制输出，防止内存爆炸
    link: /guide/architecture#内存防护机制
  - icon: 🦀
    title: Rust 内存安全
    details: 零 unsafe 代码，编译期保证内存安全；jemalloc 分配器提升多线程吞吐
    link: /guide/architecture#安全性保证
  - icon: 🧪
    title: 168 项测试全通过
    details: 151 单元测试 + 11 集成测试 + 5 模块测试 + 1 文档测试，criterion 基准测试，GitHub Actions CI
    link: /roadmap#测试覆盖
  - icon: 📦
    title: 跨平台支持
    details: 支持 Linux (静态/动态链接)、macOS (Intel/ARM)、Windows，提供预编译二进制下载
    link: https://github.com/LessUp/bwa-rust/releases
---

<script setup>
import { VPButton } from 'vitepress/theme'

const features = [
  { title: '快速', value: 'O(n log²n)' },
  { title: '准确', value: 'BWA-MEM 风格' },
  { title: '安全', value: '零 unsafe' },
]
</script>

<div class="stats-container">
  <div v-for="feat in features" class="stat-item">
    <div class="stat-value">{{ feat.value }}</div>
    <div class="stat-label">{{ feat.title }}</div>
  </div>
</div>

<style>
.stats-container {
  display: flex;
  justify-content: center;
  gap: 3rem;
  margin: 2rem 0;
  padding: 1.5rem;
  border-radius: 12px;
  background: var(--vp-c-bg-soft);
}

.stat-item {
  text-align: center;
}

.stat-value {
  font-size: 1.5rem;
  font-weight: 600;
  color: var(--vp-c-brand-1);
}

.stat-label {
  font-size: 0.9rem;
  color: var(--vp-c-text-2);
  margin-top: 0.25rem;
}

@media (max-width: 640px) {
  .stats-container {
    flex-direction: column;
    gap: 1rem;
  }
}
</style>
