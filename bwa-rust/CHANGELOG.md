# Changelog

本文件记录 bwa-rust 项目的所有重要变更。

格式基于 [Keep a Changelog](https://keepachangelog.com/)，版本号遵循 [语义化版本](https://semver.org/)。

## [0.1.0] - 2026-02-13

### 新增

- **索引构建** (`index` 子命令)
  - FASTA 解析器（支持多 contig、不同换行符、非标准字符过滤）
  - 倍增法后缀数组（SA）构建
  - BWT 构建
  - FM 索引（C 表、分块 Occ 采样、稀疏 SA 采样）
  - 索引序列化为 `.fm` 文件（含 magic number、版本号、构建元数据）

- **序列比对** (`align` 子命令)
  - SMEM 种子查找（超级最大精确匹配）
  - 种子链构建与过滤（DP + 贪心剥离）
  - 带状仿射间隙 Smith-Waterman 局部对齐
  - 正向 / 反向互补双向比对
  - 多链候选去重、主/次要比对输出
  - MAPQ 估算（基于主次候选得分差）
  - SAM 格式输出（含 header、CIGAR、AS/XS/NM 标签）
  - 多线程并行处理（`--threads` 参数，基于 rayon）

- **工程化**
  - criterion 基准测试
  - GitHub Actions CI（fmt、clippy、test、release build）
  - 架构文档、教程、示例代码

### 详细记录

参见 [changelog/v0.1.0.md](changelog/v0.1.0.md)。
