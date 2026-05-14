# 限制声明

## 概述

本页面明确说明 bwa-rust **不支持**的内容。诚实的范围声明是项目的核心价值。

## 当前限制

### 仅支持单端

**状态**: 计划于 v0.3.0

bwa-rust 目前仅支持单端 reads：

- ✅ FASTQ 单端输入
- ❌ FASTQ 配对端输入
- ❌ 正确配对推断
- ❌ 插入大小估计

**替代方案**: 对于配对端数据，分别比对每个文件并手动合并。

### 仅 SAM 输出

**状态**: 计划于 v0.5.0

当前输出格式：

- ✅ SAM 文本输出
- ❌ BAM 二进制输出
- ❌ CRAM 压缩输出
- ❌ 坐标排序

**替代方案**: 通过 `samtools view -bS` 管道 SAM：

```bash
bwa-rust align index.fm reads.fq | samtools view -bS -o output.bam -
```

### 无 BWA 兼容性

**状态**: 非目标

bwa-rust **不**设计用于 BWA 输出兼容：

- 不同的索引格式（单文件 vs 多文件）
- 不同的算法决策
- 不同的平局打破规则
- 不同的 MAPQ 计算

**含义**: 不应用于 BWA 兼容性测试。

### 无大参考优化

**状态**: 计划于 v0.4.0

当前对大参考的限制：

- 完全内存索引
- 无流式参考
- 无染色体级别分块

**实际限制**: 适用于 ~1 Gbp。人类基因组（3 Gbp）可能需要大量 RAM。

### 无 GPU 加速

**状态**: 非目标

不计划 GPU 加速：

- 专注于可移植的 CPU 实现
- 使用 Rayon 进行多线程
- 如需要可考虑外部 GPU 封装

## 非目标

这些明确**不**在计划中：

| 功能 | 原因 |
|------|------|
| BWA 索引兼容 | 不同的设计理念 |
| 位级输出匹配 | 不追求兼容性 |
| 实时流式 | 批处理模型 |
| GUI 界面 | 以 CLI 为中心的设计 |
| Windows 优先支持 | Unix 优先，Windows 尽力支持 |

## 何时使用替代方案

### 使用 BWA-MEM 当

- 现在就需要配对端比对
- 需要 BAM/CRAM 输出
- 大规模处理人类基因组
- 需要 BWA 兼容性

### 使用 minimap2 当

- 比对长 reads (PacBio, ONT)
- 需要剪接比对 (RNA-seq)
- 跨物种映射

### 使用 bwa-rust 当

- 学习比对算法
- 开发 Rust 生物信息学工具
- 需要内存安全的实现
- 处理单端数据
- 原型设计新方法

## 未来计划

参见 [ROADMAP.md](https://github.com/LessUp/bwa-rust/blob/master/ROADMAP.md) 了解计划功能。

---

[← 验证](/zh/specs/validation) | [指南 →](/zh/guide/)
