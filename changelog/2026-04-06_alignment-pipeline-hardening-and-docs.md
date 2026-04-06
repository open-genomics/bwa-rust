# Alignment pipeline hardening and docs

日期：2026-04-06

## 变更内容

- 修复 `src/align/pipeline.rs` 中 rayon 线程池初始化的双重 `unwrap`，改为错误传播，避免线程池构建失败时 panic
- 提取命名常量，消除魔数：
  - `src/align/pipeline.rs`：`MAX_ALIGNMENTS_PER_READ`
  - `src/align/chain.rs`：`MAX_CHAINS_PER_CONTIG`
  - `src/align/extend.rs`：`EXTEND_REF_PAD`
- 优化 `src/align/pipeline.rs` 中 read/qual 与反向互补输出的字符串构建路径，移除 `from_utf8_lossy(...).into_owned()`
- 为 `src/align/seed.rs::MemSeed` 添加 `Copy`，并移除 `src/align/chain.rs` 中两处不必要的 `clone`
- 为公开 API 补充文档注释：
  - `src/align/chain.rs`
  - `src/align/candidate.rs`
  - `src/align/extend.rs`
  - `src/align/sw.rs`
  - `src/util/dna.rs`
  - `src/align/seed.rs`
- 清理 `src/align/chain.rs` 中重复的文档注释
- 在 `src/align/sw.rs` 中补充 `SwParams` / `SwResult` 文档注释，并将 `extend_left` 中的 `.cloned()` 改为 `.copied()`
- 为 `parse_cigar` 补充边界行为测试，明确：未知操作符会被保留、缺少操作符的尾部纯数字会被忽略

## 验证情况

- 尝试执行 `cargo test`
- 当前环境因 crates.io 网络超时失败，未观察到本地代码断言失败：
  - `windows-sys` 下载超时
  - 失败点来自 `criterion` 的传递依赖拉取，而非本次改动逻辑

## 影响

- 提高线程池初始化失败时的健壮性
- 降低维护成本，便于后续调整输出条数、链提取上限和延伸窗口策略
- 减少热路径上的小对象拷贝
- 提升库 API 的可读性与可维护性
