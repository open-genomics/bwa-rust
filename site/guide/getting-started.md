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
bwa-rust index data/toy.fa -o data/toy
# 输出：data/toy.fm
```

### 对齐 reads

```bash
# 基本对齐
bwa-rust align -i data/toy.fm data/toy_reads.fq

# 输出到文件
bwa-rust align -i data/toy.fm data/toy_reads.fq -o output.sam

# 多线程对齐
bwa-rust align -i data/toy.fm data/toy_reads.fq -t 4

# 自定义比对参数
bwa-rust align -i data/toy.fm data/toy_reads.fq \
    --match 2 --mismatch 1 --gap-open 2 --gap-ext 1 --band-width 16
```

### 一步比对（BWA-MEM 风格）

```bash
# 构建索引并比对
bwa-rust mem data/toy.fa data/toy_reads.fq -t 4 -o output.sam

# 使用 BWA-MEM 默认打分
bwa-rust mem data/toy.fa data/toy_reads.fq \
    -A 1 -B 4 -O 6 -E 1 -w 100
```

## CLI 参数详解

### `index` — 构建索引

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `<reference.fa>` | 必填 | FASTA 参考文件 |
| `-o, --output` | `ref` | 输出前缀，生成 `.fm` 索引 |

### `align` — 使用索引比对

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `-i, --index` | 必填 | `.fm` 索引路径 |
| `<reads.fq>` | 必填 | FASTQ reads 文件 |
| `-o, --out` | stdout | 输出 SAM 路径 |
| `-t, --threads` | 1 | 线程数 |
| `--match` | 2 | 匹配得分 |
| `--mismatch` | 1 | 错配罚分 |
| `--gap-open` | 2 | Gap 开启罚分 |
| `--gap-ext` | 1 | Gap 扩展罚分 |
| `--band-width` | 16 | 带状 SW 带宽 |
| `--score-threshold` | 20 | 最低输出得分 |

### `mem` — 一步比对

使用 BWA-MEM 默认打分：match=1, mismatch=4, gap-open=6, gap-ext=1。

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `-o, --out` | stdout | 输出 SAM 路径 |
| `-t, --threads` | 1 | 线程数 |
| `-A, --match` | 1 | 匹配得分 |
| `-B, --mismatch` | 4 | 错配罚分 |
| `-O, --gap-open` | 6 | Gap 开启罚分 |
| `-E, --gap-ext` | 1 | Gap 扩展罚分 |
| `-w, --band-width` | 100 | 带宽 |
| `-T, --score-threshold` | 10 | 最低得分 |

## 索引格式

`index` 子命令接受一个 FASTA 文件，执行以下步骤：

1. **读取参考序列**：逐条解析 FASTA 记录，将碱基归一化为 `{A,C,G,T,N}`
2. **编码为数值字母表**：`{0:$, 1:A, 2:C, 3:G, 4:T, 5:N}`，contig 之间用 `$` 分隔
3. **构建后缀数组**：使用倍增法（O(n log²n)）排序所有后缀
4. **构建 BWT**：从 SA 直接推导 Burrows-Wheeler 变换
5. **构建 FM 索引**：计算 C 表和分块 Occ 采样表
6. **序列化**：使用 bincode 写入 `.fm` 文件

索引文件包含 magic number（`BWAFM_RS`）和版本号（v2），用于格式兼容性检查。

## 功能支持

### 已实现

| 功能 | 说明 |
|------|------|
| 单端 reads 对齐 | 正向 + 反向互补双向比对 |
| SMEM 种子查找 | 超级最大精确匹配，`max_occ` 过滤 |
| 种子链构建 | DP + 贪心剥离，`max_chains` 限制 |
| 带状 SW 对齐 | 仿射间隙，semi-global refinement |
| SAM 输出 | 完整 header、CIGAR、MAPQ、AS/XS/NM |
| 多线程并行 | rayon 数据并行 |

### 计划中

| 功能 | 计划版本 |
|------|----------|
| 配对端（PE）对齐 | v0.2.0 |
| BWA 原生索引读取 | v0.3.0 |
| BAM 输出格式 | v0.4.0 |
| SIMD 加速 | v0.5.0 |

## 作为库使用

```bash
cargo run --example simple_align
```

bwa-rust 同时提供 library API，可在 Rust 项目中直接调用索引构建和比对功能。详见 [算法教程](./tutorial.md)。

## 常见问题

### 为什么比对结果与 BWA 不同？

bwa-rust **不追求 100% 行为兼容**：

- 索引格式不同（单一 `.fm` vs 多文件）
- MAPQ 计算方式简化
- 部分边界情况处理可能不同

### 如何处理内存不足？

使用内存防护参数：

```bash
# 减少重复种子
bwa-rust align -i ref.fm reads.fq --max-occ 100

# 减少候选链数
bwa-rust mem ref.fa reads.fq --max-chains 3

# 减少输出数量
bwa-rust mem ref.fa reads.fq --max-alignments 3
```

### 如何提升性能？

1. 增加线程数：`-t 8`
2. 调整带宽：小带宽更快但对 indel 敏感度低
3. 提高阈值：减少次要比对输出
