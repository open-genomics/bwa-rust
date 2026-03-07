# bwa-rust

[![CI](https://github.com/LessUp/bwa-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/LessUp/bwa-rust/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

受 [BWA](https://github.com/lh3/bwa) 启发的 Rust 版序列比对器。本项目在整体结构和算法思想上接近 BWA/BWA-MEM，但**不追求与 C 版 BWA 100% 行为兼容**（命令行选项、索引格式、MAPQ 细节等允许不同）。

## 已实现功能

### 索引构建（`index` 子命令）
- 读取 FASTA 参考序列（支持多 contig、不同换行符、非标准字符过滤）
- 基于倍增法构建后缀数组（SA）
- 从 SA 构建 BWT
- 构建 FM 索引（含 C 表、分块 Occ 采样、稀疏 SA 采样支持）
- 序列化索引到 `.fm` 文件（含 magic number、版本号、构建元数据）

### 序列比对（`align` 子命令）
- 加载 `.fm` 索引
- 读取 FASTQ reads
- SMEM 种子查找（超级最大精确匹配）+ 多链构建与过滤
- 带状仿射间隙 Smith-Waterman 局部对齐（支持错配、插入、缺失）
- 正向 / 反向互补双向比对
- 多链候选去重、主/次要比对输出（primary / secondary FLAG）
- 改进的 MAPQ 估算（基于主次候选得分差）
- 输出 SAM 格式（含 @HD/@SQ/@PG header，CIGAR、MAPQ、AS/XS/NM 标签）
- 未比对 reads 标记为 unmapped（FLAG=4）
- **多线程并行**：通过 `--threads` 参数使用 rayon 并行处理

### 支持/不支持
- **支持**：单端 reads 对齐
- **不支持**：配对端（PE）对齐（未来可扩展）

## 快速开始

```bash
# 构建
cargo build --release

# 构建索引
cargo run --release -- index data/toy.fa -o data/toy

# 对齐 reads
cargo run --release -- align -i data/toy.fm data/toy_reads.fq

# 输出到文件
cargo run --release -- align -i data/toy.fm data/toy_reads.fq -o output.sam

# 多线程对齐
cargo run --release -- align -i data/toy.fm data/toy_reads.fq -t 4

# 自定义比对参数
cargo run --release -- align -i data/toy.fm data/toy_reads.fq \
    --match 2 --mismatch 1 --gap-open 2 --gap-ext 1 --band-width 16
```

## 索引构建与索引格式简介

`index` 子命令接受一个 FASTA 文件，执行以下步骤：

1. **读取参考序列**：逐条解析 FASTA 记录，将碱基归一化为 `{A,C,G,T,N}`。
2. **编码为数值字母表**：`{0:$, 1:A, 2:C, 3:G, 4:T, 5:N}`，contig 之间用 `$`（0）分隔。
3. **构建后缀数组**：使用倍增法（O(n log²n)）排序所有后缀。
4. **构建 BWT**：从 SA 直接推导 Burrows-Wheeler 变换。
5. **构建 FM 索引**：计算 C 表和分块 Occ 采样表，保存 SA 用于位置查询（支持稀疏采样）。
6. **序列化**：使用 bincode 将整个 `FMIndex` 结构写入 `.fm` 文件。

索引文件包含 magic number（`BWAFM_RS`）和版本号（v2），用于格式兼容性检查。可选的构建元数据记录参考文件名、命令参数和时间戳。

## 项目结构

```
src/
├── main.rs              # CLI 入口（clap）
├── lib.rs               # Library 入口
├── error.rs             # 自定义错误类型（BwaError / BwaResult）
├── io/
│   ├── mod.rs           # IO 模块声明
│   ├── fasta.rs         # FASTA 解析器
│   ├── fastq.rs         # FASTQ 解析器
│   └── sam.rs           # SAM 格式输出（header / record / unmapped）
├── index/
│   ├── mod.rs           # 索引模块声明
│   ├── sa.rs            # 后缀数组构建（倍增法）
│   ├── bwt.rs           # BWT 构建
│   ├── fm.rs            # FM 索引（C/Occ/SA/backward_search/稀疏SA）
│   └── builder.rs       # 从 FASTA 一键构建 FM 索引
├── align/
│   ├── mod.rs           # 对齐模块声明与 AlignOpt 配置
│   ├── seed.rs          # SMEM 种子查找
│   ├── chain.rs         # 种子链构建与过滤
│   ├── sw.rs            # 带状仿射间隙 Smith-Waterman
│   ├── extend.rs        # 链→完整对齐（左右扩展 + CIGAR 合并）
│   ├── candidate.rs     # 对齐候选收集与去重
│   ├── mapq.rs          # MAPQ 估算（BWA 风格得分差模型）
│   └── pipeline.rs      # 对齐 pipeline（批量并行 + SAM 输出）
└── util/
    ├── mod.rs           # 工具模块声明
    └── dna.rs           # DNA 编码/解码/反向互补
```

## 文档

- [架构文档](docs/architecture.md) — 模块划分、索引格式、算法流程
- [教程](docs/tutorial.md) — 从 0 实现 BWA 风格对齐器
- [全量复刻计划](docs/plan.md) — BWA 全量复刻远景规划（供未来扩展参考）
- [开发路线图](../ROADMAP.md) — 项目规划（v0.1.0 已完成）
- [变更日志](CHANGELOG.md) — 版本变更记录

## 测试用例报告

全部 **133** 个测试通过（121 单元测试 + 11 集成测试 + 1 文档测试），0 失败。

```bash
cargo test
# test result: ok. 133 passed; 0 failed; 0 ignored
```

### 单元测试（118 个）

| 模块 | 测试数 | 覆盖范围 |
|------|--------|----------|
| `util::dna` | 6 | 碱基归一化、编码/解码往返、互补/反向互补、未知字符映射 |
| `io::fasta` | 3 | FASTA 解析（基础、CRLF/空白、前导空行） |
| `io::fastq` | 10 | FASTQ 解析（基础、CRLF、空输入、错误头、缺少+、长度不匹配、截断、多reads、描述、小写） |
| `io::sam` | 8 | SAM header 格式、unmapped 记录、mapped 记录、空 contig、字段数验证、反向互补 FLAG、次要比对、字符串 contig |
| `index::sa` | 3 | 后缀数组基础构建、与朴素算法对比、多分隔符处理 |
| `index::bwt` | 2 | 单 contig BWT、多 contig+分隔符 BWT |
| `index::fm` | 15 | FM 索引构建、backward search（命中/未命中/单字符/全文）、序列化/反序列化、位置映射、Occ 正确性/边界、SA 区间、稀疏 SA、rank 单调性、元数据 |
| `index::builder` | 7 | 从 FASTA 构建索引、空输入、单序列、多 contig 偏移、序列内容保存、搜索验证、小写处理 |
| `align::seed` | 10 | SMEM 种子基础查找、最小长度过滤、长匹配、空查询、零长度、超长度、无匹配、坐标验证、去重、MEM 兼容接口 |
| `align::chain` | 11 | 链构建（简单对角线、避免重叠/大间距、多链、空种子、单种子、三共线种子、不同 contig、排序）、链过滤（弱链移除、空、非重叠保留）、大间距处理 |
| `align::sw` | 18 | 带状 SW（完美匹配、单错配、插入、缺失、空输入、全错配、长序列、部分匹配）、缓冲区复用、CIGAR 生成/解析/往返、右扩展/左扩展（正常/空输入） |
| `align::pipeline` | 13 | MAPQ 模型（单调递减、等分零值）、候选收集（精确/错配/空查询）、候选去重（重复/唯一/空）、单 read 比对（unmapped/空序列/mapped/反向互补）、chain_to_alignment 系列 |
| `align::mod` | 7 | chain_to_alignment（空链、单种子、双相邻种子、种子间间隙、右侧裁剪、结果字段）、push_run 合并 |

### 集成测试（11 个）

| 测试用例 | 验证内容 |
|----------|----------|
| `e2e_build_index_and_exact_search` | FASTA → FM 索引构建 → 多 contig 精确搜索 |
| `e2e_seed_chain_align_exact` | SMEM 种子 → 链构建/过滤 → SW 对齐（完美匹配） |
| `e2e_seed_chain_align_with_mismatch` | 含错配 read 的种子-链-对齐全流程 |
| `e2e_revcomp_alignment` | 反向互补 read 的 SMEM 种子查找 |
| `e2e_sam_output_format` | SAM header + unmapped/mapped 记录格式完整性 |
| `e2e_parse_fasta_and_fastq` | FASTA + FASTQ 联合解析一致性 |
| `e2e_fm_index_serialize_deserialize_search` | FM 索引序列化/反序列化后搜索结果一致 |
| `e2e_dna_encode_decode_roundtrip` | DNA 编码/解码往返（多种序列模式） |
| `e2e_dna_revcomp_preserves_length` | 反向互补保持长度 + 双重反向互补恢复 |
| `e2e_multi_contig_search` | 多 contig 精确搜索 → 位置映射到正确 contig |
| `e2e_sa_produces_sorted_suffixes` | 后缀数组字典序排列正确性验证 |

### 文档测试（1 个）

| 测试用例 | 验证内容 |
|----------|----------|
| `lib.rs` 示例代码 | 库级文档示例编译通过 |

## 基准测试

```bash
cargo bench
```

## 示例

```bash
cargo run --example simple_align
```

## 安装

### 从源码构建

```bash
git clone https://github.com/LessUp/bwa-rust.git
cd bwa-rust/bwa-rust
cargo build --release
```

编译后的二进制文件位于 `target/release/bwa-rust`。

### 系统要求

- **Rust** 1.70 或更高版本
- 支持 Linux、macOS、Windows

## 规划

v0.1.0 路线图已全部完成，详见 [ROADMAP.md](../ROADMAP.md)。未来展望（配对端、BAM 输出等）见路线图末尾。

## 贡献

欢迎贡献！请阅读 [CONTRIBUTING.md](CONTRIBUTING.md) 了解详情。

## 许可证

本项目采用 [MIT 许可证](LICENSE) 发布。
