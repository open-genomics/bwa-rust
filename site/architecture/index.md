# 架构概览

bwa-rust 保持一个小而清晰的单端比对流水线：

```text
FASTA reference
      |
      v
FM-index (.fm)
      |
FASTQ read -> SMEM seeds -> chains -> Smith-Waterman -> ranked candidates -> SAM
```

## 模块边界

| 模块 | 责任 |
|------|------|
| `src/io/fasta.rs` | 读取参考序列并归一化。 |
| `src/io/fastq.rs` | 读取 reads，校验 seq/qual。 |
| `src/index/` | 构建 SA、BWT、FM-index 并序列化 `.fm`。 |
| `src/align/seed.rs` | 通过 FM-index 查找 SMEM/MEM 种子。 |
| `src/align/chain.rs` | 链构建、弱链过滤、候选数量控制。 |
| `src/align/extend.rs` + `sw.rs` | 链端延伸、链内 gap 补齐、CIGAR/NM/score。 |
| `src/align/pipeline.rs` | 正反向候选、排序、MAPQ、SAM 行生成。 |
| `src/io/sam.rs` | SAM header/record/MD:Z/SA:Z 格式化。 |

## 设计取舍

- 索引格式不兼容 BWA，而是选择单文件 `.fm`。
- SA 构建使用清晰的倍增法实现，不追求最优 O(n) 构建速度。
- MAPQ 和启发式是 BWA-MEM 风格，不保证与 BWA 完全一致。
- 当前只承诺单端 reads；配对端相关基础设施不代表已交付 CLI 能力。

## 继续阅读

- [核心算法](/architecture/algorithms)
- [比对流水线](/architecture/pipeline)
