# 索引构建详解

> 本文档详细介绍 FM 索引的构建流程，包括后缀数组、BWT 变换、FM 索引结构和序列化。

---

## 目录

- [概述](#概述)
- [数据流](#数据流)
- [后缀数组构建](#后缀数组构建)
- [BWT 构建](#bwt-构建)
- [FM 索引构建](#fm-索引构建)
- [内存优化](#内存优化)
- [索引文件格式](#索引文件格式)
- [性能分析](#性能分析)

---

## 概述

bwa-rust 的索引构建流程：

```
FASTA 文件
    │
    ▼
┌─────────────────┐
│ 序列归一化       │ ← 大写转换、过滤非标准字符
│ 多 contig 支持   │ ← 每个 contig 添加 $ 分隔符
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ 后缀数组 (SA)    │ ← O(n log²n) 倍增算法
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ BWT 变换         │ ← 从 SA 生成 BWT
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ FM 索引构建       │ ← C 表 + Occ 采样
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ 序列化           │ ← bincode 编码 → .fm 文件
└─────────────────┘
```

---

## 数据流

### 1. FASTA 解析

**流程：**
```rust
// 1. 读取 FASTA 文件
let fasta = parse_fasta(file)?;

// 2. 序列归一化
text = normalize_seq(seq);  // A/C/G/T/N, uppercase

// 3. 编码转换
for byte in text {
    encoded.push(to_alphabet(byte));  // A→1, C→2, G→3, T→4, N→5
}

// 4. 添加终止符
encoded.push(0);  // $ → 0
```

### 2. 输入验证

| 检查项 | 处理策略 |
|--------|---------|
| 空序列 | 拒绝构建，返回错误 |
| 重复 contig 名 | 拒绝构建，返回错误 |
| 非标准字符 | 转换为 N (5) |
| 不同换行符 | 自动处理 LF/CRLF |

---

## 后缀数组构建

### 倍增算法

**原理：** 通过倍增比较长度逐步排序后缀。

```rust
// 第 k 轮：按前 2^k 个字符排序
fn doubling_sort(k: usize, sa: &mut [usize], rank: &[i32]) {
    // 使用 (rank[i], rank[i+k]) 作为排序键
    sa.sort_by_key(|&i| (rank[i], rank.get(i + k).copied().unwrap_or(-1)));
}

// 整体流程
pub fn build_sa(text: &[u8]) -> Vec<usize> {
    let n = text.len();
    let mut sa: Vec<usize> = (0..n).collect();
    let mut rank: Vec<i32> = text.iter().map(|&c| c as i32).collect();

    let mut k = 1;
    loop {
        // 按 (rank[i], rank[i+k]) 排序
        sa.sort_by_key(|&i| {
            let r1 = rank[i];
            let r2 = if i + k < n { rank[i + k] } else { -1 };
            (r1, r2)
        });

        // 更新 rank
        let mut new_rank = vec![0i32; n];
        new_rank[sa[0]] = 0;
        for i in 1..n {
            new_rank[sa[i]] = new_rank[sa[i-1]]
                + if is_different(&sa, i, k, &rank) { 1 } else { 0 };
        }

        rank = new_rank;
        if rank[sa[n-1]] == (n - 1) as i32 { break; }
        k *= 2;
    }

    sa
}
```

**复杂度分析：**
- 时间复杂度：O(n log²n) —— 每轮 O(n log n) 排序，共 O(log n) 轮
- 空间复杂度：O(n) —— SA 和 rank 数组

### 示例

文本：`"banana$"` (n=7)

| 轮次 (k) | 排序键长度 | SA 状态 |
|---------|-----------|---------|
| 1 | 2 | [6, 5, 3, 1, 0, 4, 2] |
| 2 | 4 | [6, 5, 3, 1, 0, 4, 2] |
| 4 | 8 | [6, 5, 3, 1, 0, 4, 2] ✓ |

最终 SA = [6, 5, 3, 1, 0, 4, 2] 对应后缀：
- SA[0]=6 → "$"
- SA[1]=5 → "a$"
- SA[2]=3 → "ana$"
- SA[3]=1 → "anana$"
- SA[4]=0 → "banana$"
- SA[5]=4 → "na$"
- SA[6]=2 → "nana$"

---

## BWT 构建

### 算法

Burrows-Wheeler Transform 通过 SA 构建：

```
BWT[i] = text[(SA[i] - 1) mod n]
```

**Rust 实现：**
```rust
pub fn build_bwt(text: &[u8], sa: &[usize]) -> Vec<u8> {
    let n = text.len();
    let mut bwt = Vec::with_capacity(n);

    for &sa_i in sa {
        let pos = if sa_i == 0 { n - 1 } else { sa_i - 1 };
        bwt.push(text[pos]);
    }

    bwt
}
```

### 示例

文本：`"abracadabra$"` (n=12)

| i | SA[i] | BWT[i] = text[(SA[i]-1)%n] | 后缀 |
|---|-------|---------------------------|------|
| 0 | 11 | $ | "abracadabra$" |
| 1 | 10 | a | "$" |
| 2 | 7 | a | "abra$" |
| 3 | 0 | $ | "abracadabra$" |
| 4 | 3 | a | "acadabra$" |
| 5 | 5 | c | "adabra$" |
| 6 | 8 | d | "abra$" |
| 7 | 1 | b | "bracadabra$" |
| 8 | 4 | a | "cadabra$" |
| 9 | 6 | a | "dabra$" |
| 10 | 9 | r | "ra$" |
| 11 | 2 | r | "racadabra$" |

BWT = "$aa$acdbraara"]### FM 索引构建

FM 索引由两部分组成：
- **C 表**：记录每个字符在 BWT 中的起始位置
- **Occ 表**：记录 BWT 前缀中每个字符的出现次数

#### C 表

```rust
// C[c] = 字符 c 在 BWT 中首次出现的位置
pub fn build_c(bwt: &[u8], sigma: usize) -> Vec<u32> {
    let mut count = vec![0u32; sigma];

    // 统计各字符出现次数
    for &c in bwt {
        count[c as usize] += 1;
    }

    // 计算累计频率
    let mut c = vec![0u32; sigma + 1];
    for i in 0..sigma {
        c[i + 1] = c[i] + count[i];
    }

    c  // c[0]=0, c[1]=count($), c[2]=count($)+count(A), ...
}
```

#### Occ 采样表

为了减少内存占用，使用块级采样：

```rust
pub struct FMIndex {
    block: u32,           // 采样间隔（默认 64）
    occ_samples: Vec<u32>, // 采样点处的累计计数
}

impl FMIndex {
    // 计算 Occ(c, pos) = BWT[0..pos] 中字符 c 的数量
    pub fn occ(&self, c: u8, pos: usize) -> u32 {
        let block_idx = pos / self.block as usize;
        let sample_idx = block_idx * self.sigma as usize + c as usize;
        let base = self.occ_samples[sample_idx];

        // 从上一个采样点开始计数
        let start = block_idx * self.block as usize;
        let count = self.bwt[start..pos]
            .iter()
            .filter(|&&b| b == c)
            .count() as u32;

        base + count
    }
}
```

**空间优化对比：**

| 方法 | 空间复杂度 | 访问时间 |
|-----|-----------|---------|
| 完整 Occ 表 | O(n × σ) | O(1) |
| 采样 Occ (rate=64) | O(n × σ / 64) | O(64) ≈ O(1) |

对于 DNA 序列（σ=6），采样率 64 可减少约 94% 的内存占用。

#### Backward Search

FM 索引的核心操作：

```rust
pub fn backward_search(&self, pattern: &[u8]) -> Option<(usize, usize)> {
    let mut l: usize = 0;
    let mut r: usize = self.bwt.len();

    for &c in pattern.iter().rev() {
        l = self.c[c as usize] as usize + self.occ(c, l) as usize;
        r = self.c[c as usize] as usize + self.occ(c, r) as usize;

        if l >= r {
            return None;  // 无匹配
        }
    }

    Some((l, r))  // SA 区间 [l, r)
}
```

**时间复杂度：O(m)**，其中 m 为模式长度

---

## 内存优化

### 1. 稀疏 SA 采样

```rust
pub struct FMIndex {
    sa: Vec<u32>,           // 只存储采样点
    sa_sample_rate: u32,    // 采样间隔（默认 4）
}

impl FMIndex {
    // 获取任意 SA 位置的值
    pub fn sa(&self, i: usize) -> Option<u32> {
        if i % self.sa_sample_rate as usize == 0 {
            // 直接返回采样值
            Some(self.sa[i / self.sa_sample_rate as usize])
        } else {
            // 通过 LF-mapping 回溯
            self.lf_mapping_backtrack(i)
        }
    }
}
```

**内存节省：**
- 完整 SA: n × 4 bytes
- 稀疏 SA (rate=4): n × 1 byte（75% 节省）

### 2. Occ 块采样

- 块大小：64（可配置）
- 每个字符存储 u32 计数
- 内存：n × 6 × 4 / 64 ≈ n × 0.375 bytes

### 3. 综合内存估算

对于长度为 n 的参考序列：

| 组件 | 内存占用 |
|-----|---------|
| BWT | n bytes |
| C 表 | 28 bytes (7 × u32) |
| Occ 采样 | n × 0.375 bytes |
| SA 采样 | n bytes |
| 文本 | n bytes |
| Contig 元数据 | ~KB 级别 |
| **总计** | **≈ 2.4n 字节** |

对于人类基因组（≈3G bases）：约 7.2 GB（原始 FASTA 约 3 GB）

---

## 索引文件格式

### 文件结构

```
.magic: [u8; 8]       = b"BWAFM_RS" (0x424D4146_4D5F5253)
.version: u32         = 2

.sigma: u8            = 6 ($, A, C, G, T, N)
.block: u32           = 64

.c: Vec<u32>          = 长度 σ+1 = 7
.bwt: Vec<u8>         = 长度 n
.occ_samples: Vec<u32> = 长度 (n/block) × σ

.sa: Vec<u32>         = 长度 n/sa_sample_rate
.sa_sample_rate: u32  = 4

.contigs: Vec<Contig>  # contig 元信息
.text: Vec<u8>         # 原始编码文本（可选）

.meta: Option<IndexMeta>  # 构建元信息
```

### Contig 结构

```rust
pub struct Contig {
    pub name: String,   // 染色体名称
    pub len: u32,       // 长度
    pub offset: u64,    // 在拼接序列中的起始位置
}
```

### IndexMeta 结构

```rust
pub struct IndexMeta {
    pub version: String,
    pub build_time: String,  // ISO 8601 格式
    pub sequences: u32,      // contig 数量
    pub total_bases: u64,    // 总碱基数
}
```

---

## 性能分析

### 构建时间

| 操作 | 复杂度 | 典型时间 (1亿 bp) |
|-----|--------|------------------|
| FASTA 解析 | O(n) | ~1s |
| SA 构建 | O(n log²n) | ~30s |
| BWT 构建 | O(n) | ~1s |
| FM-index 构建 | O(n) | ~2s |
| 序列化 | O(n) | ~3s |
| **总计** | - | **~37s** |

### 查询性能

| 操作 | 复杂度 | 典型时间 |
|-----|--------|---------|
| Backward Search | O(m) | <1µs (100bp read) |
| SA 位置查询 | O(rate) | <1µs |
| 完整比对 | O(read_len + ref_window) | 0.1-1ms |

### 对比 BWA

| 指标 | bwa-rust | BWA (C) |
|-----|----------|---------|
| SA 算法 | 倍增 O(n log²n) | SA-IS O(n) |
| 构建时间 | 较慢 | 快 3-5x |
| 索引大小 | 相似 | 相似 |
| 查询速度 | 相似 | 相似 |

---

## 相关文档

- [架构总览](./overview.zh-CN.md) — 模块架构概览
- [比对算法详解](./alignment.zh-CN.md) — 完整比对流程
- [快速入门](../tutorial/getting-started.zh-CN.md) — 使用指南
