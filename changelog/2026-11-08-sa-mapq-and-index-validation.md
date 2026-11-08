# 2026-11-08 SA:Z MAPQ、FM 索引防御与后缀数组秩宽

## SA:Z 的 mapQ

`generate_sa_tag` 此前把所有条目的 mapQ 写成 0。现在 pipeline 为每条候选计算
MAPQ（primary 用全局次优，supplementary 用自身与所有重叠且已输出候选的次优分，
secondary 保持 0），并把 `mapqs` 传给 SA 生成器。`tests/chimeric.rs` 断言 SA
条目中的 mapQ 与被引用记录自身的 MAPQ 一致。

## 外部 `.fm` 索引

- `validate()` 增加 C 表校验：必须从 0 开始、非递减、末项不超过 BWT 长度。
- `occ()` / `rank_range()` 对越界字符与区间做饱和处理，不再直接索引 panic。
- `sa_value()` 的 LF 映射加迭代上限；环状损坏索引返回 `u32::MAX` 而不是死循环。
- `sa_interval_positions` / `for_each_sa_interval_position` 对 `l/r` 做 clamp。

## semiglobal 重打分使用 band_width

新增 `semiglobal_align_banded_with_buf`：以链内种子估计的对角线为中心，只填充
`|j - i - center| <= band_width` 的单元；候选重打分改用它。带外保持 `NEG_INF`，
末端选取与回溯都不会越出带。测试断言带内结果与未带版本 score/CIGAR/区间一致。

## 后缀数组秩宽

`build_sa` 的秩数组从 `i32` 改为 `i64`，避免参考文本超过 2^31 bp 时累加溢出；
同时对超过 `u32::MAX` 的文本长度显式报错（SA 对外是 `u32`）。

## 验证

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-targets --all-features`：197 lib + 2 main + 2 chimeric + 3 cli + 1 flush +
  11 integration + 12 sw_golden，全部通过。
