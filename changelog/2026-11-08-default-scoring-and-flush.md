# 变更记录：修正默认打分参数与文件输出 flush

- 日期：2026-11-08
- 变更类型：比对正确性修复 + 输出错误传播修复

## 内容

### 1. 默认打分改为 BWA-MEM 风格

`src/align/mod.rs` 的 `AlignOpt::default()`：

- match / mismatch：`2 / 1` → `1 / 4`
- gap open：`2` → `6`
- clip penalty：`1` → `0`

旧默认下，随机序列约 1/3 碱基一致即可获得正分，链端延伸会沿非同源尾部
持续到整条 query，导致部分匹配被扩成带大量错配的全长比对，且 query 区间
全部重叠，supplementary/SA:Z 路径在真实管线中永不触发。

同步更新 `README.md` 默认参数表。

### 2. 文件输出显式 flush

`src/align/pipeline.rs` 的 `align_fastq_with_fm_opt` 在返回前调用
`out_box.flush()?`。此前 `BufWriter` 仅在 Drop 时 flush 并丢弃错误：
写入 `/dev/full`（模拟磁盘满/配额超限）时，单条 read 的小输出会在最后一次
flush 失败但仍以成功退出，留下截断或空 SAM。

## 新增回归测试

- `tests/chimeric.rs`
  - `partial_match_is_soft_clipped_at_default_params`：40bp 精确 + 40bp 无关
    尾部，默认参数下必须为 `40M40S`、`NM:i:0`。
  - `chimeric_read_emits_supplementary_with_sa_tag`：两个相距 290bp 的 60bp
    精确片段，必须输出 primary + supplementary（FLAG 0x800），CIGAR 为
    `60M60S` / `60S60M`，两条均带 `SA:Z`。
- `tests/flush.rs`（Linux）：单条 read 输出到 `/dev/full` 必须返回 `Err`。
- 两个测试均先在未修复 HEAD 上确认失败，修复后通过。

## 影响

- 默认参数下部分匹配与嵌合 read 输出符合 BWA-MEM 语义；短 read 的有效
  得分阈值随 match=1 提高（20bp 单错配 read 在默认阈值 20 下变为 unmapped）。
- 写入磁盘失败不再被静默吞掉；CLI 以非零退出码报错。
- 库 API 签名、索引格式、SAM 字段布局不变。

## 验证

- `cargo fmt --all -- --check`：通过
- `cargo clippy --all-targets --all-features --jobs 4 -- -D warnings`：通过
- `cargo test --all-targets --all-features --jobs 4`：lib 191、main 2、
  chimeric 2、cli 3、flush 1、integration 11、sw_golden 12，全部通过
- 手工验证：`bwa-rust mem data/toy.fa data/toy_reads.fq -o /dev/full`
  报 `No space left on device (os error 28)`，退出码 1
