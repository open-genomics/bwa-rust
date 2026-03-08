---
description: 与 BWA 对比 — 将 bwa-rust 的输出与原版 BWA 进行比较分析
---

将 bwa-rust 的比对结果与原版 BWA (bwa-0.7.19) 进行对比，验证正确性和一致性。

1. 确认用户已安装原版 `bwa`，如果没有则提示安装方式。

2. 使用 bwa-rust 运行比对：
```bash
cargo run --release -- mem data/toy.fa data/toy_reads.fq -o bwa_rust_output.sam
```

3. 使用原版 bwa 运行比对（如果可用）：
```bash
bwa index data/toy.fa
bwa mem data/toy.fa data/toy_reads.fq > bwa_original_output.sam
```

4. 如果 `scripts/compare.sh` 可用，使用它进行对比：
```bash
bash scripts/compare.sh bwa_rust_output.sam bwa_original_output.sam
```

5. 手动对比关键字段：
   - 比对位置 (POS) 差异
   - CIGAR 差异
   - MAPQ 差异
   - FLAG 差异
   - 总 mapped/unmapped 比例

6. 总结差异原因，区分：
   - **预期差异**：算法简化导致的合理差异
   - **潜在 Bug**：需要修复的不一致
