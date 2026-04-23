# Roadmap

> **当前版本：v0.2.0** — 单端 BWA-MEM 风格比对器，双语文档支持，内存防护机制。

---

## 项目目标

实现一个**受 BWA 启发的 Rust 版序列比对器**：

- 整体结构和算法思想接近 BWA/BWA-MEM
- **不追求 100% 行为兼容**（CLI 选项、索引格式、MAPQ 细节等允许不同）
- 优先保证正确性、可读性和内存安全

---

## v0.1.0 ✅ 已完成

| 阶段 | 内容 | 状态 |
|------|------|:----:|
| **项目基线** | 目标定义、测试数据集 (`data/toy.fa`)、开发脚本 | ✅ |
| **索引稳定化** | FASTA 解析健壮性、FM 索引序列化（magic + 版本号）、SA/BWT 正确性验证 | ✅ |
| **对齐 MVP** | `AlignOpt` 配置、种子 + 带状 SW 局部对齐、CIGAR/NM 输出 | ✅ |
| **BWA-MEM 风格** | SMEM 种子查找、种子链构建与过滤、链→对齐扩展、主/次比对、MAPQ 估算 | ✅ |
| **性能工程化** | criterion 基准测试、rayon 多线程并行、稀疏 SA 采样、DP 缓冲区复用 | ✅ |
| **文档维护** | 架构文档、教程、示例代码、GitHub Actions CI | ✅ |

### 核心功能

```
┌─────────────────────────────────────────────────────────────┐
│                     v0.1.0 功能总览                          │
├─────────────────────────────────────────────────────────────┤
│  📦 索引构建                                                 │
│     • FASTA 解析（多 contig、自动归一化）                      │
│     • 后缀数组（倍增法 O(n log²n)）                           │
│     • BWT + FM Index（C 表 + Occ 采样）                      │
│     • 序列化为单一 .fm 文件                                   │
├─────────────────────────────────────────────────────────────┤
│  🔬 序列比对                                                 │
│     • SMEM 种子查找（max_occ 过滤）                           │
│     • 种子链构建（DP 评分 + 贪心剥离）                         │
│     • 带状 Smith-Waterman（仿射间隙）                         │
│     • 双向比对（正向 + 反向互补）                              │
│     • 多线程并行（rayon）                                     │
├─────────────────────────────────────────────────────────────┤
│  📄 输出                                                     │
│     • 完整 SAM 格式（header + record）                        │
│     • CIGAR、MAPQ、AS/XS/NM 标签                             │
│     • 主/次比对输出                                          │
└─────────────────────────────────────────────────────────────┘
```

---

## v0.2.0 特性状态清单

### 已实现核心功能（当前版本 v0.2.0）

#### F-001: FM-Index Based Sequence Alignment ✅
- ✅ Build FM-index from FASTA reference genome
- ✅ Support single-end read alignment from FASTQ input
- ✅ Output SAM format alignment results
- ✅ Handle multi-contig reference genomes
- ✅ Support reverse complement alignment

#### F-002: SMEM Seed Finding ✅
- ✅ Find longest exact match covering each read position
- ✅ Support left extension until no longer maximal
- ✅ Filter seeds with occurrence count exceeding max_occ (default: 500)

#### F-003: Seed Chain Building ✅
- ✅ DP-based chain scoring with gap penalties
- ✅ Greedy peeling for multi-chain extraction
- ✅ Filter low-score chains below threshold
- ✅ Limit chains per contig (max_chains_per_contig, default: 5)

#### F-004: Banded Smith-Waterman Alignment ✅
- ✅ Banded SW with configurable band width (default: 16)
- ✅ Affine gap penalty (gap open + gap extend)
- ✅ Generate CIGAR string from alignment
- ✅ Compute NM (edit distance) tag
- ✅ Semi-global refinement for edge cases

#### F-005: SAM Output ✅
- ✅ Generate @HD header line (VN:1.6, SO:unsorted)
- ✅ Generate @SQ header lines for each contig
- ✅ Generate @PG header line with program info
- ✅ Format alignment records with correct FLAG values
- ✅ Include optional tags: AS:i, XS:i, NM:i

#### F-006: Multi-Threading Support ✅
- ✅ Configurable thread count via CLI (-t option)
- ✅ Near-linear speedup on multi-core systems
- ✅ Thread-safe data structures using rayon

#### F-007: Memory Protection ✅
- ✅ max_occ: Skip seeds with SA interval > threshold (default: 500)
- ✅ max_chains_per_contig: Limit chains per contig (default: 5)
- ✅ max_alignments_per_read: Limit output alignments (default: 5)

#### F-008: MAPQ Estimation ✅
- ✅ Score-difference based MAPQ model
- ✅ Consider best vs second-best alignment score difference
- ✅ Output MAPQ in SAM record (column 5)

#### F-009: CLI Interface ✅
- ✅ `index` subcommand: Build FM-index from FASTA
- ✅ `align` subcommand: Align FASTQ reads to existing index
- ✅ `mem` subcommand: One-step index + align (BWA-MEM style)
- ✅ Support -o output option for file output
- ✅ Support -t thread count option

#### F-010: Input Validation ✅
- ✅ Reject empty FASTA sequences
- ✅ Reject duplicate contig names
- ✅ Handle malformed FASTQ records
- ✅ Validate thread count (must be > 0)
- ✅ Handle various line endings (LF/CRLF)

### 计划中的功能

#### F-011: Paired-End Alignment (Planned)
- ⏳ Support paired-end read alignment (v0.3.0)
- ⏳ Insert size constraints
- ⏳ Mate rescue for unpaired reads
- ⏳ Proper pair flag marking
- **Note**: Infrastructure exists in code but not exposed via CLI

#### F-012: BAM Output (Planned)
- ⏳ Support compressed BAM format output (v0.5.0)
- ⏳ Coordinate-sorted output option

#### F-013: BWA Native Index Compatibility (Future)
- ⏳ Read BWA's .bwt/.sa/.pac index files directly

## 非功能性需求

### NFR-001: Memory Safety ✅
- ✅ Zero `unsafe` code in the entire codebase
- ✅ All memory safety guaranteed by Rust compiler
- ✅ Enforced via `unsafe_code = "forbid"` lint

### NFR-002: Performance Targets

| Metric | Target | v0.2.0 Status |
|--------|--------|---------------|
| Index build (100M bp) | < 60s | ✅ Typical: ~40s |
| Alignment (1K reads, 4 threads) | < 1s | ✅ Typical: ~0.6s |
| Memory usage (human genome) | < 8 GB | ⚠️ Not yet tested on hg38 |
| Multi-thread scaling | Near-linear up to 8 threads | ✅ Achieved |

### NFR-003: Code Quality ✅
- ✅ Pass `cargo fmt --all -- --check`
- ✅ Pass `cargo clippy --all-targets --all-features -- -D warnings`
- ✅ Pass `cargo test --all-targets --all-features`
- ✅ MSRV: Rust 1.70
- ✅ 201 tests (188 unit + 11 integration + 2 other)

### NFR-004: Platform Support ✅
- ✅ Linux (primary, CI-tested)
- ✅ macOS (builds successfully)
- ✅ Windows (builds successfully)

---

## 未来规划

以下版本按优先级排序，具体实现时间待定：

### v0.3.0 — 配对端比对

| 功能 | 描述 |
|------|------|
| PE Reads | 配对端 FASTQ 输入（基础架构已存在） |
| Insert Size | 插入片段长度估计 |
| Mate Rescue | 配对挽救（单端未比对时） |
| Proper Pair | 正确配对 FLAG 标记 |

### v0.4.0 — 索引兼容

| 功能 | 描述 |
|------|------|
| BWA Index | 读取 BWA 原生索引文件 |
| 格式支持 | `.bwt/.sa/.pac/.ann/.amb` |

### v0.5.0 — 输出增强

| 功能 | 描述 |
|------|------|
| BAM 输出 | 直接输出压缩格式 |
| 排序输出 | 按 coordinate 排序 |

### v0.6.0 — 性能飞跃

| 功能 | 描述 |
|------|------|
| SIMD 加速 | Smith-Waterman 向量化 |
| 内存映射 | 大索引文件 mmap 支持 |

### v1.0.0 — 生产就绪

| 条件 | 描述 |
|------|------|
| 正确性验证 | 人类基因组（hg38）级别测试通过 |
| 性能对标 | 与 C 版 BWA mapping rate 相近 |
| API 稳定 | 公共 API 冻结，承诺向后兼容 |
| 文档完善 | 生产级文档和错误处理 |

---

## 版本策略

### 语义化版本

本项目遵循 [SemVer](https://semver.org/)：**`MAJOR.MINOR.PATCH`**

| 组件 | 变更类型 | 示例 |
|------|----------|------|
| **MAJOR** | 不兼容变更（索引格式、API 破坏） | `0.x` → `1.0` |
| **MINOR** | 新功能，向后兼容 | PE 比对、BAM 输出 |
| **PATCH** | Bug 修复、性能微调、文档 | `0.1.0` → `0.1.1` |

### 0.x 阶段规则

- API 和索引格式**允许不兼容变更**
- 破坏性变更在 CHANGELOG 中标注 `BREAKING`
- `.fm` 索引文件的 `version` 字段用于兼容性检查

### 进入 1.0.0 的条件

1. ✅ 人类基因组（hg38）级别正确性验证通过
2. ✅ 与 C 版 BWA 的 mapping rate 在合理范围
3. ✅ 公共 API 稳定，无频繁破坏性变更
4. ✅ 文档和错误处理达到生产级质量

---

## 索引格式版本

FM 索引文件（`.fm`）有独立的内部版本号：

```
┌──────────────────────────────────────┐
│ .fm 文件结构                          │
├──────────────────────────────────────┤
│ magic: 0x424D4146_4D5F5253 ("BWAFM_RS") │
│ version: u32 (当前 = 2)               │
│ ... 其他字段 ...                      │
└──────────────────────────────────────┘
```

**版本策略**：

- 软件升级时若索引格式未变 → 用户无需重建索引
- 索引格式不兼容变更 → 递增 `version`，加载时报错提示

---

## 发布流程

### 发布前检查

```bash
# 1. 代码质量
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings

# 2. 测试
cargo test --all-targets --all-features

# 3. 构建
cargo build --release
```

### 发布步骤

```bash
# 1. 更新版本号
#    - Cargo.toml
#    - CHANGELOG.md（添加发布日期）

# 2. 提交并打标签
git commit -am "chore: release v0.x.x"
git tag v0.x.x

# 3. 推送
git push origin main --tags
```

### CHANGELOG 更新

每次发布必须更新 [CHANGELOG.md](CHANGELOG.md)：

- 添加发布日期
- 按类型分类变更（Added/Fixed/Changed/Deprecated/Removed）
- 标注破坏性变更（BREAKING）

---

## 相关文档

| 文档 | 说明 |
|------|------|
| [CHANGELOG.md](CHANGELOG.md) | 版本变更日志 |
| [CONTRIBUTING.md](CONTRIBUTING.md) | 贡献指南 |
| [docs/architecture.md](docs/architecture.md) | 架构设计 |
| [docs/plan.md](docs/plan.md) | 详细功能规划 |
