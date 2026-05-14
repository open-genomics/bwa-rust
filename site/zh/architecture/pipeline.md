# 比对流水线

## 1. 输入归一化

FASTA 与 FASTQ 输入统一映射到大写 DNA 字母。未知碱基映射为 `N`，内部字母表保留 `$` 作为哨兵。

## 2. 正反向候选

每条 read 同时尝试正向与反向互补。若输出 FLAG 包含 `0x10`，SAM 中的 `SEQ` 为反向互补序列，`QUAL` 同步反转。

## 3. 种子与链

SMEM 种子经过 occurrence 过滤后进入链构建。链构建按 contig 分组，并保留有限数量高分链，避免重复序列造成候选爆炸。

## 4. 延伸与精修

每条链先转换为近似对齐，再尝试半全局精修。候选排序使用 raw score、soft-clip penalty、NM、contig、位置和方向做稳定 tie-break。

## 5. 输出控制

`max_alignments_per_read` 控制每条 read 输出的记录数。低于 `score_threshold` 的候选不输出；没有有效候选则输出 unmapped 记录。

## 6. SAM 辅助标签

- `AS:i`: 当前 alignment score。
- `XS:i`: 次优 alignment score。
- `NM:i`: edit distance-like mismatch/indel count。
- `MD:Z`: 参考侧 mismatch/deletion 描述。
- `SA:Z`: chimeric/supplementary alignment 描述。

soft-clipped reads 的 MD:Z 生成使用与完整 CIGAR 坐标一致的 query slice，避免 soft clip 导致坐标错位。
