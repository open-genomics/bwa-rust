---
description: 构建项目 — 编译 debug 或 release 版本
---

构建 bwa-rust 项目。

// turbo
1. 先清理旧的编译产物（可选，仅在遇到编译缓存问题时执行）：
```bash
cargo clean
```

// turbo
2. 构建 debug 版本（快速迭代）：
```bash
cargo build
```

// turbo
3. 构建 release 版本（优化性能）：
```bash
cargo build --release
```

4. 确认构建成功，报告二进制文件路径和大小。
