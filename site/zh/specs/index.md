# OpenSpec 规范

## 概述

bwa-rust 使用 [OpenSpec](https://github.com/LessUp/openspec) 进行能力驱动开发。每个规范定义一个具有清晰边界的独立能力。

## 规范结构

规范位于 `openspec/specs/`：

```
openspec/specs/
├── 001-core-types.md
├── 002-fm-index.md
├── 003-seeding.md
├── 004-chaining.md
├── 005-alignment.md
├── 006-sam-output.md
├── 007-parallelism.md
├── 008-cli.md
├── 009-validation.md
├── 010-memory-safety.md
└── 011-error-handling.md
```

## 能力概览

| 规范 | 能力 | 状态 |
|------|------|:----:|
| 001 | 核心类型 (DNA, Contig, Read) | <span class="status-badge delivered">✓</span> |
| 002 | FM-index 构建与查询 | <span class="status-badge delivered">✓</span> |
| 003 | SMEM 种子 | <span class="status-badge delivered">✓</span> |
| 004 | 链构建 | <span class="status-badge delivered">✓</span> |
| 005 | Smith-Waterman 比对 | <span class="status-badge delivered">✓</span> |
| 006 | SAM 输出 (CIGAR, MAPQ, Tags) | <span class="status-badge delivered">✓</span> |
| 007 | Rayon 并行 | <span class="status-badge delivered">✓</span> |
| 008 | CLI 接口 | <span class="status-badge delivered">✓</span> |
| 009 | 验证与测试 | <span class="status-badge delivered">✓</span> |
| 010 | 内存安全 (无 unsafe) | <span class="status-badge delivered">✓</span> |
| 011 | 错误处理 | <span class="status-badge delivered">✓</span> |

## 规范格式

每个规范遵循此结构：

```markdown
# Spec: [能力名称]

## Intent
此能力解决什么问题？

## Scope
包含什么和不包含什么？

## Interface
公共 API 和类型。

## Behavior
预期行为和不变量。

## Verification
如何验证正确性。
```

## 阅读规范

理解一个能力：

1. 阅读 `openspec/specs/` 中的规范
2. 检查 `src/` 中的实现
3. 审查 `src/*/tests.rs` 中的测试

## 开发工作流

添加或修改能力时：

1. **提议**: 创建或更新规范
2. **实现**: 编写符合规范的代码
3. **验证**: 为规范要求添加测试
4. **文档**: 更新架构文档

---

[下一篇：验证 →](/zh/specs/validation)
