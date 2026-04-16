# bwa-rust 快速入门

> 使用 bwa-rust 作为命令行工具和库的入门指南。

---

## 目录

- [安装](#安装)
- [快速开始](#快速开始)
- [CLI 命令](#cli-命令)
- [库使用](#库使用)
- [配置说明](#配置说明)
- [性能调优](#性能调优)
- [常见问题](#常见问题)

---

## 安装

### 从源码编译

```bash
# 克隆仓库
git clone https://github.com/LessUp/bwa-rust.git
cd bwa-rust

# 编译 release 版本
cargo build --release

# 二进制文件位置
target/release/bwa-rust
```

### 系统要求

- Rust 1.70 或更高版本
- Linux、macOS 或 Windows

### 验证安装

```bash
bwa-rust --version
```

---

## 快速开始

### 1. 构建 FM 索引

```bash
# 从 FASTA 参考序列
bwa-rust index reference.fa -o ref

# 生成：ref.fm
```

### 2. 比对 Reads（两步式）

```bash
# 步骤 1：构建索引（如不存在）
bwa-rust index reference.fa -o ref

# 步骤 2：比对
bwa-rust align -i ref.fm reads.fq -o output.sam
```

### 3. 一步比对（BWA-MEM 风格）

```bash
# 一步完成索引构建和比对
bwa-rust mem reference.fa reads.fq -o output.sam

# 多线程
bwa-rust mem reference.fa reads.fq -t 4 -o output.sam
```

### 4. 使用测试数据

```bash
# 使用提供的测试数据
cd data/

# 构建索引
bwa-rust index toy.fa -o toy

# 比对
bwa-rust align -i toy.fm toy_reads.fq

# 或一步完成
bwa-rust mem toy.fa toy_reads.fq
```

---

## CLI 命令

### `index` — 构建 FM 索引

```bash
bwa-rust index <reference.fa> -o <prefix>
```

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `reference` | 必填 | FASTA 参考文件 |
| `-o, --output` | `ref` | 输出前缀，生成 `.fm` 索引 |

**示例：**
```bash
bwa-rust index hg38.fa -o hg38   # 生成 hg38.fm
```

### `align` — 使用已有索引比对

```bash
bwa-rust align -i <index.fm> <reads.fq> [options]
```

| 选项 | 默认值 | 说明 |
|------|--------|------|
| `-i, --index` | 必填 | `.fm` 索引路径 |
| `reads` | 必填 | FASTQ 文件 |
| `-o, --out` | stdout | 输出 SAM 文件 |
| `-t, --threads` | 1 | 线程数 |
| `--match` | 2 | 匹配得分 |
| `--mismatch` | 1 | 错配罚分 |
| `--gap-open` | 2 | Gap 开启罚分 |
| `--gap-ext` | 1 | Gap 扩展罚分 |
| `--clip-penalty` | 1 | 软剪切惩罚（候选排序用）|
| `--band-width` | 16 | SW 带宽 |
| `--score-threshold` | 20 | 最低输出得分 |
| `--max-occ` | 500 | 跳过出现次数 >500 的种子 |
| `--max-chains` | 5 | 每 contig 最大链数 |
| `--max-alignments` | 5 | 每 read 最大输出数 |

**示例：**
```bash
# 基本用法
bwa-rust align -i ref.fm reads.fq

# 多线程
bwa-rust align -i ref.fm reads.fq -t 8 -o out.sam

# 自定义打分
bwa-rust align -i ref.fm reads.fq \
    --match 2 --mismatch 1 --gap-open 2 --gap-ext 1
```

### `mem` — 一步比对

```bash
bwa-rust mem <reference.fa> <reads.fq> [options]
```

使用 BWA-MEM 默认打分：match=1, mismatch=4, gap-open=6, gap-ext=1。

| 选项 | 默认值 | 说明 |
|------|--------|------|
| `-o, --out` | stdout | 输出 SAM 文件 |
| `-t, --threads` | 1 | 线程数 |
| `-A, --match` | 1 | 匹配得分 |
| `-B, --mismatch` | 4 | 错配罚分 |
| `-O, --gap-open` | 6 | Gap 开启罚分 |
| `-E, --gap-ext` | 1 | Gap 扩展罚分 |
| `-w, --band-width` | 100 | SW 带宽 |
| `-T, --score-threshold` | 10 | 最低得分 |

**示例：**
```bash
# 默认参数一步完成
bwa-rust mem ref.fa reads.fq -o out.sam

# 多线程
bwa-rust mem ref.fa reads.fq -t 4 -o out.sam

# 自定义参数
bwa-rust mem ref.fa reads.fq \
    -A 1 -B 4 -O 6 -E 1 \
    -w 100 -T 10 \
    -t 8 \
    -o out.sam
```

---

## 库使用

### 基础示例

```rust
use bwa_rust::index::{sa, bwt, fm};
use bwa_rust::util::dna;

fn main() {
    // 1. 加载或构建参考序列
    let reference = b"ACGTACGTACGT";
    
    // 2. 归一化和编码
    let norm = dna::normalize_seq(reference);
    let mut text: Vec<u8> = norm.iter().map(|&b| dna::to_alphabet(b)).collect();
    text.push(0);  // 终止符
    
    // 3. 构建索引组件
    let sa_arr = sa::build_sa(&text);
    let bwt_arr = bwt::build_bwt(&text, &sa_arr);
    
    // 4. 构建 FM 索引
    let contigs = vec![fm::Contig {
        name: "chr1".into(),
        len: 12,
        offset: 0,
    }];
    
    let fm_idx = fm::FMIndex::build(
        text,
        bwt_arr,
        sa_arr,
        contigs,
        6,  // sigma
        4,  // SA 采样间隔
    );
    
    // 5. 搜索模式
    let pattern: Vec<u8> = b"CGT".iter().map(|&b| dna::to_alphabet(b)).collect();
    
    if let Some((l, r)) = fm_idx.backward_search(&pattern) {
        println!("找到 {} 次出现", r - l);
        
        // 获取位置
        for pos in fm_idx.sa_interval_positions(l, r) {
            println!("  位于位置 {}", pos);
        }
    }
}
```

### 运行示例

```bash
cargo run --example simple_align
```

---

## 配置说明

### 内存防护限制

bwa-rust 提供三个可配置的限制，防止重复序列导致内存爆炸：

```bash
# 限制重复种子（默认：500）
bwa-rust align -i ref.fm reads.fq --max-occ 200

# 限制每 contig 链数（默认：5）
bwa-rust mem ref.fa reads.fq --max-chains 3

# 限制每 read 输出数（默认：5）
bwa-rust mem ref.fa reads.fq --max-alignments 10
```

### 带宽调整

```bash
# 小带宽：快但对 indel 敏感度低
bwa-rust align -i ref.fm reads.fq --band-width 16

# 大带宽：慢但对 indel 容忍度高
bwa-rust align -i ref.fm reads.fq --band-width 64
```

---

## 性能调优

### 多线程

```bash
# 使用所有可用核心
bwa-rust mem ref.fa reads.fq -t $(nproc)

# 或手动指定
bwa-rust mem ref.fa reads.fq -t 8
```

### 内存优化

| 设置 | 影响 | 建议 |
|------|------|------|
| `--max-occ 500` | 跳过重复种子 | 大多数情况下保持 500 |
| `--max-chains 5` | 限制每 contig 链数 | 追求速度时可设为 3 |
| `--max-alignments 5` | 限制输出 | 需要更多比对时可设为 10 |

---

## 常见问题

### 索引未找到

```
Error: Index file not found: ref.fm
```

**解决：** 先构建索引
```bash
bwa-rust index reference.fa -o ref
```

### 线程数错误

```
Error: --threads must be >= 1
```

**解决：** 使用 `--threads 1` 或更高，单线程可省略该参数。

### 比对率过低

**可能原因：**
1. `--score-threshold` 设置太高
2. `--max-occ` 设置太低（跳过了有效种子）
3. 带宽太小，无法容忍 indel

**解决方案：**
```bash
# 降低阈值
bwa-rust align -i ref.fm reads.fq --score-threshold 10

# 增加 max_occ
bwa-rust align -i ref.fm reads.fq --max-occ 1000

# 增加带宽
bwa-rust align -i ref.fm reads.fq --band-width 32
```

---

## 下一步

- [算法教程](./algorithms.zh-CN.md) — 深入了解 FM 索引和比对算法
- [架构详解](../architecture/) — 模块设计与实现
- [API 文档](../api/) — 库 API 参考
