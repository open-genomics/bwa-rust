# 2026-02-13 项目完善为完整 GitHub 项目

## 变更内容

### Cargo.toml 元数据完善
- 添加 `authors`、`description`、`license`、`repository`、`homepage`
- 添加 `readme`、`keywords`、`categories`、`rust-version`

### 新增项目文件
- **LICENSE** — MIT 许可证
- **CONTRIBUTING.md** — 贡献指南（开发流程、代码规范、项目结构）
- **CODE_OF_CONDUCT.md** — 行为准则（基于 Contributor Covenant）
- **CHANGELOG.md** — 标准化变更日志（整合 changelog/v0.1.0.md）
- **.gitignore** — bwa-rust 目录级别的 gitignore

### GitHub 模板
- **.github/ISSUE_TEMPLATE/bug_report.md** — Bug 报告模板
- **.github/ISSUE_TEMPLATE/feature_request.md** — 功能请求模板
- **.github/pull_request_template.md** — PR 模板
- **.github/workflows/ci.yml** — 独立的 CI 配置（不依赖外层目录）

### 代码改进
- **lib.rs** — 添加完整的 crate 级文档注释（项目说明、快速示例、模块说明）
- **main.rs** — 改为使用 library crate (`use bwa_rust::*`) 而非重复声明 mod，消除双重编译
- **align/seed.rs** — 修复未使用变量警告

### README.md 完善
- 添加 CI、License、Rust 版本 badges
- 添加安装说明（从源码构建、系统要求）
- 添加贡献指引和许可证章节

### 结果
- 编译零警告
- 48 个单元测试 + 1 个文档测试全部通过
