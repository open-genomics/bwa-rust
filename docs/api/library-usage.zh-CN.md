# 库使用指南

> 将 bwa-rust 作为 Rust 库使用。

---

## 目录

- [概述](#概述)
- [基础示例](#基础示例)
- [FM 索引操作](#fm-索引操作)
- [比对流水线](#比对流水线)
- [API 参考](#api-参考)

---

## 概述

bwa-rust 可以作为库在您的 Rust 项目中使用：

```toml
[dependencies]
bwa-rust = { path = "path/to/bwa-rust" }
```

---

## 基础示例

```rust
use bwa_rust::index::{sa, bwt, fm};
use bwa_rust::util::dna;

fn main() {
    // 参考序列
    let reference = b"ACGTACGTACGT";

    // 归一化和编码
    let norm = dna::normalize_seq(reference);
    let mut text: Vec<u8> = norm.iter().map(|&b| dna::to_alphabet(b)).collect();
    text.push(0);  // 终止符

    // 构建后缀数组
    let sa_arr = sa::build_sa(&text);

    // 构建 BWT
    let bwt_arr = bwt::build_bwt(&text, &sa_arr);

    // 构建 FM 索引
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
        6,  // 字母表大小
        4,  // SA 采样间隔
    );

    // 搜索模式
    let pattern: Vec<u8> = b"CGT".iter()
        .map(|&b| dna::to_alphabet(b))
        .collect();

    if let Some((l, r)) = fm_idx.backward_search(&pattern) {
        println!("找到 {} 次出现", r - l);
    }
}
```

---

## FM 索引操作

### 从 FASTA 构建

```rust
use bwa_rust::index::builder;
use std::fs::File;

fn build_index(fasta_path: &str, output: &str) {
    let mut file = File::open(fasta_path).unwrap();
    let fm_idx = builder::build_from_fasta(&mut file, 6, 4).unwrap();

    // 序列化到文件
    fm_idx.save(output).unwrap();
}
```

### 加载和搜索

```rust
use bwa_rust::index::fm::FMIndex;
use bwa_rust::util::dna;

fn search_pattern(index_path: &str, pattern: &[u8]) {
    // 加载索引
    let fm = FMIndex::load(index_path).unwrap();

    // 编码模式
    let encoded: Vec<u8> = dna::normalize_seq(pattern)
        .iter()
        .map(|&b| dna::to_alphabet(b))
        .collect();

    // 搜索
    if let Some((l, r)) = fm.backward_search(&encoded) {
        println!("模式出现 {} 次", r - l);

        // 获取位置
        for pos in fm.sa_interval_positions(l, r) {
            println!("  位于位置 {}", pos);
        }
    }
}
```

---

## 比对流水线

```rust
use bwa_rust::align::{AlignOpt, pipeline};
use bwa_rust::index::fm::FMIndex;
use bwa_rust::io::fastq;

fn align_reads(fm_path: &str, fastq_path: &str) {
    // 加载索引
    let fm = FMIndex::load(fm_path).unwrap();

    // 配置比对参数
    let opt = AlignOpt {
        match_score: 2,
        mismatch_penalty: 1,
        gap_open: 2,
        gap_extend: 1,
        clip_penalty: 1,
        band_width: 16,
        score_threshold: 20,
        min_seed_len: 19,
        threads: 4,
        max_occ: 500,
        max_chains_per_contig: 5,
        max_alignments_per_read: 5,
    };

    // 解析 FASTQ
    let reads = fastq::parse_file(fastq_path).unwrap();

    // 比对
    let results = pipeline::align_reads(&fm, &reads, &opt);

    // 输出 SAM
    for record in results {
        println!("{}", record);
    }
}
```

---

## API 参考

### `index` 模块

| 结构体/函数 | 说明 |
|------------|------|
| `FMIndex` | FM 索引数据结构 |
| `build_sa()` | 构建后缀数组 |
| `build_bwt()` | 从 SA 构建 BWT |
| `FMIndex::build()` | 构建 FM 索引 |
| `FMIndex::load()` | 从文件加载 |
| `FMIndex::save()` | 保存到文件 |

### `align` 模块

| 结构体/函数 | 说明 |
|------------|------|
| `AlignOpt` | 比对配置 |
| `banded_sw()` | 带状 Smith-Waterman |
| `find_smem_seeds()` | 查找 SMEM 种子 |
| `build_chains()` | 构建种子链 |
| `pipeline::align_reads()` | 完整比对流水线 |

### `io` 模块

| 函数 | 说明 |
|------|------|
| `parse_fasta()` | 解析 FASTA 文件 |
| `parse_fastq()` | 解析 FASTQ 文件 |
| `write_sam_header()` | 写入 SAM 头 |
| `format_sam_record()` | 格式化 SAM 记录 |

### `util` 模块

| 函数 | 说明 |
|------|------|
| `normalize_seq()` | 归一化 DNA 序列 |
| `to_alphabet()` | 编码为 0-5 字母表 |
| `from_alphabet()` | 从 0-5 字母表解码 |
| `revcomp()` | 反向互补 |

---

## 参见

- [教程](../tutorial/) — 用户指南
- [架构](../architecture/) — 实现细节
- [开发](../development/) — 贡献指南
