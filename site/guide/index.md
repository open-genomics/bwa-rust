# 使用指南

欢迎来到 bwa-rust 使用指南。

## 指南目录

| 章节 | 说明 |
|------|------|
| [安装](installation) | 系统要求、构建步骤、预编译二进制 |
| [快速开始](quickstart) | 基础用法、CLI 参数详解、功能概览 |

## 快速导航

### 新用户

1. [安装](installation) — 确保系统满足要求并完成安装
2. [快速开始](quickstart) — 学习基础用法和常用参数

### 开发者

- [架构文档](../architecture/) — 了解内部实现和算法
- [API 文档](https://docs.rs/bwa-rust) — 查看 Rust API 参考
- [GitHub](https://github.com/LessUp/bwa-rust) — 源码和 Issue

## 命令速查

```bash
# 构建索引
bwa-rust index reference.fa -o ref

# 比对序列（使用索引）
bwa-rust align -i ref.fm reads.fq -o output.sam

# 一步比对（BWA-MEM 风格）
bwa-rust mem reference.fa reads.fq -t 8 -o output.sam
```

## 系统要求

- **Rust**: 1.70+ (MSRV)
- **平台**: Linux / macOS / Windows
- **内存**: 建议 8GB+（人类基因组索引构建）
