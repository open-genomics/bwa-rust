# 性能与验证边界

bwa-rust 的性能目标是提供清晰、内存安全、可调参的 Rust 单端比对基线，而不是在当前版本追求完全替代 BWA。

## 当前判断

| 项目 | 状态 |
|------|------|
| 微基准 | `cargo bench` 覆盖 FM-index search、SMEM、SW、SA 构建等热点。 |
| 单端吞吐 | 支持 rayon read 级并行。 |
| 人类基因组生产验证 | 未宣称完成。 |
| BWA bit-level 兼容 | 非目标。 |

## 本地运行

```bash
cargo bench
```

CI 不定时自动跑基准，因为 GitHub-hosted runner 噪声较大，且没有稳定基线趋势系统。性能相关变更应在本地或专用环境运行基准并记录对比条件。

## 正确性验证

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

真实数据测试位于 `tests/real_data.rs`，默认 ignored，需要显式准备数据和 feature 后运行。
