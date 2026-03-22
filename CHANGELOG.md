# Changelog

本文件记录 bwa-rust 项目的所有重要变更。

格式基于 [Keep a Changelog](https://keepachangelog.com/)，版本号遵循 [语义化版本](https://semver.org/)。

## [Unreleased]

### 改进

- **比对质量修正**
  - 修复正向/反向候选在排序前提前做阈值判断，导致强反向命中被误判为 unmapped 的问题
  - 为链候选增加局部窗口全长重比对（semi-global refinement），改善 mismatch / indel 的 CIGAR 与 NM
  - 引入软剪切惩罚参与候选排序，避免免费 soft-clip 长期压过真实的单碱基 indel
  - `mem` 样例中的 insertion / deletion read 现在会输出真实 `I/D`，不再被假全长 `M` 或过度软剪切掩盖

- **输入校验增强**
  - FASTA header 缺少序列名时直接报错
  - FASTA 空序列和重复 contig 名在建索引阶段拒绝通过
  - `--threads 0` 现在会在 CLI 层直接报错，而不是静默回退到默认线程池

- **GitHub Pages 工作流优化**
  - 修复 paths 触发器引用错误的文件名（`docs.yml` → `pages.yml`）
  - 添加 sparse-checkout，仅拉取 `site/` 和 `package.json`，跳过源码和构建产物
  - Node.js 20 → 22（当前 LTS），启用 npm 缓存加速依赖安装

- **VitePress 配置增强**
  - 启用 `cleanUrls`（去除 `.html` 后缀）
  - 启用根级 `lastUpdated`（显示页面最后更新时间）
  - 添加 sitemap 生成（SEO）
  - 添加 Open Graph 元标签（社交分享预览）
  - 添加 `theme-color` 元标签

- **文档站内容丰富**
  - 首页新增"架构设计"快捷入口按钮
  - 首页新增"Rust 内存安全"和"133 项测试全通过"两个特性卡片
  - 丰富已有特性卡片描述（SAM header、magic number 等）
  - 中英文首页同步更新

- **README 徽章补齐**
  - 英文 README 新增 Docs、Rust 版本徽章，与中文 README 保持一致

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
