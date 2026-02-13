# bwa-rust

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
├── main.rs          # CLI 入口（clap）
├── lib.rs           # Library 入口
├── io/
│   ├── fasta.rs     # FASTA 解析器
│   └── fastq.rs     # FASTQ 解析器
├── index/
│   ├── sa.rs        # 后缀数组构建（倍增法）
│   ├── bwt.rs       # BWT 构建
│   └── fm.rs        # FM 索引（C/Occ/SA/backward_search/稀疏SA）
├── align/
│   ├── mod.rs       # 对齐主流程（并行、SAM输出、MAPQ）
│   ├── seed.rs      # SMEM 种子查找
│   ├── chain.rs     # 种子链构建与过滤
│   └── sw.rs        # 带状仿射间隙 Smith-Waterman
└── util/
    └── dna.rs       # DNA 编码/解码/反向互补
```

## 文档

- [架构文档](docs/architecture.md) — 模块划分、索引格式、算法流程
- [教程](docs/tutorial.md) — 从 0 实现 BWA 风格对齐器
- [开发路线图](../ROADMAP.md) — 项目规划与 TODO

## 基准测试

```bash
cargo bench
```

## 示例

```bash
cargo run --example simple_align
```

## 规划

详见 [ROADMAP.md](../ROADMAP.md)。
