# bwa-rust

受 [BWA](https://github.com/lh3/bwa) 启发的 Rust 版序列比对器。本项目在整体结构和算法思想上接近 BWA/BWA-MEM，但**不追求与 C 版 BWA 100% 行为兼容**（命令行选项、索引格式、MAPQ 细节等允许不同）。

## 已实现功能

### 索引构建（`index` 子命令）
- 读取 FASTA 参考序列（支持多 contig）
- 基于倍增法构建后缀数组（SA）
- 从 SA 构建 BWT
- 构建 FM 索引（含 C 表、分块 Occ 采样）
- 序列化索引到 `.fm` 文件

### 序列比对（`align` 子命令）
- 加载 `.fm` 索引
- 读取 FASTQ reads
- MEM 种子查找 + 种子链构建
- 带状仿射间隙 Smith-Waterman 局部对齐（支持错配、插入、缺失）
- 正向 / 反向互补双向比对
- 输出 SAM 格式（含 CIGAR、MAPQ、AS/XS/NM 标签）
- 未比对 reads 标记为 unmapped（FLAG=4）

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
5. **构建 FM 索引**：计算 C 表和分块 Occ 采样表，保存完整 SA 用于位置查询。
6. **序列化**：使用 bincode 将整个 `FMIndex` 结构写入 `.fm` 文件。

索引文件包含 magic number 和版本号，用于格式兼容性检查。

## 项目结构

```
src/
├── main.rs          # CLI 入口（clap）
├── io/
│   ├── fasta.rs     # FASTA 解析器
│   └── fastq.rs     # FASTQ 解析器
├── index/
│   ├── sa.rs        # 后缀数组构建（倍增法）
│   ├── bwt.rs       # BWT 构建
│   └── fm.rs        # FM 索引（C/Occ/SA/backward_search）
├── align/
│   └── mod.rs       # 对齐流程（MEM种子、链构建、带状SW、SAM输出）
└── util/
    └── dna.rs       # DNA 编码/解码/反向互补
```

## 规划

详见 [ROADMAP.md](../ROADMAP.md)。
