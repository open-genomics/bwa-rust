# Contributing Guide

感谢你对 bwa-rust 项目的关注！本文档将帮助你参与项目贡献。

---

## 规范驱动开发 (SDD)

本项目严格遵循**规范驱动开发（Spec-Driven Development）**范式。

- **`/specs/product/`**: 产品功能定义与验收标准
- **`/specs/rfc/`**: 技术设计文档与架构方案
- **`/specs/api/`**: API 接口规范
- **`/specs/testing/`**: BDD 测试用例规范

**工作流要求**：
1. 新功能开发必须先更新 Spec 文档
2. 代码实现必须 100% 遵守 Spec 定义
3. 测试用例需覆盖 Spec 中的验收标准

### 如何参与编写 Spec

1. **新功能**：在 `specs/product/` 创建或更新功能规范，定义验收标准
2. **技术设计**：在 `specs/rfc/` 创建 RFC 文档，编号遵循 `NNNN-title.md` 格式
3. **API 变更**：在 `specs/api/` 更新接口规范
4. **测试规范**：在 `specs/testing/` 定义测试策略

详见 [AGENTS.md](AGENTS.md) 中的 AI 工作流指令。

---

## 快速开始

```bash
# 1. Fork 并克隆仓库
git clone https://github.com/<your-username>/bwa-rust.git
cd bwa-rust

# 2. 构建项目
cargo build

# 3. 运行测试
cargo test

# 4. 创建特性分支
git checkout -b feature/your-feature
```

---

## 贡献方式

### 🐛 报告 Bug

在提交 Issue 前，请先：

1. 在 [Issues](https://github.com/LessUp/bwa-rust/issues) 搜索是否已有相同问题
2. 确认使用的是最新版本
3. 准备好以下信息：
   - 问题描述和复现步骤
   - 期望行为 vs 实际行为
   - 环境信息（OS、Rust 版本）
   - 相关日志或错误信息

### 💡 提出新功能

1. 先开 Issue 讨论你的想法
2. 说明功能的使用场景
3. 等待维护者反馈后再开始实现

### 🔧 提交代码

```bash
# 1. 创建分支
git checkout -b feature/your-feature

# 2. 编写代码和测试
cargo test

# 3. 检查代码质量
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings

# 4. 提交（遵循 Conventional Commits）
git commit -m "feat: add your feature"

# 5. 推送并创建 PR
git push origin feature/your-feature
```

---

## 代码规范

### 格式化

```bash
cargo fmt --all             # 应用格式
cargo fmt --all -- --check  # CI 检查
```

### Lint 规则

```bash
# 与 CI 一致的 lint 命令
cargo clippy --all-targets --all-features -- -D warnings
```

项目配置了 `clippy.toml`：

| 规则 | 值 |
|------|-----|
| `cognitive-complexity-threshold` | 30 |
| `max-fn-lines` | 200 |
| `max-fn-params` | 8 |
| `msrv` | "1.70" |

### 安全性

```toml
[lints.rust]
unsafe_code = "forbid"  # 全项目禁止 unsafe
```

**禁止使用 `unsafe` 代码块**。

### 提交信息规范

遵循 [Conventional Commits](https://www.conventionalcommits.org/)：

| 前缀 | 说明 | 示例 |
|------|------|------|
| `feat:` | 新功能 | `feat: add paired-end alignment support` |
| `fix:` | Bug 修复 | `fix: correct MAPQ calculation for secondary alignments` |
| `docs:` | 文档变更 | `docs: update installation instructions` |
| `refactor:` | 代码重构 | `refactor: simplify seed chain scoring logic` |
| `test:` | 测试相关 | `test: add edge cases for SMEM finding` |
| `perf:` | 性能优化 | `perf: optimize BWT construction with SIMD` |
| `ci:` | CI/CD 相关 | `ci: add Windows build target` |
| `chore:` | 其他杂项 | `chore: update dependencies` |

### 测试要求

- 新功能必须附带单元测试
- Bug 修复应包含回归测试
- 当前测试覆盖：**168 项测试**（151 单元 + 11 集成 + 5 模块 + 1 文档）

```bash
# 运行所有测试
cargo test --all-targets --all-features

# 运行单个测试
cargo test <test_name> -- --nocapture

# 运行基准测试
cargo bench
```

---

## 项目结构

```
bwa-rust/
├── specs/               # 规范文档 (Single Source of Truth)
│   ├── product/         # 产品功能定义
│   ├── rfc/             # 技术设计 RFC
│   ├── api/             # API 接口规范
│   └── testing/         # 测试策略
├── docs/                # 用户文档与开发文档
│   ├── tutorial/        # 教程
│   ├── architecture/    # 架构说明
│   ├── api/             # API 使用指南
│   └── development/     # 开发指南
├── src/
│   ├── main.rs          # CLI 入口（clap）
│   ├── lib.rs           # Library 入口
│   ├── error.rs         # BwaError / BwaResult<T>
│   ├── io/              # 输入输出
│   ├── index/           # FM 索引
│   ├── align/           # 比对算法
│   └── util/            # 工具函数
├── tests/               # 集成测试
├── benches/             # Criterion 基准测试
├── examples/            # 示例代码
├── data/                # 测试数据
└── site/                # VitePress 文档站点
```

---

## 扩展开发

### 添加新的比对参数

1. 在 `src/align/mod.rs` 的 `AlignOpt` 结构体中添加字段
2. 在 `validate()` 方法中添加校验逻辑
3. 在 `src/main.rs` 的 CLI 参数中添加对应选项
4. 添加单元测试验证

### 添加新的输出格式

1. 在 `src/io/` 下创建新模块
2. 实现 `write_header` 和 `write_record` 函数
3. 在 `src/align/pipeline.rs` 中集成

---

## CI 流程

GitHub Actions 自动执行：

```yaml
jobs:
  ci:
    steps:
      - cargo fmt --all -- --check
      - cargo clippy --all-targets --all-features -- -D warnings
      - cargo test
      - cargo build --release
```

---

## 相关文档

| 文档 | 说明 |
|------|------|
| [AGENTS.md](AGENTS.md) | AI 编程助手完整指南 |
| [README.md](README.md) | 项目介绍（英文）|
| [README.zh-CN.md](README.zh-CN.md) | 项目介绍（中文）|
| [specs/](specs/) | **规范文档 (Single Source of Truth)** |
| [docs/](docs/) | 用户教程与架构文档 |
| [ROADMAP.md](ROADMAP.md) | 开发路线图 |
| [CHANGELOG.md](CHANGELOG.md) | 变更日志 |

---

## 获取帮助

- 💬 [GitHub Discussions](https://github.com/LessUp/bwa-rust/discussions)
- 📧 [Issues](https://github.com/LessUp/bwa-rust/issues)

---

## 许可证

参与贡献即表示你同意你的贡献将以 [MIT 许可证](LICENSE) 发布。
