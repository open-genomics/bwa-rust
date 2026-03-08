# Roadmap

> **当前版本：v0.1.0** — 单端 BWA-MEM 风格比对器，所有规划任务已完成。

## 项目目标

实现一个**受 BWA 启发的 Rust 版序列比对器**，在整体结构和算法思想上接近 BWA/BWA-MEM，但**不追求 100% 行为兼容**（命令行选项、索引格式、MAPQ 细节等允许不同）。

## v0.1.0 已完成内容

| 阶段 | 内容 | 状态 |
|------|------|------|
| **0. 项目基线** | 目标定义、测试数据集 (`data/toy.fa`)、开发脚本 (`scripts/`) | ✅ |
| **1. 索引稳定化** | FASTA 解析健壮性、FM 索引序列化（magic + 版本号）、SA/BWT 正确性验证 | ✅ |
| **2. 对齐 MVP** | `AlignOpt` 配置、种子 + 带状 SW 局部对齐、CIGAR/NM 输出 | ✅ |
| **3. BWA-MEM 风格** | SMEM 种子查找、种子链构建与过滤、链→对齐扩展、主/次比对、MAPQ 估算 | ✅ |
| **4. 性能与工程化** | criterion 基准测试、rayon 多线程并行、稀疏 SA 采样、带状 DP 缓冲区复用 | ✅ |
| **5. 文档与维护** | 架构文档、教程、示例代码、GitHub Actions CI | ✅ |

---

## 未来展望

以下是可能的后续发展方向（不在 v0.1.0 范围内）：

| 版本 | 里程碑 | 核心内容 |
|------|--------|----------|
| **0.2.0** | 配对端比对 | PE reads、insert size 估计、mate rescue、proper pair FLAG |
| **0.3.0** | 索引兼容 | 读取 BWA 原生索引（`.bwt/.sa/.pac/.ann/.amb`） |
| **0.4.0** | 输出增强 | BAM 直接输出、排序输出支持 |
| **0.5.0** | 性能飞跃 | Smith-Waterman SIMD 加速、内存映射索引 |
| **1.0.0** | 生产就绪 | 人类基因组级验证通过、API 稳定承诺、完整文档 |

详细的全量复刻设计见 [`docs/plan.md`](docs/plan.md)。

---

## 版本策略

本项目遵循 [语义化版本 (SemVer)](https://semver.org/)：**`MAJOR.MINOR.PATCH`**

- **MAJOR**：`0` → `1` 表示 API 稳定化承诺；后续 MAJOR 升级表示不兼容变更（如索引格式变更）
- **MINOR**：新增功能（PE 比对、BAM 输出等），保持向后兼容
- **PATCH**：Bug 修复、性能微调、文档改进

### `0.x` 阶段

- API 和索引格式**允许不兼容变更**（在 CHANGELOG 中标注 `BREAKING`）
- `.fm` 索引文件的 `version` 字段用于格式兼容检查

### 进入 `1.0.0` 的条件

1. 人类基因组（hg38）级别的正确性验证通过
2. 与 C 版 BWA 的 mapping rate 在合理范围内
3. 公共 API（lib 模式）稳定，不再有频繁破坏性变更
4. 文档和错误处理达到生产级质量

### 索引格式版本

FM 索引文件（`.fm`）有独立的内部版本号（`FM_VERSION`），与软件版本解耦：

- 软件升级时若索引格式未变，用户无需重建索引
- 若索引格式发生不兼容变更，递增 `FM_VERSION` 并在加载时检查，给出明确的错误提示

### 发布流程

- 每次发布使用 `v{version}` 格式的 Git 标签（如 `v0.1.0`）
- 发布前确保：`cargo fmt --check` + `cargo clippy -- -D warnings` + `cargo test` 全部通过
- [CHANGELOG.md](CHANGELOG.md) 同步更新
