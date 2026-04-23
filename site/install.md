# 安装指南

选择适合您平台的安装方式。

## 系统要求

- **操作系统**: Linux, macOS, Windows
- **Rust**: 1.70+ (如果从源码构建)
- **内存**: 至少 8GB RAM (推荐 16GB+)
- **磁盘空间**: 根据参考基因组大小，索引通常需要 5-10GB

## 方式一：Cargo 安装 (推荐)

如果您已经安装了 Rust 工具链，最简单的方式是通过 cargo 安装：

```bash
cargo install bwa-rust
```

安装完成后，验证安装：

```bash
bwa-rust --version
```

## 方式二：预编译二进制

从 [GitHub Releases](https://github.com/LessUp/bwa-rust/releases) 下载对应平台的预编译二进制文件：

### Linux

```bash
# x86_64
wget https://github.com/LessUp/bwa-rust/releases/latest/download/bwa-rust-linux-x64.tar.gz
tar -xzf bwa-rust-linux-x64.tar.gz
sudo mv bwa-rust /usr/local/bin/

# 验证
bwa-rust --version
```

### macOS

```bash
# Intel
wget https://github.com/LessUp/bwa-rust/releases/latest/download/bwa-rust-macos-x64.tar.gz
tar -xzf bwa-rust-macos-x64.tar.gz
sudo mv bwa-rust /usr/local/bin/

# Apple Silicon (M1/M2)
wget https://github.com/LessUp/bwa-rust/releases/latest/download/bwa-rust-macos-arm64.tar.gz
tar -xzf bwa-rust-macos-arm64.tar.gz
sudo mv bwa-rust /usr/local/bin/
```

### Windows

从 Releases 下载 `bwa-rust-windows-x64.zip`，解压后将 `bwa-rust.exe` 添加到 PATH。

## 方式三：从源码构建

```bash
# 克隆仓库
git clone https://github.com/LessUp/bwa-rust.git
cd bwa-rust

# 构建 Release 版本
cargo build --release

# 二进制位于 target/release/bwa-rust
./target/release/bwa-rust --version
```

## 方式四：Docker

```bash
# 拉取镜像
docker pull ghcr.io/lessup/bwa-rust:latest

# 运行
docker run --rm ghcr.io/lessup/bwa-rust:latest --version
```

## 下一步

安装完成后，查看 [快速开始指南](/guide/quickstart) 了解如何使用 bwa-rust 进行序列比对。
