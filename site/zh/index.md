---
layout: home
---

<div class="whitepaper-header">
  <div class="whitepaper-title">bwa-rust</div>
  <div class="whitepaper-subtitle">内存安全的 BWA-MEM 风格单端 DNA 短读比对器</div>
</div>

<div class="whitepaper-intro">
受 BWA-MEM 启发的 Rust 2021 实现，采用单文件 FM-index、零 unsafe 代码、清晰的 seed-chain-extend 流水线。专为学习、扩展和安全敏感的生物信息学实验而设计。
</div>

## 核心价值主张

<div class="value-grid">
  <div class="value-item">
    <div class="value-item-title">📦 单文件 FM-index</div>
    <div class="value-item-desc">后缀数组、BWT、Occ 采样和 contig 元信息统一保存为 <code>.fm</code> 格式——比 BWA 多文件索引更易移动和归档。</div>
  </div>
  <div class="value-item">
    <div class="value-item-title">🔒 零 unsafe 代码</div>
    <div class="value-item-desc"><code>unsafe_code = "forbid"</code> 由 Cargo lint 强制执行，为 Rust 生物信息学实验提供内存安全边界。</div>
  </div>
  <div class="value-item">
    <div class="value-item-title">🧬 清晰流水线</div>
    <div class="value-item-desc">SMEM 种子、DP 链构建、带状 Smith-Waterman 延伸和 SAM 输出分别对应清晰模块——适合学习和二次开发。</div>
  </div>
  <div class="value-item">
    <div class="value-item-title">🎯 诚实范围</div>
    <div class="value-item-desc">已交付单端 FASTQ 到 SAM；配对端和 BAM/CRAM 为计划功能，不伪装为生产就绪。</div>
  </div>
</div>

## 能力矩阵

| 能力 | 状态 | 说明 |
|------|:----:|------|
| FASTA 参考输入 | <span class="status-badge delivered">✓ 已交付</span> | 支持多 contig。 |
| FASTQ 单端 reads | <span class="status-badge delivered">✓ 已交付</span> | 当前稳定数据路径。 |
| `.fm` 索引 | <span class="status-badge delivered">✓ 已交付</span> | 单文件 bincode 格式，magic/version 校验。 |
| SMEM + chaining + SW | <span class="status-badge delivered">✓ 已交付</span> | BWA-MEM 风格，不追求 bit-level 兼容。 |
| SAM 输出 | <span class="status-badge delivered">✓ 已交付</span> | CIGAR、MAPQ、AS/XS/NM、MD:Z、SA:Z。 |
| Rayon 并行 | <span class="status-badge delivered">✓ 已交付</span> | read 级并行。 |
| 配对端比对 | <span class="status-badge planned">📋 计划中</span> | 保留设计与局部基础设施，CLI 未开放。 |
| BAM/CRAM 输出 | <span class="status-badge planned">📋 计划中</span> | 当前只输出 SAM。 |

## 这个项目适合谁

| 用户画像 | 价值主张 |
|----------|----------|
| Rust 生物信息学开发者 | 直接复用 FM-index、SMEM、链构建、SW 和 SAM 组件。 |
| 算法学习者 | 用 Rust 阅读 BWA-MEM 风格核心流程，而不是从大型 C 代码库开始。 |
| 单端 reads 实验 | 需要可配置、可测试、易归档的单端比对基线。 |
| 安全敏感原型 | 需要禁止 `unsafe` 的 DNA 比对实验环境。 |

## 不适合的场景

- 生产级配对端工作流。
- 与 BWA 输出完全一致的兼容性测试。
- 人类基因组规模的成熟生产调度。
- 需要 BAM/CRAM 原生输出的流程。

<hr class="section-divider" />

## 快速开始

<div class="quick-start-block">

### 安装

```bash
cargo install bwa-rust
```

### 构建索引

```bash
bwa-rust index reference.fasta -o reference.fm
```

### 比对 reads

```bash
bwa-rust align reference.fm reads.fastq -o output.sam
```

</div>

## 入口

- [安装与快速开始](/zh/guide/) — 快速上手
- [核心架构](/zh/architecture/) — 理解设计
- [性能与验证边界](/zh/benchmarks) — 性能数据
- [常见问题](/zh/faq) — 常见问题解答

<div style="text-align: center; margin-top: 2rem; color: var(--vp-c-text-3); font-size: 13px;">
  <a href="https://docs.rs/bwa-rust" target="_blank">API 参考 (docs.rs)</a> ·
  <a href="https://github.com/LessUp/bwa-rust" target="_blank">GitHub 仓库</a>
</div>
