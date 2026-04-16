# 路线图

> **当前版本：v0.1.0** — 单端 BWA-MEM 风格比对器，所有规划任务已完成。

---

## 项目目标

实现一个**受 BWA 启发的 Rust 版序列比对器**：

- 整体结构和算法思想接近 BWA/BWA-MEM
- **不追求 100% 行为兼容**
- 优先保证正确性、可读性和内存安全

---

## v0.1.0 ✅ 已完成

| 阶段 | 内容 | 状态 |
|------|------|:----:|
| **项目基线** | 目标定义、测试数据集、开发脚本 | ✅ |
| **索引稳定化** | FASTA 解析、FM 索引序列化、SA/BWT 正确性验证 | ✅ |
| **对齐 MVP** | 对齐配置、种子 + 带状 SW、CIGAR/NM 输出 | ✅ |
| **BWA-MEM 风格** | SMEM 种子、种子链、链扩展、MAPQ 估算 | ✅ |
| **性能工程化** | 基准测试、多线程并行、稀疏 SA、缓冲区复用 | ✅ |
| **文档维护** | 架构文档、教程、示例代码、CI | ✅ |

### v0.1.0 后续改进

| 改进项 | 说明 | 状态 |
|--------|------|:----:|
| 内存防护 | `max_occ`、`max_chains`、`max_alignments` | ✅ |
| 对齐质量 | semi-global refinement、clip penalty 排序 | ✅ |
| 输入校验 | FASTA 错误检测、参数验证 | ✅ |
| 代码质量 | 命名常量、API 文档注释 | ✅ |

---

## 未来规划

| 版本 | 里程碑 | 核心内容 |
|------|--------|----------|
| **v0.2.0** | 配对端比对 | PE reads、insert size 估计、mate rescue |
| **v0.3.0** | 索引兼容 | 读取 BWA 原生索引文件 |
| **v0.4.0** | 输出增强 | BAM 直接输出、排序输出 |
| **v0.5.0** | 性能飞跃 | SIMD 加速、内存映射索引 |
| **v1.0.0** | 生产就绪 | 人类基因组验证、API 稳定 |

详细设计见 [`docs/plan.md`](https://github.com/LessUp/bwa-rust/blob/main/docs/plan.md)。

---

## 版本策略

本项目遵循 [语义化版本 (SemVer)](https://semver.org/)：**`MAJOR.MINOR.PATCH`**

| 部分 | 含义 |
|------|------|
| **MAJOR** | 不兼容变更（API、索引格式） |
| **MINOR** | 新功能，向后兼容 |
| **PATCH** | Bug 修复、性能微调 |

### `0.x` 阶段规则

- API 和索引格式**允许不兼容变更**
- 破坏性变更在 CHANGELOG 标注 `BREAKING`
- `.fm` 索引 `version` 字段用于兼容性检查

### 进入 `1.0.0` 的条件

1. ✅ 人类基因组（hg38）级别正确性验证通过
2. ✅ 与 C 版 BWA 的 mapping rate 在合理范围
3. ✅ 公共 API 稳定
4. ✅ 文档和错误处理达到生产级质量

---

## 索引格式版本

| 版本 | 变更 |
|------|------|
| v1 | 初始格式 |
| v2 | 添加 `IndexMeta` 构建元数据 |

- 软件升级若索引格式未变 → 无需重建索引
- 索引格式不兼容变更 → 递增版本号，加载时报错

---

## 发布流程

```bash
# 代码质量检查
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features

# 发布构建
cargo build --release

# 创建标签
git tag v0.x.x
git push origin v0.x.x
```

---

## 测试覆盖

| 类型 | 数量 |
|------|------|
| 单元测试 | 151 |
| 集成测试 | 11 |
| 模块测试 | 5 |
| 文档测试 | 1 |
| **总计** | **168** |

CI 流程：GitHub Actions（fmt → clippy → test → release）
