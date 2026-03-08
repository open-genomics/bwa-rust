# 快速开始

## 系统要求

- **Rust** 1.70 或更高版本
- 支持 Linux、macOS、Windows

## 安装

### 从源码构建

```bash
git clone https://github.com/LessUp/bwa-rust.git
cd bwa-rust
cargo build --release
```

编译后的二进制文件位于 `target/release/bwa-rust`。

## 基本用法

### 构建索引

```bash
cargo run --release -- index data/toy.fa -o data/toy
```

### 对齐 reads

```bash
# 基本对齐
cargo run --release -- align -i data/toy.fm data/toy_reads.fq

# 输出到文件
cargo run --release -- align -i data/toy.fm data/toy_reads.fq -o output.sam

# 多线程对齐
cargo run --release -- align -i data/toy.fm data/toy_reads.fq -t 4

# 自定义比对参数
cargo run --release -- align -i data/toy.fm data/toy_reads.fq \
    --match 2 --mismatch 1 --gap-open 2 --gap-ext 1 --band-width 16
```

## 索引格式

`index` 子命令接受一个 FASTA 文件，执行以下步骤：

1. **读取参考序列**：逐条解析 FASTA 记录，将碱基归一化为 `{A,C,G,T,N}`
2. **编码为数值字母表**：`{0:$, 1:A, 2:C, 3:G, 4:T, 5:N}`，contig 之间用 `$` 分隔
3. **构建后缀数组**：使用倍增法（O(n log²n)）排序所有后缀
4. **构建 BWT**：从 SA 直接推导 Burrows-Wheeler 变换
5. **构建 FM 索引**：计算 C 表和分块 Occ 采样表
6. **序列化**：使用 bincode 写入 `.fm` 文件

索引文件包含 magic number（`BWAFM_RS`）和版本号（v2），用于格式兼容性检查。

## 已实现功能

### 支持

- 单端 reads 对齐
- SMEM 种子查找 + 种子链构建
- 带状仿射间隙 Smith-Waterman 局部对齐
- SAM 格式输出（含 CIGAR、MAPQ、AS/XS/NM 标签）
- 多线程并行处理

### 暂不支持

- 配对端（PE）对齐（计划在 v0.2.0 实现）
- BAM 输出格式
- BWA 原生索引文件读取

## 作为库使用

```bash
cargo run --example simple_align
```

bwa-rust 同时提供 library API，可在 Rust 项目中直接调用索引构建和比对功能。详见 [算法教程](./tutorial.md)。
