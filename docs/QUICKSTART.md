# ptu Quick Start Guide

**创建日期**: 2026-01-07
**类型**: 快速入门
**状态**: 已完成
**相关**: [README.md](../README.md) | [开发指南](./DEVELOPMENT_GUIDE.md)

## 快速开始（5分钟）

### 前置要求

- **Rust** 版本 1.70 或更高
- **macOS** (arm64/aarch64) 或 **Linux** (x86_64)

### 1. 编译项目

```bash
# 克隆或进入项目目录
cd /path/to/ptu

# 编译发布版本
cargo build --release

# 编译后的二进制文件
# macOS/Linux: target/release/ptu
```

### 2. 验证安装

```bash
# 查看帮助信息
./target/release/ptu --help

# 查看版本
./target/release/ptu --version
```

### 3. 基本使用

#### 列出所有监听端口

```bash
# 列出所有监听的 TCP 和 UDP 端口
./target/release/ptu list

# 仅显示监听端口
./target/release/ptu list --listen

# 仅显示 TCP
./target/release/ptu list --tcp

# 仅显示 UDP
./target/release/ptu list --udp
```

#### 查询特定端口

```bash
# 查询端口 8080
./target/release/ptu get port 8080

# 输出示例:
# Sockets using port 8080:
#
# Protocol: TCP
# Local Address: 0.0.0.0:8080
# State: LISTEN
# PID: 5432
# Process: node
```

#### 查询特定进程

```bash
# 查询进程 1234 的所有端口
./target/release/ptu get pid 1234
```

#### JSON 输出

```bash
# JSON 格式输出
./target/release/ptu --json list

# JSON 格式查询端口
./target/release/ptu -j get port 8080
```

### 4. 安装到系统路径

```bash
# 方式1: 使用 cargo 安装
cargo install --path .

# 方式2: 手动复制
sudo cp target/release/ptu /usr/local/bin/

# 验证安装
ptu --help
```

## 常见使用场景

### 场景1: 查看所有 Web 服务器端口

```bash
# 查看 HTTP (80) 和 HTTPS (443)
ptu list --port 80
ptu list --port 443

# 查看所有监听的 TCP 端口
ptu list --listen --tcp
```

### 场景2: 调试端口占用

```bash
# 检查 8080 端口被哪个进程占用
ptu get port 8080

# 查看 Node.js 进程的所有端口
ptu list --pid $(pgrep node)
```

### 场景3: 系统安全审计

```bash
# 列出所有监听端口（需要 root）
sudo ptu list --listen

# 查看特定进程的网络连接
sudo ptu get pid 1
```

### 场景4: 自动化脚本

```bash
# 在脚本中使用 JSON 输出
#!/bin/bash
PORTS=$(ptu -j list | jq -r '.[].local_address.port')
echo "Open ports: $PORTS"

# 检查端口是否被占用
if ptu -j get port 8080 | jq -e '. | length > 0' > /dev/null; then
    echo "Port 8080 is in use"
fi
```

## 下一步

- 阅读完整文档: [README.md](../README.md)
- 了解开发细节: [开发指南](./DEVELOPMENT_GUIDE.md)
- 查看示例: 下方示例代码

## 示例代码

### 作为 Rust 库使用

```rust
use ptu::{get_sockets, SocketFilter};

fn main() -> anyhow::Result<()> {
    // 获取所有监听的 TCP 端口
    let filter = SocketFilter {
        listen_only: true,
        ..Default::default()
    };

    let sockets = get_sockets(&filter)?;

    for socket in sockets {
        println!(
            "{}:{} - {} ({})",
            socket.local_address.ip,
            socket.local_address.port,
            socket.process_name.unwrap_or("unknown".into()),
            socket.pid.unwrap_or(0)
        );
    }

    Ok(())
}
```

### 过滤特定协议

```rust
use ptu::{get_sockets, SocketFilter, Protocol};

fn main() -> anyhow::Result<()> {
    // 仅查询 TCP
    let filter = SocketFilter {
        protocols: Some(vec![Protocol::Tcp, Protocol::Tcp6]),
        listen_only: true,
        ..Default::default()
    };

    let sockets = get_sockets(&filter)?;
    println!("Found {} TCP sockets", sockets.len());

    Ok(())
}
```

### 获取进程信息

```rust
use ptu::platform::get_process_info;

fn main() -> anyhow::Result<()> {
    let pid = 1234;
    let process = get_process_info(pid)?;

    println!("Process: {}", process.name);
    println!("PID: {}", process.pid);
    println!("Command: {}", process.command_line.unwrap_or("unknown".into()));
    println!("Open sockets: {}", process.sockets.len());

    Ok(())
}
```

## 故障排除

### 问题1: 权限不足

**错误**:
```
Error: Permission denied
```

**解决**:
```bash
# 使用 sudo
sudo ptu list
```

### 问题2: 某些端口不显示进程信息

**原因**: 某些进程需要更高权限

**解决**:
```bash
# 使用 sudo 运行
sudo ptu list
```

### 问题3: macOS 上需要完全磁盘访问权限

**错误**: 无法访问某些进程信息

**解决**:
1. 打开"系统偏好设置" > "安全性与隐私" > "隐私"
2. 选择"完全磁盘访问权限"
3. 添加你的终端应用（Terminal, iTerm2 等）

### 问题4: 编译错误

**错误**: `error: linker not found`

**解决**:
```bash
# macOS: 安装 Xcode 命令行工具
xcode-select --install

# Ubuntu/Debian:
sudo apt-get install build-essential

# Fedora/RHEL:
sudo dnf install gcc
```

## 性能提示

1. **使用过滤器**: 只查询需要的信息
   ```bash
   # 好: 只查询 TCP
   ptu list --tcp

   # 慢: 查询所有后过滤
   ptu list | grep TCP
   ```

2. **使用 JSON 输出**: 在脚本中更高效
   ```bash
   # 好: 直接 JSON
   ptu -j get port 8080 | jq '.[0].pid'

   # 慢: 解析文本输出
   ptu get port 8080 | grep "PID:" | awk '{print $2}'
   ```

3. **限制查询范围**
   ```bash
   # 好: 只查询特定端口
   ptu get port 8080

   # 慢: 列出所有后查找
   ptu list | grep 8080
   ```

## 进阶技巧

### 组合选项

```bash
# 查看监听的 TCP IPv4 端口
ptu list -l -t -4

# 查看特定进程的端口
ptu list -p $(pgrep nginx)

# 查看端口 80-443 范围内的监听端口
for port in {80..443}; do
    ptu get port $port 2>/dev/null | grep -q "LISTEN" && echo "Port $port is listening"
done
```

### 与其他工具组合

```bash
# 与 lsof 对比
echo "=== ptu output ==="
ptu list --listen --tcp
echo ""
echo "=== lsof output ==="
lsof -i -P -n | grep LISTEN

# 找出占用端口最多的进程
for pid in $(ptu -j list | jq -r '.[].pid' | sort -u); do
    count=$(ptu -j list | jq --arg pid $pid '[.[] | select(.pid == ($pid | tonumber))] | length')
    echo "PID $pid: $count sockets"
done | sort -t: -k2 -rn | head -5
```

## 参考资源

- [完整文档](../README.md)
- [开发指南](./DEVELOPMENT_GUIDE.md)
- [Rust 官方文档](https://doc.rust-lang.org/)
- [clap 文档](https://docs.rs/clap/)
