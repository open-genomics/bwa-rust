# 验证边界

## 概述

本文档记录 bwa-rust 如何验证正确性以及验证的边界。

## 测试覆盖

### 单元测试

每个模块有相邻的测试：

```
src/
├── align/
│   ├── mod.rs
│   ├── sw.rs
│   └── tests.rs  ← 模块测试
├── index/
│   ├── fm.rs
│   └── tests.rs
└── ...
```

### 集成测试

位于 `tests/`：

- `integration_test.rs`: 端到端比对测试
- `sam_output_test.rs`: SAM 格式验证

### 测试数据

参考测试数据位于 `tests/data/`：

- `tiny.fasta`: 用于快速测试的最小参考
- `ecoli_subset.fasta`: 用于集成测试的 E. coli 子集
- `reads.fastq`: 示例 reads

## CI 流水线

### 持续集成

```yaml
# .github/workflows/ci.yml
jobs:
  test:
    - cargo test --all-targets
    - cargo clippy -- -D warnings
    - cargo fmt -- --check
```

### 覆盖率报告

覆盖率被跟踪但不作为硬性阈值强制执行：

- 当前覆盖率：~80%（近似）
- 重点：关键路径（比对、SAM 输出）

## 验证方法

### 1. 格式验证

SAM 输出根据 [SAM 规范](https://samtools.github.io/hts-specs/SAMv1.pdf)验证：

- CIGAR 字符串有效性
- MAPQ 范围 [0, 255]
- Tag 格式 (AS:i, XS:i, NM:i, MD:Z, SA:Z)

### 2. 往返测试

```rust
// 索引往返
let index = FmIndex::build(&reference);
let bytes = index.serialize();
let restored = FmIndex::deserialize(&bytes)?;
assert_eq!(index, restored);
```

### 3. 属性测试

使用 `proptest` 进行：

- DNA 编码/解码
- CIGAR 字符串操作
- 坐标变换

### 4. 已知答案测试

与已知结果比较：

- 小参考（手动计算）
- E. coli 子集（使用外部工具验证）

## 未验证的内容

### 非位级 BWA 兼容

我们**不**验证输出是否与 BWA 完全匹配：

- 不同的浮点决策
- 不同的平局打破规则
- 不同的启发式阈值

### 非生产规模测试

测试侧重于正确性而非规模：

- 人类基因组不在 CI 中（太大）
- 百万 read 数据集不在 CI 中（太慢）

### 非性能回归测试

无自动化性能回归测试：

- Benchmark 是手动的
- 性能因硬件而异

## 手动验证

生产使用前，请验证：

```bash
# 构建索引
bwa-rust index reference.fasta -o index.fm

# 比对测试 reads
bwa-rust align index.fm test.fastq -o output.sam

# 验证 SAM
samtools view -Sb output.sam > /dev/null

# 与 BWA 比较（可选）
bwa mem reference.fasta test.fastq > bwa.sam
# 比较关键指标：比对率、MAPQ 分布
```

---

[下一篇：限制 →](/zh/specs/limitations)
