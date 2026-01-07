# ptu - Port Management Utility

**创建日期**: 2026-01-07
**类型**: 项目文档
**状态**: v0.1.0 (Linux/macOS 生产就绪)
**相关**: [已知限制](docs/KNOWN_LIMITATIONS.md)

## 概述

`ptu` 是一个用 Rust 编写的跨平台网络端口管理工具，旨在提供类似 `ss`、`lsof` 和 `netstat` 的功能，但采用现代化的设计和更简洁的用户体验。

## 平台支持状态

| 平台 | 状态 | 说明 |
|------|------|------|
| **Linux** | ✅ 完全支持 | 生产就绪，所有功能可用 |
| **macOS** | ✅ 完全支持 | 生产就绪，所有功能可用（使用 lsof） |
| **Windows** | ❌ 不支持 | 计划在 v0.3.0 添加 |

## 特性

- 🚀 **快速高效**: 使用 Rust 编写，性能优异，内存占用小
- 🔍 **功能强大**: 支持查询 TCP/UDP 端口、进程信息、连接状态
- 🖥️ **跨平台**: 支持 Linux 和 macOS
- 📊 **灵活输出**: 支持人类可读的表格格式和机器可读的 JSON 格式
- 🎨 **简洁易用**: 现代化的 CLI 界面，直观的命令结构

## 安装

### 从源码编译

```bash
# 克隆仓库
git clone https://github.com/user/ptu.git
cd ptu

# 编译发布版本
cargo build --release

# 二进制文件位置
# macOS/Linux: target/release/ptu
```

### 安装到系统路径

```bash
# 安装到 ~/.cargo/bin
cargo install --path .

# 或手动复制
sudo cp target/release/ptu /usr/local/bin/
```

### 跨平台编译

#### 为 macOS 编译（在 macOS 上）

```bash
# 当前系统
cargo build --release

# 目标: aarch64-apple-darwin（当前架构）
cargo build --release --target aarch64-apple-darwin
```

#### 为 Linux 编译（在 Linux 上）

```bash
# 目标: x86_64-unknown-linux-gnu
cargo build --release --target x86_64-unknown-linux-gnu
```

#### 为 macOS 编译（在 Linux 上，需要交叉编译工具）

```bash
# 安装交叉编译工具
rustup target add aarch64-apple-darwin

# 编译（需要适当的工具链）
cargo build --release --target aarch64-apple-darwin
```

## 使用方法

### 基本命令

#### 1. 列出所有监听的端口

```bash
# 列出所有监听的 TCP 和 UDP 端口
ptu list

# 仅显示监听端口
ptu list --listen
# 或
ptu list -l
```

**输出示例**:
```
┌──────────┬─────────────────┬────────────────┬────────────┬───────┬──────────────┐
│ PROTOCOL │ LOCAL ADDRESS   │ REMOTE ADDRESS │ STATE      │ PID   │ PROCESS NAME │
├──────────┼─────────────────┼────────────────┼────────────┼───────┼──────────────┤
│ TCP      │ 0.0.0.0:22      │ 0.0.0.0:0      │ LISTEN     │ 1234  │ sshd         │
│ TCP      │ 127.0.0.1:3306  │ 0.0.0.0:0      │ LISTEN     │ 5678  │ mysqld       │
│ UDP      │ 0.0.0.0:53      │ 0.0.0.0:0      │ UNKNOWN    │ 9101  │ named        │
└──────────┴─────────────────┴────────────────┴────────────┴───────┴──────────────┘
```

#### 2. 查询指定端口

```bash
# 查询端口 8080 的详细信息
ptu get port 8080
```

**输出示例**:
```
Sockets using port 8080:

Protocol: TCP
Local Address: 0.0.0.0:8080
Remote Address: 0.0.0.0:0
State: LISTEN
PID: 5432
Process: node
Command: /usr/local/bin/node /path/to/app.js

Protocol: TCP6
Local Address: :::8080
Remote Address: :::0
State: LISTEN
PID: 5432
Process: node
Command: /usr/local/bin/node /path/to/app.js
```

#### 3. 查询指定进程

```bash
# 查询进程 ID 1234 打开的所有端口
ptu get pid 1234
```

**输出示例**:
```
Process nginx (PID: 1234)
Command: /usr/sbin/nginx -c /etc/nginx/nginx.conf

Open Sockets:

┌──────────┬─────────────────┬────────────────┬────────────┬───────┬──────────────┐
│ PROTOCOL │ LOCAL ADDRESS   │ REMOTE ADDRESS │ STATE      │ PID   │ PROCESS NAME │
├──────────┼─────────────────┼────────────────┼────────────┼───────┼──────────────┤
│ TCP      │ 0.0.0.0:80      │ 0.0.0.0:0      │ LISTEN     │ 1234  │ nginx        │
│ TCP      │ 0.0.0.0:443     │ 0.0.0.0:0      │ LISTEN     │ 1234  │ nginx        │
└──────────┴─────────────────┴────────────────┴────────────┴───────┴──────────────┘
```

### 高级选项

#### 过滤协议类型

```bash
# 仅显示 TCP 端口
ptu list --tcp
# 或
ptu list -t

# 仅显示 UDP 端口
ptu list --udp
# 或
ptu list -u

# 同时显示 TCP 和 UDP（默认）
ptu list --tcp --udp
```

#### 过滤协议版本

```bash
# 仅显示 IPv4
ptu list --ipv4
# 或
ptu list -4

# 仅显示 IPv6
ptu list --ipv6
# 或
ptu list -6
```

#### 按进程过滤

```bash
# 显示特定进程的端口
ptu list --pid 1234
# 或
ptu list -p 1234
```

#### 按端口号过滤

```bash
# 显示特定端口
ptu list --port 8080
# 或
ptu list -P 8080
```

### JSON 输出

所有命令都支持 JSON 输出格式，便于脚本处理：

```bash
# JSON 输出（全局选项）
ptu --json list
ptu -j list

# 组合使用
ptu -j get port 8080
ptu --json get pid 1234
```

**JSON 输出示例**:
```json
[
  {
    "protocol": "tcp",
    "local_address": {
      "ip": "0.0.0.0",
      "port": 8080
    },
    "remote_address": {
      "ip": "0.0.0.0",
      "port": 0
    },
    "state": "LISTEN",
    "pid": 5432,
    "process_name": "node",
    "command_line": "/usr/local/bin/node /path/to/app.js"
  }
]
```

### 组合选项

```bash
# 查看所有监听的 TCP IPv4 端口
ptu list -l -t -4

# 查看 PID 1234 的所有 TCP 端口
ptu list -p 1234 -t

# JSON 格式输出端口 8080 的信息
ptu -j get port 8080
```

## 权限要求

某些操作可能需要 root/sudo 权限才能获取完整的进程信息：

```bash
# 使用 sudo 运行以获取所有进程信息
sudo ptu list
sudo ptu get port 80
sudo ptu get pid 1
```

## 架构设计

```
ptu/
├── src/
│   ├── main.rs          # CLI 入口
│   ├── lib.rs           # 库入口
│   ├── cli.rs           # 命令行解析（clap）
│   ├── core/            # 核心功能
│   │   ├── types.rs     # 数据结构定义
│   │   └── output.rs    # 输出格式化
│   ├── commands/        # 命令实现
│   │   ├── list.rs      # list 命令
│   │   └── get.rs       # get 命令
│   └── platform/        # 平台相关代码
│       ├── macos.rs     # macOS 实现
│       └── linux.rs     # Linux 实现
└── Cargo.toml           # 项目配置
```

### 平台支持

- **macOS**: 使用 `sysctl` 和 `libproc` API 获取网络和进程信息
- **Linux**: 解析 `/proc/net/tcp*`、`/proc/net/udp*` 和 `/proc/<pid>/` 文件

## 开发

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行测试并显示输出
cargo test -- --nocapture

# 运行特定测试
cargo test test_parse_socket_address
```

### 代码检查

```bash
# 运行 clippy
cargo clippy

# 自动修复可修复的问题
cargo clippy --fix
```

### 格式化代码

```bash
# 格式化代码
cargo fmt

# 检查格式是否正确
cargo fmt --check
```

## 类似工具对比

| 工具 | 语言 | 输出格式 | 跨平台 | JSON 输出 |
|------|------|---------|--------|-----------|
| **ptu** | Rust | 表格 | ✅ macOS/Linux | ✅ |
| **ss** | C | 表格 | ✅ Linux | ❌ |
| **lsof** | C | 列表 | ✅ macOS/Linux | ❌ |
| **netstat** | C | 表格 | ✅ macOS/Linux | ❌ |

## 常见问题

### Q: 为什么某些端口没有显示进程名称？

A: 这可能是权限问题。某些进程的信息需要 root 权限才能读取。尝试使用 `sudo ptu list`。

### Q: 在 macOS 上运行需要什么权限？

A: 在 macOS 上，你可能需要授予终端/Shell "完全磁盘访问权限"，才能访问所有进程信息。

### Q: ptu 支持哪些平台？

A: 目前支持 macOS (arm64/aarch64) 和 Linux (x86_64)。未来计划支持 Windows。

### Q: 如何以编程方式使用 ptu？

A: 你可以将 ptu 作为库使用：

```rust
use ptu::{get_sockets, SocketFilter};

let filter = SocketFilter {
    listen_only: true,
    ..Default::default()
};

let sockets = get_sockets(&filter)?;
```

## 贡献

欢迎贡献！请随时提交 Issue 或 Pull Request。

## 许可证

MIT OR Apache-2.0

## 作者

Hanson

## 致谢

本项目受以下工具启发：
- [iproute2/ss](https://wiki.linuxfoundation.org/networking/iproute2)
- [lsof](https://github.com/lsof-org/lsof)
- [netstat](https://linux.die.net/man/8/netstat)
