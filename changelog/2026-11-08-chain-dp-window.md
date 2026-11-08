# 2026-11-08 种子链 DP 的前驱窗口剪枝

## 问题

`best_chain` 对每个种子回扫全部更早的种子，`build_chains_with_limit` 在剥离
子集上反复调用它，种子多时退化为 O(S²)。在 2 万条 MEM 种子的典型比对里，
纯 DP 比较就占了链构建的主要时间。

## 改动

按 `(contig, qb, rb)` 排序后可链接前驱必须满足
`si.qb - sj.qe <= max_gap`，而 `sj.qe <= sj.qb + max_seed_len`，因此

```
si.qb - sj.qb > max_gap + max_seed_len  =>  该前驱必然不可链接
```

于是为每个 `t` 维护一个只前进的下界指针 `lo`：同一 contig 内 `qb` 单调不减，
`lo` 只会在窗口下界推进时前进，扫描范围从 `0..t` 收窄为 `lo..t`（窗口内种子数）。
contig 改变时重置到新分组起点。遍历顺序仍是「由早到晚」的前向顺序，
`cand > dp[t]` 的并列取舍与旧实现完全一致。

## 验证

- 新增 `window_tests::windowed_best_chain_matches_naive_on_random_seeds`：
  300 轮随机种子（最多 40 条、3 个 contig、随机 `max_gap`）与内联的原始
  O(S²) 实现对拍，`Chain` 完全相等（含种子顺序与得分）。
- 新增 `windowed_best_chain_matches_naive_on_clustered_seeds`：100 轮把种子
  挤在 40 bp 窗口内的极端分布，确认剪枝既不漏链也不改变并列选择。
- `cargo test --quiet`：8 个测试目标全部通过（lib 200 项）。
- `cargo clippy --all-targets -- -D warnings`、`cargo fmt --all -- --check` 通过。
- release 粗测（2000 条种子、跨度 10 万 / 2000，5000 条种子、跨度 20 万）：
  单次 `best_chain` 0.12 ms / 0.45 ms / 0.23 ms。

## 未做

- 仍保留 O(S²) 最坏情况（所有种子落在同一窗口内）；进一步优化需要按 qb 的
  单调性做分块或单调队列 DP，收益与风险不成比例。
- 该仓库已被上游归档为只读，改动只能在本地提交，无法推送。
