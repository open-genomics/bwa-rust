---
description: 快速比对测试 — 使用 toy 数据集运行完整的 index + mem 比对管线
---

使用项目自带的 toy 测试数据运行完整的比对管线，验证功能正确性。

// turbo
1. 构建 release 版本：
```bash
cargo build --release
```

2. 使用 `mem` 子命令一步完成索引构建和比对：
```bash
cargo run --release -- mem data/toy.fa data/toy_reads.fq
```

3. 检查输出的 SAM 记录：
   - 是否有 mapped reads（FLAG 不为 4）
   - CIGAR 是否合理（非全 S/全 *）
   - AS 标签分数是否合理
   - 参考名和位置是否正确

4. 如果输出可以保存到文件对比：
```bash
cargo run --release -- mem data/toy.fa data/toy_reads.fq -o test_output.sam
```

5. 汇总比对结果：mapped/unmapped 数量、平均 MAPQ、比对质量概况。
