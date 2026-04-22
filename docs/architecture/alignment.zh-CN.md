# 比对算法详解

> 本文档详细介绍 bwa-rust 的比对算法流程，包括 SMEM 种子查找、链构建、Smith-Waterman 对齐和完整流水线。

---

## 目录

- [整体流程](#整体流程)
- [SMEM 种子查找](#smem-种子查找)
- [种子链构建](#种子链构建)
- [Smith-Waterman 对齐](#smith-waterman-对齐)
- [候选管理](#候选管理)
- [完整流水线](#完整流水线)
- [内存防护机制](#内存防护机制)
- [性能优化](#性能优化)

---

## 整体流程

```
Read (FASTQ)
    │
    ▼
┌─────────────────────────────────────┐
│ 1. 序列归一化                        │
│    • DNA 编码 (A/C/G/T/N)            │
│    • 反向互补                        │
└─────────────────────────────────────┘
    │
    ├─ 正向 ─────────┐
    │                │
    ├─ 反向互补 ─────┤
    │                │
    ▼                ▼
┌─────────────────────────────────────┐
│ 2. SMEM 种子查找                     │
│    • 最大精确匹配                    │
│    • 左扩展直到不再最大              │
│    • max_occ 过滤                    │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│ 3. 种子链构建                        │
│    • DP 评分                         │
│    • 贪心剥离提取多链                │
│    • 过滤低分链                      │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│ 4. 链扩展对齐                        │
│    • 参考序列窗口提取                │
│    • 带状 Smith-Waterman             │
│    • CIGAR 生成                      │
│    • 半全局细化                      │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│ 5. 候选管理                          │
│    • 位置/方向去重                   │
│    • 按得分排序                      │
│    • 主/次比对标记                   │
└─────────────────────────────────────┘
    │
    ▼
Output (SAM)
```

---

## SMEM 种子查找

### 定义

SMEM (Super-Maximal Exact Match) 是指：
> 对于 read 上的每个位置，找到覆盖该位置的**最长**精确匹配，且不被任何其他精确匹配包含。

### 算法

```rust
pub fn find_smem_seeds(
    fm: &FMIndex,
    read: &[u8],
    min_len: usize,
    max_occ: usize,
) -> Vec<MemSeed> {
    let mut seeds = Vec::new();
    let mut i = 0;

    while i < read.len() {
        // 从位置 i 开始，向右找最长匹配
        let (seed_len, sa_interval) = extend_right(fm, &read[i..]);

        if seed_len >= min_len && sa_interval.occ() <= max_occ {
            // 尝试向左扩展
            let (left_ext, final_interval) = extend_left(
                fm, &read[..i], sa_interval
            );

            let seed = MemSeed {
                qb: i - left_ext,     // query begin
                qe: i + seed_len,     // query end
                sa_l: final_interval.l,
                sa_r: final_interval.r,
                occ: final_interval.occ(),
            };

            seeds.push(seed);
            i += seed_len;  // 跳过已覆盖区域
        } else {
            i += 1;
        }
    }

    seeds
}
```

### 示例

Read: `ACGTACGTACGT`

```
Forward strand:
Position: 0  1  2  3  4  5  6  7  8  9  10 11
Read:     A  C  G  T  A  C  G  T  A  C  G  T
          └──────── SMEM 1 ──────┘
Seed 1: read[0:8] → ref[1000:1008], occ=1
                         └──── SMEM 2 ────┘
Seed 2: read[4:12] → ref[2000:2008], occ=2

Seeds may overlap, but each is maximal at its position.
```

### 内存防护：max_occ

```rust
// 过滤出现次数过多的种子（如重复序列）
const DEFAULT_MAX_OCC: usize = 500;

if seed.occ > max_occ {
    skip!();  // 防止 poly-A 等重复序列爆炸
}
```

---

## 种子链构建

### 目标

将多个种子组合成一条连贯的"链"，表示 read 可能比对到参考序列的某区域。

### 评分模型

```rust
// 链得分计算
fn chain_score(seeds: &[MemSeed]) -> i32 {
    let mut score = 0i32;

    for window in seeds.windows(2) {
        let (s1, s2) = (&window[0], &window[1]);

        // 种子得分
        score += s1.len() as i32;

        // Gap 罚分
        let gap_cost = if collinear(s1, s2) {
            // 共线：根据距离线性罚分
            gap_penalty_linear(s1, s2)
        } else {
            // 不共线：重罚
            GAP_PENALTY_INCONSISTENT
        };

        score -= gap_cost;
    }

    score
}
```

### DP 链构建

```rust
pub fn build_chains(seeds: &[MemSeed], read_len: usize) -> Vec<Chain> {
    let mut chains = Vec::new();
    let mut used = vec![false; seeds.len()];

    // 按参考位置排序种子
    let mut ordered: Vec<_> = seeds.iter().enumerate().collect();
    ordered.sort_by_key(|(_, s)| s.rb);

    // 贪心提取最佳链
    while !used.iter().all(|&x| x) {
        // 对每个未使用的种子，向后 DP 找最佳链
        let best_chain = dp_find_best_chain(&ordered, &used);

        if best_chain.score < MIN_CHAIN_SCORE {
            break;
        }

        // 标记已使用
        for &idx in &best_chain.seed_indices {
            used[idx] = true;
        }

        chains.push(best_chain);
    }

    chains
}
```

### 链过滤

```rust
pub fn filter_chains(chains: &mut Vec<Chain>, ratio: f32) {
    if chains.is_empty() { return; }

    // 按得分排序
    chains.sort_by_key(|c| -c.score);

    let best_score = chains[0].score;
    let threshold = (best_score as f32 * ratio) as i32;

    // 过滤低分链
    chains.retain(|c| c.score >= threshold);
}
```

---

## Smith-Waterman 对齐

### 带状（Banded）实现

标准 Smith-Waterman 需要 O(m×n) 空间和时间。bwa-rust 使用带状优化：

```rust
pub struct SwParams {
    pub match_score: i32,       // 默认 2
    pub mismatch_penalty: i32,  // 默认 1
    pub gap_open: i32,          // 默认 2
    pub gap_extend: i32,        // 默认 1
    pub band_width: usize,      // 默认 16
}

pub fn banded_sw(
    query: &[u8],
    reference: &[u8],
    params: SwParams,
) -> SwResult {
    let w = params.band_width;
    let mut dp = vec![vec![0i32; 2 * w + 1]; query.len() + 1];

    for i in 1..=query.len() {
        // 只计算带状区域内的单元格
        let center = best_col_from_prev(&dp[i-1]);
        let col_start = center.saturating_sub(w);
        let col_end = (center + w).min(reference.len());

        for j in col_start..=col_end {
            // 仿射间隙 DP
            let match_score = if query[i-1] == reference[j-1] {
                params.match_score
            } else {
                -params.mismatch_penalty
            };

            dp[i][j - col_start] = max(
                dp[i-1][j - col_start - 1] + match_score,  // 匹配/错配
                dp[i-1][j - col_start] - params.gap_extend,  // 纵向 gap
                dp[i][j - col_start - 1] - params.gap_extend,  // 横向 gap
                0,  // Smith-Waterman: 允许局部对齐
            );
        }
    }

    // 回溯生成 CIGAR
    let cigar = traceback(&dp);
    let score = dp.iter().flatten().max().copied().unwrap_or(0);

    SwResult { score, cigar }
}
```

### 复杂度

| 实现 | 时间 | 空间 |
|-----|------|------|
| 标准 SW | O(m×n) | O(m×n) |
| 带状 SW | O(m×w) | O(m×w) |

其中 w 为带宽（默认 16），相比标准实现，对于长序列可节省 99%+ 的时间和空间。

---

## 候选管理

### 流程

```rust
pub fn collect_candidates(
    chains: &[Chain],
    align_opt: &AlignOpt,
) -> Vec<Candidate> {
    let mut candidates = Vec::new();

    for chain in chains {
        // 1. 提取参考窗口
        let ref_window = extract_ref_window(chain, &align_opt);

        // 2. 执行 SW 对齐
        let sw_result = banded_sw(&chain.query, &ref_window, align_opt.sw_params);

        // 3. 过滤低分对齐
        if sw_result.score < align_opt.score_threshold {
            continue;
        }

        // 4. 半全局细化
        let refined = semi_global_refinement(
            &chain.query, &ref_window, sw_result
        );

        candidates.push(Candidate {
            ref_id: chain.ref_id,
            ref_pos: chain.ref_pos,
            score: refined.score,
            cigar: refined.cigar,
            nm: refined.nm,
            is_reverse: chain.is_reverse,
        });
    }

    candidates
}
```

### 去重与排序

```rust
pub fn dedup_candidates(candidates: &mut Vec<Candidate>) {
    // 按 (ref_id, ref_pos, is_reverse) 去重
    candidates.sort_by_key(|c| (c.ref_id, c.ref_pos, c.is_reverse));
    candidates.dedup_by_key(|c| (c.ref_id, c.ref_pos, c.is_reverse));

    // 按对齐质量排序（分数 - clip_penalty）
    candidates.sort_by(|a, b| {
        let score_a = a.score - a.clipped_bases;
        let score_b = b.score - b.clipped_bases;
        score_b.cmp(&score_a)
    });
}
```

---

## 完整流水线

### AlignOpt 配置

```rust
pub struct AlignOpt {
    // 打分参数
    pub match_score: i32,
    pub mismatch_penalty: i32,
    pub gap_open: i32,
    pub gap_extend: i32,
    pub clip_penalty: i32,

    // 对齐参数
    pub band_width: usize,
    pub score_threshold: i32,
    pub min_seed_len: usize,

    // 并行参数
    pub threads: usize,

    // 内存防护
    pub max_occ: usize,
    pub max_chains_per_contig: usize,
    pub max_alignments_per_read: usize,
}

impl AlignOpt {
    pub fn validate(&self) -> Result<(), BwaError> {
        // 验证参数合法性
        if self.band_width == 0 {
            return Err(BwaError::Align("band_width must be > 0".into()));
        }
        // ... 其他验证
        Ok(())
    }
}
```

### 流水线入口

```rust
pub fn align_reads(
    fm: &FMIndex,
    reads: &[FastqRecord],
    opt: &AlignOpt,
) -> Vec<SamRecord> {
    let pool = ThreadPoolBuilder::new()
        .num_threads(opt.threads)
        .build()
        .expect("Failed to create thread pool");

    pool.install(|| {
        reads.par_iter().map(|read| {
            align_single_read(fm, read, opt)
        }).collect()
    })
}

fn align_single_read(
    fm: &FMIndex,
    read: &FastqRecord,
    opt: &AlignOpt,
) -> SamRecord {
    // 正向
    let forward_seeds = find_smem_seeds(fm, &read.seq, opt.min_seed_len, opt.max_occ);
    let forward_chains = build_chains(&forward_seeds, read.seq.len());
    let forward_candidates = collect_candidates(&forward_chains, opt);

    // 反向互补
    let revcomp_seq = revcomp(&read.seq);
    let reverse_seeds = find_smem_seeds(fm, &revcomp_seq, opt.min_seed_len, opt.max_occ);
    let reverse_chains = build_chains(&reverse_seeds, read.seq.len());
    let mut reverse_candidates = collect_candidates(&reverse_chains, opt);
    reverse_candidates.iter_mut().for_each(|c| c.is_reverse = true);

    // 合并、去重、排序
    let mut all = forward_candidates;
    all.extend(reverse_candidates);
    dedup_candidates(&mut all);

    // 选择主/次比对
    select_primary_and_secondary(&all, opt.max_alignments_per_read)
}
```

---

## 内存防护机制

bwa-rust 实现了三层内存防护，防止重复序列（如 poly-A）导致内存爆炸：

```
┌────────────────────────────────────────────────────────────┐
│                     三层内存防护                            │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ 1. max_occ = 500                                     │  │
│  │    跳过 SA 区间 > 500 的种子                          │  │
│  │    防止 poly-A 等重复序列展开数万位置                  │  │
│  └──────────────────────────────────────────────────────┘  │
│                         ↓                                  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ 2. max_chains_per_contig = 5                         │  │
│  │    每个 contig 最多提取 5 条链                        │  │
│  │    防止重复区域产生过多候选                           │  │
│  └──────────────────────────────────────────────────────┘  │
│                         ↓                                  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ 3. max_alignments_per_read = 5                       │  │
│  │    每个 read 最多输出 5 个比对结果                     │  │
│  │    控制最终输出规模                                   │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

---

## 性能优化

### 关键优化点

| 优化项 | 方法 | 效果 |
|--------|------|------|
| 缓冲区复用 | `SwBuffer` 池 | 减少 hot-path 分配 |
| 批处理 | 按染色体批处理种子 | 提高缓存命中率 |
| 并行化 | Rayon parallel iterator | 多核线性加速 |
| 提前过滤 | max_occ + 链得分阈值 | 减少无效计算 |

### 性能基准

在小规模测试数据集（toy.fa, 1K reads）上：

| 操作 | 单线程 | 4 线程 | 8 线程 |
|-----|--------|--------|--------|
| 索引构建 | 0.1s | - | - |
| 比对 | 2.1s | 0.6s | 0.4s |

---

## 相关文档

- [架构总览](./overview.zh-CN.md) — 模块架构概览
- [索引构建详解](./index-building.zh-CN.md) — FM 索引构建
- [快速入门](../tutorial/getting-started.zh-CN.md) — 使用指南
