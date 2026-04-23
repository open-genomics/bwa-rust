# 安装

## 系统要求

| 要求 | 版本 | 说明 |
|------|------|------|
| Rust | 1.70+ | 最低支持版本 (MSRV) |
| 平台 | Linux / macOS / Windows | 主流操作系统 |
| 内存 | 8GB+ | 人类基因组索引构建建议 |

## 安装方式

### 方式一：从源码构建（推荐）

```bash
# 克隆仓库
git clone https://github.com/LessUp/bwa-rust.git
cd bwa-rust

# 构建 Release 版本
cargo build --release
```

编译后的二进制文件位于 `target/release/bwa-rust`。

验证安装：

```bash
./target/release/bwa-rust --version
```

### 方式二：Cargo 安装

```bash
cargo install bwa-rust
```

安装后确保 `~/.cargo/bin` 在 PATH 中：

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

### 方式三：预编译二进制

从 [GitHub Releases](https://github.com/LessUp/bwa-rust/releases) 下载对应平台的二进制文件：

| 平台 | 文件名 |
|------|--------|
| Linux (x86_64) | `bwa-rust-linux-amd64.tar.gz` |
| macOS (Intel) | `bwa-rust-macos-amd64.tar.gz` |
| macOS (Apple Silicon) | `bwa-rust-macos-arm64.tar.gz` |
| Windows | `bwa-rust-windows-amd64.zip` |

解压后即可使用。

## 开发环境

如需修改源码或参与开发：

```bash
# 开发构建
cargo build

# 运行测试
cargo test

# 运行示例
cargo run --example simple_align
```

## 常见问题

### 编译失败

确保 Rust 版本 >= 1.70：

```bash
rustc --version
```

如需更新：

```bash
rustup update
```

### 找不到 `bwa-rust` 命令

Cargo 安装的二进制在 `~/.cargo/bin`，确保该目录在 PATH 中，或直接使用完整路径：

```bash
~/.cargo/bin/bwa-rust --version
```
