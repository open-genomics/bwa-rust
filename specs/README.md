# Specification Documents

> **Single Source of Truth** for all development work.

This directory contains all specification documents for the bwa-rust project. Specifications follow the **Spec-Driven Development (SDD)** methodology.

## Directory Structure

```
specs/
├── product/              # 产品功能定义与验收标准 (PRDs)
│   ├── README.md
│   └── core-features.md
├── rfc/                  # 技术设计文档与架构方案 (RFCs)
│   ├── README.md
│   ├── 0001-core-architecture.md
│   ├── 0002-index-building.md
│   └── 0003-alignment-algorithm.md
├── api/                  # 接口规范 (CLI、库 API)
│   ├── README.md
│   └── cli-interface.md
└── testing/              # 测试策略与 BDD 测试用例规范
    ├── README.md
    └── test-strategy.md
```

## Directory Overview

| Directory | Purpose |
|-----------|---------|
| `product/` | Product feature definitions and acceptance criteria (PRDs) |
| `rfc/` | Technical design documents and architecture proposals (RFCs) |
| `api/` | API specifications (CLI interface, library API) |
| `testing/` | BDD test case specifications and acceptance test definitions |

## How to Use

### New Feature
1. Create a product spec in `product/` defining the feature and acceptance criteria
2. Create an RFC in `rfc/` if technical design is needed
3. Update API spec in `api/` if interface changes are required
4. Implement code that strictly follows the specs
5. Write tests against the acceptance criteria

### API Change
1. Update the API spec in `api/` first
2. Review and approve the spec change
3. Implement the code change
4. Update tests

### Bug Fix
1. Reference the relevant spec to understand expected behavior
2. Create a spec if the behavior wasn't previously specified
3. Fix the bug to match spec behavior
4. Add regression test

## Spec-Driven Workflow

```
┌─────────────────────────────────────────────────────────────┐
│                    Spec-Driven Development                   │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  1. Review    ──→  Read existing specs before changes       │
│                                                             │
│  2. Update    ──→  Modify specs first when requirements     │
│                    change                                    │
│                                                             │
│  3. Implement ──→  Write code that strictly follows specs   │
│                                                             │
│  4. Test      ──→  Verify against acceptance criteria       │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Related Documents

| Document | Description |
|----------|-------------|
| [AGENTS.md](../AGENTS.md) | AI agent workflow instructions |
| [CLAUDE.md](../CLAUDE.md) | Claude Code specific context |
| [CONTRIBUTING.md](../CONTRIBUTING.md) | Contribution guidelines |
