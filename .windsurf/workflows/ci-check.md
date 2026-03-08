---
description: 本地 CI 检查 — 模拟 GitHub Actions CI 流程，在提交前确保代码质量
---

在提交代码前，执行完整的本地 CI 检查流程，模拟 GitHub Actions 中的 CI 管线。

// turbo
1. 检查代码格式：
```bash
cargo fmt --all -- --check
```

// turbo
2. 运行 Clippy 静态分析（所有警告视为错误）：
```bash
cargo clippy -- -D warnings
```

// turbo
3. 运行全部测试（单元测试 + 集成测试）：
```bash
cargo test
```

// turbo
4. 构建 release 版本确认编译通过：
```bash
cargo build --release
```

5. 如果以上全部通过，输出总结：哪些步骤通过了，是否有需要关注的警告。如果有失败，分析失败原因并提供修复建议。
