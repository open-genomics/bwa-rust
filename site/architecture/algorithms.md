# 核心算法

## FM-index

索引构建流程：

```text
normalized reference -> alphabet text -> suffix array -> BWT -> C table + Occ samples -> .fm
```

项目字母表：`{0:$, 1:A, 2:C, 3:G, 4:T, 5:N}`。

`.fm` 文件包含：

- magic/version；
- BWT；
- C 表；
- Occ 采样；
- SA 或稀疏 SA；
- contig 元信息；
- 编码后的参考文本。

## SMEM 种子

`seed.rs` 使用 FM-index backward search 找 exact matches，并按 `min_seed_len` 与 `max_occ` 控制种子数量。`max_occ` 对重复区域非常重要，避免高度重复种子拖垮后续链构建。

## 链构建

`chain.rs` 将同一 contig 上方向一致、坐标合理的种子组织成链。链评分偏好近似共线的种子，并通过最大链数限制控制候选数量。

## Smith-Waterman 延伸

`sw.rs` 提供带状仿射 gap SW、半全局对齐和链端延伸。`extend.rs` 负责把链转换为完整 CIGAR：

- 左端和右端使用可配置 `zdrop` 终止延伸；
- 链内 gap 使用全局对齐补齐，避免把中间 indel 裁掉；
- 两端无法对齐的 read 碱基以 soft clip 表示。

## SAM 标签

`pipeline.rs` 对候选排序、分类 primary/secondary/supplementary，并生成 SAM 行。`sam.rs` 生成 MD:Z；`supplementary.rs` 生成 SA:Z。
