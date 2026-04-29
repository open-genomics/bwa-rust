# 安装

## 下载 Release

从 GitHub Releases 下载对应平台产物。

| 平台 | 文件 |
|------|------|
| Linux 静态链接 | `bwa-rust-linux-amd64-static.tar.gz` |
| Linux glibc | `bwa-rust-linux-amd64.tar.gz` |
| macOS Intel | `bwa-rust-macos-amd64.tar.gz` |
| macOS Apple Silicon | `bwa-rust-macos-arm64.tar.gz` |
| Windows | `bwa-rust-windows-amd64.zip` |

Linux 示例：

```bash
curl -sL https://github.com/LessUp/bwa-rust/releases/latest/download/bwa-rust-linux-amd64-static.tar.gz | tar xz
./bwa-rust --version
```

## 从源码构建

```bash
git clone https://github.com/LessUp/bwa-rust.git
cd bwa-rust
cargo build --release
target/release/bwa-rust --version
```

## 不支持的安装方式

当前没有发布 Docker 镜像，也没有声明 Homebrew、Conda 或 apt 仓库。若文档或第三方页面声称支持这些方式，以本页为准。
