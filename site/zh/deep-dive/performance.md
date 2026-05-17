# 性能分析

## 概述

本文档记录 bwa-rust 的复杂度分析、Benchmark 方法和性能特征。

## 复杂度分析

### 索引构建

| 操作 | 时间复杂度 | 空间复杂度 |
|------|-----------|-----------|
| 后缀数组 (Doubling) | O(n log² n) | O(n) |
| BWT 构建 | O(n) | O(n) |
| FM-index 序列化 | O(n) | O(n) |

n = 参考基因组长度。

### 比对操作

| 操作 | 时间复杂度 | 说明 |
|------|-----------|------|
| 后向搜索 | O(m) | m = read 长度 |
| SMEM 种子 | O(m) | 与 read 长度线性 |
| 链构建 | O(k²) | k = 种子数量 |
| Smith-Waterman | O(w × l) | w = 带宽, l = 比对长度 |

## Benchmark 方法论

### 测试数据集

| 数据集 | 大小 | 来源 |
|--------|------|------|
| E. coli K-12 | 4.6 Mbp | RefSeq NC_000913.3 |
| S. cerevisiae | 12 Mbp | RefSeq R64 |
| Human chr22 | 51 Mbp | GRCh38 |

### 指标

- **吞吐量**: 每秒比对的 reads 数
- **延迟**: 每次 read 比对时间
- **内存**: 运行时峰值 RSS
- **准确性**: 比对率和正确性

### 运行 Benchmark

```bash
# 构建索引
bwa-rust index reference.fasta -o reference.fm

# 计时比对
time bwa-rust align reference.fm reads.fastq -o output.sam

# 内存分析 (Linux)
/usr/bin/time -v bwa-rust align reference.fm reads.fastq -o output.sam
```

## 性能特征

### 索引速度

现代 CPU 上典型的索引时间（单线程）：

| 参考序列 | 大小 | 时间 | 峰值内存 |
|----------|------|------|----------|
| E. coli | 4.6 Mbp | ~3s | ~50 MB |
| Yeast | 12 Mbp | ~8s | ~120 MB |
| Chr22 | 51 Mbp | ~30s | ~500 MB |

### 比对吞吐量

单端比对（单线程）：

| 参考序列 | Reads | Reads/秒 |
|----------|-------|----------|
| E. coli | 1M 100bp | ~50,000 |
| Yeast | 1M 100bp | ~45,000 |
| Chr22 | 1M 100bp | ~40,000 |

使用 Rayon 并行（8 线程）：

| 参考序列 | Reads | Reads/秒 |
|----------|-------|----------|
| E. coli | 1M 100bp | ~300,000 |
| Yeast | 1M 100bp | ~280,000 |
| Chr22 | 1M 100bp | ~250,000 |

## 权衡决策

### SA 采样间隔

默认间隔：4

| 间隔 | 内存 | 查询时间 |
|------|------|----------|
| 1 | 100% | 最快 |
| 4 | 25% | +O(4) 开销 |
| 8 | 12.5% | +O(8) 开销 |

### 带宽

默认：16

| 宽度 | 灵敏度 | 速度 |
|------|--------|------|
| 8 | 较低 | 较快 |
| 16 | 平衡 | 平衡 |
| 32 | 较高 | 较慢 |

## 优化建议

1. **使用并行**: 设置 `--threads` 匹配 CPU 核心数
2. **调整 `max_occ`**: 较低值减少重复区域的种子爆炸
3. **调优 `min_seed_len`**: 更长的种子减少假阳性
4. **考虑参考大小**: 非常大的参考可能需要内存优化

## 与 BWA 对比

::: warning 非位级兼容
bwa-rust 受 BWA-MEM 启发，但不追求位级输出兼容性。结果相似但不完全相同。
:::

| 方面 | bwa-rust | BWA-MEM |
|------|----------|---------|
| 安全性 | 零 unsafe | 包含 unsafe |
| 索引格式 | 单文件 | 多文件 |
| 配对端 | 计划中 | 支持 |
| BAM 输出 | 计划中 | 支持 |

## 详细分析

使用性能分析工具：

```bash
# 带调试符号构建
cargo build --release --features profiling

# 使用 perf (Linux)
perf record -g target/release/bwa-rust align ...
perf report
```

---

[← 内存安全](/zh/deep-dive/memory-safety) | [规范 →](/zh/specs/)
