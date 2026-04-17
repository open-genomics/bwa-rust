# Support

感谢你对 bwa-rust 的关注！以下是获取帮助的渠道。

## 📖 文档资源

| 资源 | 描述 | 链接 |
|------|------|------|
| **在线文档** | 完整的使用指南和 API 文档 | [lessup.github.io/bwa-rust](https://lessup.github.io/bwa-rust/) |
| **架构文档** | 技术实现细节 | [docs/architecture/](https://github.com/LessUp/bwa-rust/tree/main/docs/architecture) |
| **教程** | 上手教程 | [docs/tutorial/](https://github.com/LessUp/bwa-rust/tree/main/docs/tutorial) |
| **API 文档** | Rust docs | `cargo doc --open` |
| **规范文档** | 开发规范 (SDD) | [specs/](https://github.com/LessUp/bwa-rust/tree/main/specs) |

## 💬 社区支持

### GitHub Discussions
- **提问**: [New Discussion](https://github.com/LessUp/bwa-rust/discussions/new?category=q-a)
- **功能讨论**: [Ideas](https://github.com/LessUp/bwa-rust/discussions/categories/ideas)

### 报告问题

发现 Bug？请使用 [Bug Report 模板](https://github.com/LessUp/bwa-rust/issues/new?template=bug_report.md) 创建 Issue。

请求新功能？请使用 [Feature Request 模板](https://github.com/LessUp/bwa-rust/issues/new?template=feature_request.md)。

## 🔒 安全报告

如果你发现了安全漏洞，**请不要**公开创建 Issue。

请通过 [Security Advisories](https://github.com/LessUp/bwa-rust/security/advisories) 或邮件私密报告。

详见 [SECURITY.md](SECURITY.md)。

## 🛠️ 故障排查

### 常见问题

<details>
<summary><b>构建失败：找不到 jemalloc</b></summary>

在 Windows 上，jemalloc 不会启用。在 Linux/macOS 上，确保安装了 build-essential/ Xcode。

</details>

<details>
<summary><b>FASTA 文件解析错误</b></summary>

- 检查 FASTA 文件格式是否正确
- 确保序列名称不为空
- 检查是否有重复的 contig 名称

</details>

<details>
<summary><b>内存不足 (OOM)</b></summary>

对于大型基因组，使用内存保护参数：
```bash
bwa-rust mem ref.fa reads.fq --max-occ 200 --max-chains 3 --max-alignments 3
```

</details>

## 📋 贡献

想要贡献代码？请阅读 [CONTRIBUTING.md](../CONTRIBUTING.md)。

## ⏱️ 响应时间

| 类型 | 预期响应时间 |
|------|-------------|
| 安全问题 | 48 小时内 |
| Bug 报告 | 7 天内 |
| 功能请求 | 14 天内 |
| 一般问题 | 取决于社区 |

---

**注意**：这是一个开源项目，维护者在业余时间维护。请保持友善和耐心。
