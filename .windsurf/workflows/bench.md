---
description: 运行基准测试 — 使用 criterion 执行性能基准，分析热点
---

运行 criterion 基准测试来评估性能。

// turbo
1. 运行全部基准测试：
```bash
cargo bench
```

2. 查看 `target/criterion/` 下的报告，总结各基准的执行时间。

3. 如果有性能回退（regression），分析可能的原因并建议优化方向。重点关注：
   - SW 对齐的内存分配
   - SMEM 种子查找的缓存命中率
   - FM 索引查询的 Occ 采样效率
