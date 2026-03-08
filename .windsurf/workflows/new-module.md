---
description: 新建模块 — 在项目中创建新的 Rust 模块，遵循项目规范
---

在 bwa-rust 项目中创建一个新的模块，确保遵循项目的代码规范和结构约定。

1. 确认新模块属于哪个父模块（`io/`、`index/`、`align/`、`util/`），或者是否需要创建新的顶层模块。

2. 创建新的 `.rs` 文件，文件头包含模块级文档注释（中文）：
```rust
//! 模块说明
//!
//! 详细描述该模块的功能和设计思路。
```

3. 在父模块的 `mod.rs`（或 `lib.rs`）中添加 `pub mod` 声明。

4. 如果是算法模块（`align/` 或 `index/`），参考 bwa-0.7.19 对应的 C 实现：
   - 注明参考的 C 源文件和函数名
   - 使用 Rust 惯用法重写，不做逐行翻译
   - 禁止 unsafe 代码

5. 为新模块编写基本的单元测试（`#[cfg(test)] mod tests`）。

6. 如果是性能关键路径，在 `benches/benchmarks.rs` 中添加 criterion 基准。

7. 运行检查确认一切正常：
```bash
cargo fmt --all -- --check
cargo clippy -- -D warnings
cargo test
```
