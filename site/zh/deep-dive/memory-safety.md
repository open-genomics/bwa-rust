# 内存安全

## 概述

bwa-rust 通过 Cargo lint 配置强制执行**零 unsafe 代码**策略。这一设计选择提供了对生物信息学应用至关重要的强内存安全保证。

## 策略执行

```toml
# Cargo.toml
[lints]
unsafe_code = "forbid"
```

此配置使任何 `unsafe` 块的使用成为**编译错误**，而不仅仅是警告。

## 这意味着什么

### 编译器保证

- **无缓冲区溢出**：数组访问有边界检查
- **无释放后使用**：所有权系统防止悬空引用
- **无数据竞争**：编译时强制线程安全
- **无空指针解引用**：`Option<T>` 替代可空指针

### 不声称的内容

- **算法正确性**：安全保证不确保算法正确
- **性能优化**：安全代码可能比手动调优的 unsafe 代码慢
- **BWA 兼容性**：输出格式类似但不与 BWA 位级相同

## 设计理念

### 为什么禁止 unsafe？

1. **可审计性**：每行代码默认安全
2. **学习价值**：学生可以阅读整个代码库而无需 unsafe 顾虑
3. **安全性**：适合处理不可信的基因组数据
4. **可维护性**：重构更安全，无需维护 unsafe 不变量

### 权衡

| 方面 | 有 unsafe | 无 unsafe |
|------|-----------|-----------|
| 性能 | 可能更快 | 满足预期用途 |
| SIMD | 可使用 intrinsics | 使用可移植替代方案 |
| FFI | 可调用 C 库 | 仅纯 Rust |
| 内存布局 | 可优化 | 依赖编译器 |

## 代码示例

### 安全数组访问

```rust
// 在 bwa-rust 中这将是编译错误
let value = unsafe { *ptr.offset(i) }; // ❌ 被禁止

// 改用安全索引
let value = vec[i]; // ✅ 边界检查
```

### 安全并发

```rust
use rayon::prelude::*;

// 无数据竞争的并行处理
reads.par_iter().for_each(|read| {
    // 每次迭代由 Rust 所有权规则隔离
    let alignment = align_read(read, &index);
});
```

## 验证

策略在编译时验证：

```bash
cargo build
# 任何 unsafe 代码将导致: error: usage of an `unsafe` block
```

## 参考资料

- [Rust 安全保证](https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html)
- [Cargo Lint 级别](https://doc.rust-lang.org/cargo/reference/manifest.html#the-lints-section)

---

[下一篇：性能分析 →](/zh/deep-dive/performance)
