---
layout: home

hero:
  name: 'bwa-rust'
  text: '内存安全的 BWA-MEM 风格单端比对器'
  tagline: 'Rust 2021 实现，从 FASTA/FASTQ 到 FM-index、SMEM、链、Smith-Waterman 和 SAM。'
  actions:
    - theme: brand
      text: '60 秒跑通'
      link: '/guide/quickstart'
    - theme: alt
      text: '架构导览'
      link: '/architecture/'
    - theme: alt
      text: 'GitHub'
      link: 'https://github.com/LessUp/bwa-rust'

features:
  - title: '单文件 FM-index'
    details: '后缀数组、BWT、Occ 采样和 contig 元信息统一保存为 .fm，比 BWA 多文件索引更易移动和归档。'
  - title: '清晰的 seed-chain-extend 流水线'
    details: 'SMEM 种子、DP 链构建、带状 Smith-Waterman 延伸和 SAM 输出分别对应清晰模块，适合学习和二次开发。'
  - title: '零 unsafe 代码'
    details: 'unsafe_code = forbid 由 Cargo lint 强制，面向需要内存安全边界的 Rust 生物信息学实验。'
  - title: '当前范围诚实'
    details: '已交付单端 FASTQ 到 SAM；配对端和 BAM/CRAM 仍为计划项，不在标准用法中伪装成可用能力。'
---

## 这个项目适合谁

| 用户 | 价值 |
|------|------|
| Rust 生物信息学开发者 | 直接复用 FM-index、SMEM、链构建、SW 和 SAM 组件。 |
| 算法学习者 | 用 Rust 阅读 BWA-MEM 风格核心流程，而不是从大型 C 代码库开始。 |
| 单端 reads 实验 | 需要可配置、可测试、易归档的单端比对基线。 |
| 安全敏感原型 | 需要禁止 `unsafe` 的 DNA 比对实验环境。 |

## 能力矩阵

| 能力 | 状态 | 说明 |
|------|------|------|
| FASTA 参考输入 | 已交付 | 支持多 contig。 |
| FASTQ 单端 reads | 已交付 | 当前稳定数据路径。 |
| `.fm` 索引 | 已交付 | 单文件 bincode 格式，magic/version 校验。 |
| SMEM + chaining + SW | 已交付 | BWA-MEM 风格，不追求 bit-level 兼容。 |
| SAM 输出 | 已交付 | CIGAR、MAPQ、AS/XS/NM、MD:Z、SA:Z。 |
| Rayon 并行 | 已交付 | read 级并行。 |
| 配对端 | 计划中 | 保留设计与局部基础设施，CLI 未开放。 |
| BAM/CRAM | 计划中 | 当前只输出 SAM。 |

## 不适合的场景

- 生产级配对端工作流。
- 与 BWA 输出完全一致的兼容性测试。
- 人类基因组规模的成熟生产调度。
- 需要 BAM/CRAM 原生输出的流程。

## 入口

- [安装与快速开始](/guide/)
- [核心架构](/architecture/)
- [性能与验证边界](/benchmarks)
- [常见问题](/faq)
