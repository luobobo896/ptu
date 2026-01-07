# ptu Development Guide

**创建日期**: 2026-01-07
**类型**: 开发文档
**状态**: 开发中
**相关**: [README.md](../README.md)

## 项目概述

`ptu` 是一个跨平台的网络端口管理工具，使用 Rust 编写。本文档面向开发者，说明如何参与开发和扩展项目功能。

## 架构设计

### 模块划分

```
ptu/
├── src/
│   ├── main.rs          # CLI 入口点，处理命令分发
│   ├── lib.rs           # 库入口，导出公共 API
│   ├── cli.rs           # 使用 clap 定义 CLI 接口
│   ├── core/            # 平台无关的核心功能
│   │   ├── types.rs     # 数据结构：SocketInfo, ProcessInfo 等
│   │   └── output.rs    # 输出格式化（表格/JSON）
│   ├── commands/        # 命令实现逻辑
│   │   ├── list.rs      # ptu list 命令
│   │   └── get.rs       # ptu get 命令
│   └── platform/        # 平台特定实现
│       ├── mod.rs       # 平台抽象层
│       ├── macos.rs     # macOS 实现（待完善）
│       └── linux.rs     # Linux 实现
└── Cargo.toml           # 项目配置
```

### 设计原则

1. **库与 CLI 分离**: 核心功能在 `src/lib.rs` 中，CLI 入口在 `src/main.rs`
2. **平台抽象**: 通过 `platform` 模块隐藏平台差异
3. **类型安全**: 使用强类型避免运行时错误
4. **错误处理**: 使用 `anyhow` 和 `thiserror` 提供清晰的错误信息

## 待完善功能

### 1. macOS 平台实现（优先级：高）

`src/platform/macos.rs` 中的以下函数需要完善：

#### get_tcp_table()

**目标**: 使用 `sysctl` 获取 macOS 内核的 TCP 表

**参考实现思路**:
```rust
// macOS sysctl MIB 数组
let mib: [c_int; 4] = [
    libc::CTL_NET,
    libc::PF_INET,
    libc::IPPROTO_TCP,
    libc::TCPCTL_TABLE,
];

// 获取缓冲区大小
let mut len = 0;
if unsafe { sysctl(mib.as_ptr(), 4, ptr::null_mut(), &mut len, ptr::null(), 0) } != 0 {
    return Err(anyhow!("Failed to get TCP table size"));
}

// 分配缓冲区并获取数据
let mut buf = vec![0u8; len];
if unsafe { sysctl(mib.as_ptr(), 4, buf.as_mut_ptr() as *mut c_void, &mut len, ptr::null(), 0) } != 0 {
    return Err(anyhow!("Failed to get TCP table"));
}

// 解析 xinpgen 结构和 xbtcpcb 结构
// 需要参考 macOS 内核头文件
```

**相关资源**:
- macOS 内核头文件: `/usr/include/sys/socket.h`
- TCP 表定义: `/usr/include/netinet/tcp_var.h`
- 示例代码: `netstat` 源代码

#### get_udp_table()

**目标**: 类似 `get_tcp_table()`，获取 UDP 表

**实现思路**: 与 TCP 类似，但使用 `IPPROTO_UDP` 和 `UDPCTL_TABLE`

#### get_process_info()

**目标**: 使用 `libproc` 获取进程信息

**当前问题**: `proc_pidinfo` 调用返回的命令行不完整

**改进方案**:
```rust
// 使用 proc_pidinfo 获取完整命令行
extern "C" {
    fn proc_pidinfo(pid: i32, flavor: u32, arg: u64, buffer: *mut c_void, buffersize: i32) -> i32;
}

// PROC_PIDPATHINFO_MAXSIZE 定义
const PROC_PIDPATHINFO_MAXSIZE: usize = 4096;

// 获取可执行文件路径
let mut path = vec![0u8; PROC_PIDPATHINFO_MAXSIZE];
let ret = unsafe {
    proc_pidinfo(
        pid as i32,
        libc::PROC_PIDPATHINFO,
        0,
        path.as_mut_ptr() as *mut libc::c_void,
        path.len() as i32,
    )
};
```

### 2. Linux 平台增强（优先级：中）

`src/platform/linux.rs` 可以增强：

1. **性能优化**:
   - 缓存 `/proc` 文件读取结果
   - 使用 `mio` 或 `tokio` 实现异步扫描

2. **功能扩展**:
   - 支持读取 `/proc/<pid>/fd` 获取更准确的 inode 映射
   - 支持 `/proc/<pid>/net/tcp` 等特定进程的网络信息

3. **错误处理**:
   - 更优雅地处理权限不足的情况
   - 提供更清晰的错误消息

### 3. 新功能（优先级：中）

#### 3.1 持续监控模式

```bash
# 每秒刷新显示打开的端口
ptu list --watch
```

**实现思路**:
- 使用 `tokio` 定时器
- 使用终端库（如 `crossterm`）清屏刷新

#### 3.2 统计信息

```bash
# 显示端口统计
ptu stats

# 输出示例
# Total listening ports: 15
# TCP: 12
# UDP: 3
# Unique processes: 8
```

**实现思路**:
- 添加新命令 `Commands::Stats`
- 聚合和分析 `SocketInfo` 列表

#### 3.3 杀掉占用端口的进程

```bash
# 杀掉占用 8080 端口的进程
ptu kill port 8080

# 发送 SIGTERM 而不是 SIGKILL
ptu kill port 8080 --signal SIGTERM
```

**安全考虑**:
- 需要用户确认
- 需要权限检查
- 默认使用 SIGTERM，显式指定才用 SIGKILL

#### 3.4 导出功能

```bash
# 导出为 CSV
ptu list --output csv > ports.csv

# 导出为 JSON
ptu list --output json > ports.json
```

**实现思路**:
- 添加 `--output` 选项
- 支持多种格式（csv, json, table）

### 4. 用户体验改进（优先级：低）

#### 4.1 进度条

```bash
# 扫描时显示进度
ptu list --progress

# 输出: [████████░░] 80% (800/1000 sockets scanned)
```

**实现思路**:
- 使用 `indicatif` crate
- 在扫描过程中更新进度条

#### 4.2 颜色输出

```bash
# 启用颜色输出
ptu list --color=always

# 高亮重要端口（22, 80, 443）
# 不同状态使用不同颜色
```

**实现思路**:
- 使用 `colored` crate
- 根据 `TERM` 环境变量自动检测

#### 4.3 自动补全

**目标**: 支持 Bash/Zsh/Fish 自动补全

**实现思路**:
- 使用 `clap_complete` 生成补全脚本
- 在 Cargo.toml 中添加：
  ```toml
  [dependencies]
  clap = { version = "4.5", features = ["derive", "cargo"] }
  clap_complete = "4.5"

  # 构建补全脚本
  cargo run --bin generate-completions
  ```

## 测试策略

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_socket_address_ipv4() {
        let addr = "0100007F:1F90"; // 127.0.0.1:8080
        let result = parse_socket_address(addr);
        assert!(result.is_some());
        assert_eq!(result.unwrap().port, 8080);
    }
}
```

### 集成测试

在 `tests/` 目录下创建集成测试：

```rust
// tests/integration_test.rs
use ptu::get_sockets;
use ptu::SocketFilter;

#[test]
fn test_list_all_sockets() {
    let filter = SocketFilter::default();
    let sockets = get_sockets(&filter);
    assert!(sockets.is_ok());
}
```

### 性能测试

```rust
// benches/benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_get_sockets(c: &mut Criterion) {
    c.bench_function("get_sockets", |b| {
        b.iter(|| {
            get_sockets(&SocketFilter::default())
        })
    });
}

criterion_group!(benches, benchmark_get_sockets);
criterion_main!(benches);
```

## 代码质量

### Clippy 检查

```bash
# 运行 clippy
cargo clippy --all-targets --all-features

# 自动修复
cargo clippy --fix --allow-dirty
```

### 格式化

```bash
# 格式化代码
cargo fmt

# 检查格式
cargo fmt --check
```

### 文档

确保公共 API 有文档注释：

```rust
/// Get all socket information for the current platform
///
/// # Arguments
///
/// * `filter` - Filter options for limiting results
///
/// # Returns
///
/// A vector of `SocketInfo` structs
///
/// # Errors
///
/// Returns an error if:
/// - Permission is denied
/// - System calls fail
///
/// # Examples
///
/// ```no_run
/// use ptu::{get_sockets, SocketFilter};
///
/// let sockets = get_sockets(&SocketFilter::default())?;
/// ```
pub fn get_sockets(filter: &SocketFilter) -> Result<Vec<SocketInfo>> {
    // ...
}
```

## 发布流程

### 版本号

遵循语义化版本（Semantic Versioning）：
- **MAJOR**: 不兼容的 API 变更
- **MINOR**: 向后兼容的功能新增
- **PATCH**: 向后兼容的问题修复

### 发布清单

1. 更新 `Cargo.toml` 中的版本号
2. 更新 `CHANGELOG.md`
3. 更新 `README.md` 中的示例
4. 运行完整测试套件
5. 构建 release 二进制文件
6. 创建 Git tag
7. 发布到 crates.io（可选）
8. 创建 GitHub Release

### 跨平台构建

```bash
# macOS (ARM64)
cargo build --release --target aarch64-apple-darwin

# Linux (x86_64)
cargo build --release --target x86_64-unknown-linux-gnu

# 静态链接 Linux 版本
RUSTFLAGS="-C target-feature=+crt-static" \
  cargo build --release --target x86_64-unknown-linux-gnu
```

## 贡献流程

1. Fork 项目
2. 创建特性分支: `git checkout -b feature/amazing-feature`
3. 提交更改: `git commit -m 'feat: add amazing feature'`
4. 推送分支: `git push origin feature/amazing-feature`
5. 创建 Pull Request

### 提交消息格式

遵循约定式提交（Conventional Commits）：
```
feat: add TCP socket monitoring
fix: resolve panic when parsing invalid addresses
docs: update installation instructions
refactor: simplify platform abstraction
perf: optimize /proc filesystem reading
test: add unit tests for address parsing
```

## 资源链接

### Rust 生态
- [Rust 官方文档](https://doc.rust-lang.org/)
- [Cargo 手册](https://doc.rust-lang.org/cargo/)
- [API 指南](https://doc.rust-lang.org/std/index.html)

### 相关项目
- [netstat.rs](https://github.com/containers/youki) - 纯 Rust 实现
- [ss (iproute2)](https://wiki.linuxfoundation.org/networking/iproute2)
- [lsof](https://github.com/lsof-org/lsof)

### 平台特定文档
- [macOS sysctl](https://man7.org/linux/man-pages/man3/sysctl.3.html)
- [Linux /proc filesystem](https://man7.org/linux/man-pages/man5/proc.5.html)

## 许可证

本项目采用 MIT OR Apache-2.0 双重许可证。贡献的代码将使用相同的许可证。
