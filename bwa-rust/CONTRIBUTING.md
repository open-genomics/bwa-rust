# 贡献指南

感谢你对 bwa-rust 项目的关注！欢迎提交 Issue、Pull Request 或任何形式的反馈。

## 如何贡献

### 报告 Bug

1. 在 [Issues](https://github.com/LessUp/bwa-rust/issues) 页面搜索是否已有相同问题
2. 如果没有，请创建新 Issue，包含：
   - 问题描述
   - 复现步骤
   - 期望行为 vs 实际行为
   - 环境信息（OS、Rust 版本等）

### 提交代码

1. Fork 本仓库
2. 创建特性分支：`git checkout -b feature/your-feature`
3. 提交更改：`git commit -m "feat: add your feature"`
4. 推送分支：`git push origin feature/your-feature`
5. 创建 Pull Request

### 开发流程

```bash
# 克隆项目
git clone https://github.com/LessUp/bwa-rust.git
cd bwa-rust/bwa-rust

# 构建
cargo build

# 运行测试
cargo test

# 检查代码格式
cargo fmt -- --check

# 运行 lint
cargo clippy -- -D warnings

# 运行基准测试
cargo bench
```

### 代码规范

- 使用 `cargo fmt` 格式化代码
- 确保 `cargo clippy -- -D warnings` 无警告
- 新功能需附带单元测试
- 提交信息建议遵循 [Conventional Commits](https://www.conventionalcommits.org/) 规范：
  - `feat:` 新功能
  - `fix:` Bug 修复
  - `docs:` 文档变更
  - `refactor:` 代码重构
  - `test:` 测试相关
  - `perf:` 性能优化
  - `ci:` CI/CD 相关

### 项目结构

```
src/
├── main.rs          # CLI 入口
├── lib.rs           # Library 入口
├── io/              # FASTA/FASTQ 解析
├── index/           # FM 索引（SA、BWT、FM）
├── align/           # 对齐算法（SMEM、Chain、SW）
└── util/            # 工具函数（DNA 编码）
```

## 许可证

参与贡献即表示你同意你的贡献将以 [MIT 许可证](LICENSE) 发布。
