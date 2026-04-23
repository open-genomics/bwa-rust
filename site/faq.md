# 常见问题

## 一般问题

### bwa-rust 与 BWA-MEM 有什么区别？

**bwa-rust** 是受 BWA-MEM 启发的 Rust 重实现，主要区别：

| 特性 | BWA-MEM | bwa-rust |
|------|---------|----------|
| 实现语言 | C | Rust |
| 内存安全 | 手动管理 | 编译器验证（零 unsafe） |
| 索引格式 | 多文件 (.bwt/.sa/.pac) | 单文件 (.fm) |
| 线程库 | pthread | rayon |
| 配对端支持 | ✅ | 🚧 计划中 |

算法上，bwa-rust 遵循 BWA-MEM 的核心思想（SMEM 种子查找、DP 链构建、Smith-Waterman 对齐），但不追求 100% 行为兼容。

### 为什么比对结果与 BWA 不同？

正常现象。bwa-rust **不追求 100% 行为兼容**：

- 索引构建算法不同（倍增法 O(n log²n) vs DC3 O(n)）
- MAPQ 计算使用简化模型
- 部分边界情况和启发式规则有差异

对于大多数应用，这些差异不影响下游分析的有效性。

### 适合在什么场景使用？

**适合**：

- ✅ 单端短序列比对
- ✅ 需要内存安全保证的研究环境
- ✅ 学习 BWA-MEM 算法实现
- ✅ 作为 Rust 库集成到自定义流程

**不适合**：

- ❌ 需要 100% BWA 兼容性的生产流程
- ❌ 配对端比对（尚未支持，计划中）
- ❌ 需要 BAM 直接输出（尚未支持，计划中）

---

## 安装与编译

### 编译失败怎么办？

确保 Rust 版本 >= 1.70：

```bash
rustc --version
```

如果版本过低，更新 Rust：

```bash
rustup update
```

### 找不到 bwa-rust 命令

如果通过 cargo 安装，确保 `~/.cargo/bin` 在 PATH 中：

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

---

## 使用问题

### 索引构建需要多长时间？

| 参考基因组 | 大小 | 预估时间* |
|------------|------|-----------|
| 大肠杆菌 | ~5MB | < 10 秒 |
| 果蝇 | ~140MB | ~2 分钟 |
| 人类 GRCh38 | ~3GB | ~10-15 分钟 |

*基于开发硬件的粗略估计，实际时间取决于 CPU 性能

### 内存使用如何？

| 操作 | 小基因组（E. coli） | 人类基因组 |
|------|-------------------|-----------|
| 索引构建 | ~500MB | ~5-6GB |
| 比对（单线程） | ~500MB | ~5-6GB |
| 比对（多线程） | 略增 | ~6-8GB |

建议系统内存至少为所需内存的 1.5 倍。

### 如何启用多线程？

使用 `-t` 参数：

```bash
bwa-rust mem ref.fa reads.fq -t 8 -o out.sam
```

推荐线程数等于 CPU 物理核心数，过多线程可能导致性能下降。

---

## 算法问题

### SMEM 种子是什么？

Super-Maximal Exact Match（超级最大精确匹配）：对于 read 上的每个位置，找到覆盖该位置的最长精确匹配，且不被任何其他精确匹配包含。

### 如何处理重复序列？

使用 `max_occ` 参数跳过高度重复的种子：

```bash
bwa-rust mem ref.fa reads.fq --max-occ 100
```

默认值为 500，降低可加快速度但可能错过某些有效比对。

### 链和种子的区别？

- **种子**：单个精确匹配区间（read 位置 → ref 位置）
- **链**：多个种子按照位置关系组合而成的候选比对区域

链构建通过动态规划找到最佳种子组合。

### Smith-Waterman 的带宽是什么意思？

带状 SW 只计算对角线附近 `band_width` 范围内的单元格，将复杂度从 O(n·m) 降到 O(n·band_width)。

默认带宽 100，较小的带宽更快但对 indel 敏感度降低。

---

## 性能调优

### 如何优化性能？

1. **使用多线程**: `-t $(nproc)`
2. **调整 max_occ**: 对重复基因组适当降低（如 `--max-occ 200`）
3. **使用 SSD**: 索引文件放在快速存储上
4. **调整带宽**: 较小带宽（如 `--band-width 50`）更快但敏感度降低

### 索引构建比 BWA 慢

预期行为。当前使用更简单的 SA 构建算法（倍增法 O(n log²n)），比 BWA 的线性时间算法慢约 50%，但实现更简洁易懂。

---

## 输出问题

### SAM 输出与 BWA-MEM 兼容吗？

是的，bwa-rust 生成标准 SAM 格式，可用于所有支持 SAM 的下游工具：

```bash
samtools view -bS output.sam | samtools sort -o output.bam
samtools index output.bam
```

### 如何获取 MAPQ 分数？

bwa-rust 自动计算 MAPQ 分数并输出到 SAM 第 5 列。使用简化的得分差比例模型。

### 支持哪些 SAM 标签？

当前支持：
- `AS:i`: 对齐得分
- `XS:i`: 次优对齐得分
- `NM:i`: 编辑距离

计划中（未来版本）：
- `MD:Z`: 错配字符串

---

## 索引问题

### 索引文件可以在不同版本间共享吗？

索引格式包含版本信息（magic number + version）。主版本号相同时兼容。

遇到版本不兼容时会提示重新构建。

### 可以读取 BWA 的索引文件吗？

不可以。bwa-rust 使用不同的单文件 `.fm` 格式。必须使用 `bwa-rust index` 重新构建索引。

---

## 贡献与开发

### 如何贡献代码？

请参考 [CONTRIBUTING.md](https://github.com/LessUp/bwa-rust/blob/HEAD/CONTRIBUTING.md)。

### 发现 Bug 如何报告？

请在 [GitHub Issues](https://github.com/LessUp/bwa-rust/issues) 报告，包含：
- 操作系统和版本
- bwa-rust 版本 (`bwa-rust --version`)
- 重现步骤
- 错误信息或意外行为描述

### 配对端支持什么时候发布？

配对端对齐仍为后续规划，详见 [ROADMAP.md](https://github.com/LessUp/bwa-rust/blob/HEAD/ROADMAP.md)。
