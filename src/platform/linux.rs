//! Linux-specific implementation using /proc filesystem

use crate::core::types::{Protocol, SocketAddress, SocketInfo, SocketState, ProcessInfo, SocketFilter};
use anyhow::{Context, Result};
use std::fs;
use std::io::{BufRead, BufReader};

/// Get all socket information on Linux
pub fn get_sockets(filter: &SocketFilter) -> Result<Vec<SocketInfo>> {
    // Check if /proc/net exists
    if !std::path::Path::new("/proc/net").exists() {
        anyhow::bail!(
            "Cannot access /proc/net filesystem. \
            This tool only works on Linux. \
            You may be running on an unsupported platform."
        );
    }

    let mut sockets = Vec::new();

    // Get TCP sockets
    if filter.protocols.as_ref().map_or(true, |p| p.iter().any(|&x| matches!(x, Protocol::Tcp | Protocol::Tcp6))) {
        if let Ok(tcp_sockets) = parse_proc_net_tcp("/proc/net/tcp", Protocol::Tcp) {
            sockets.extend(tcp_sockets);
        }
        if let Ok(tcp6_sockets) = parse_proc_net_tcp("/proc/net/tcp6", Protocol::Tcp6) {
            sockets.extend(tcp6_sockets);
        }
    }

    // Get UDP sockets
    if filter.protocols.as_ref().map_or(true, |p| p.iter().any(|&x| matches!(x, Protocol::Udp | Protocol::Udp6))) {
        if let Ok(udp_sockets) = parse_proc_net_udp("/proc/net/udp", Protocol::Udp) {
            sockets.extend(udp_sockets);
        }
        if let Ok(udp6_sockets) = parse_proc_net_udp("/proc/net/udp6", Protocol::Udp6) {
            sockets.extend(udp6_sockets);
        }
    }

    // Apply filters
    sockets = apply_filters(sockets, filter);

    Ok(sockets)
}

/// Get process information by PID
pub fn get_process_info(pid: u32) -> Result<ProcessInfo> {
    let proc_path = format!("/proc/{}", pid);

    // Check if /proc/<pid> exists
    if !std::path::Path::new(&proc_path).exists() {
        anyhow::bail!(
            "Process {} does not exist. \
            You may need to check if the PID is correct.",
            pid
        );
    }

    // Get process name
    let name = fs::read_to_string(format!("{}/comm", proc_path))
        .with_context(|| format!("Failed to read /proc/{}/comm", pid))?
        .trim()
        .to_string();

    // Get command line
    let command_line = fs::read_to_string(format!("{}/cmdline", proc_path))
        .ok()
        .map(|cmd| cmd.replace('\0', " "))
        .filter(|s| !s.is_empty());

    // Get all sockets for this process
    let sockets = get_sockets(&SocketFilter {
        pid: Some(pid),
        ..Default::default()
    })?;

    Ok(ProcessInfo {
        pid,
        name,
        command_line,
        sockets,
    })
}

/// Parse /proc/net/tcp or /proc/net/tcp6
fn parse_proc_net_tcp(path: &str, protocol: Protocol) -> Result<Vec<SocketInfo>> {
    let file = fs::File::open(path).with_context(|| format!("Failed to open {}", path))?;
    let reader = BufReader::new(file);
    let mut sockets = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line.with_context(|| format!("Failed to read line from {}", path))?;

        // Skip header
        if index == 0 {
            continue;
        }

        if let Some(socket) = parse_tcp_line(&line, protocol) {
            sockets.push(socket);
        }
    }

    Ok(sockets)
}

/// Parse /proc/net/udp or /proc/net/udp6
fn parse_proc_net_udp(path: &str, protocol: Protocol) -> Result<Vec<SocketInfo>> {
    let file = fs::File::open(path).with_context(|| format!("Failed to open {}", path))?;
    let reader = BufReader::new(file);
    let mut sockets = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line.with_context(|| format!("Failed to read line from {}", path))?;

        // Skip header
        if index == 0 {
            continue;
        }

        if let Some(socket) = parse_udp_line(&line, protocol) {
            sockets.push(socket);
        }
    }

    Ok(sockets)
}

/// Parse a single line from /proc/net/tcp
fn parse_tcp_line(line: &str, protocol: Protocol) -> Option<SocketInfo> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 10 {
        return None;
    }

    let local_address = parse_socket_address(parts[1])?;
    let remote_address = parse_socket_address(parts[2]);
    let state = parse_tcp_state(parts[3].parse().ok()?);
    let inode = parts[9].parse::<u64>().ok()?;

    // Find PID by inode
    let pid = find_pid_by_inode(inode).ok();

    Some(SocketInfo {
        protocol,
        local_address,
        remote_address,
        state,
        pid,
        process_name: None,
        command_line: None,
    })
}

/// Parse a single line from /proc/net/udp
fn parse_udp_line(line: &str, protocol: Protocol) -> Option<SocketInfo> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 10 {
        return None;
    }

    let local_address = parse_socket_address(parts[1])?;
    let remote_address = parse_socket_address(parts[2]);
    let state = SocketState::Unknown; // UDP doesn't have connection states
    let inode = parts[9].parse::<u64>().ok()?;

    // Find PID by inode
    let pid = find_pid_by_inode(inode).ok();

    Some(SocketInfo {
        protocol,
        local_address,
        remote_address,
        state,
        pid,
        process_name: None,
        command_line: None,
    })
}

/// Parse socket address in hex format "IP:PORT"
fn parse_socket_address(addr: &str) -> Option<SocketAddress> {
    let parts: Vec<&str> = addr.split(':').collect();
    if parts.len() != 2 {
        return None;
    }

    let port = u16::from_str_radix(parts[1], 16).ok()?;

    // Parse IP address
    let ip_bytes = parts[0];
    let ip = if ip_bytes.len() == 8 {
        // IPv4
        format!(
            "{}.{}.{}.{}",
            u8::from_str_radix(&ip_bytes[6..8], 16).ok()?,
            u8::from_str_radix(&ip_bytes[4..6], 16).ok()?,
            u8::from_str_radix(&ip_bytes[2..4], 16).ok()?,
            u8::from_str_radix(&ip_bytes[0..2], 16).ok()?
        )
    } else if ip_bytes.len() == 32 {
        // IPv6 (simplified - just show first few segments)
        let mut ip_parts = Vec::new();
        for i in (0..32).step_by(4) {
            ip_parts.push(&ip_bytes[i..i+4]);
        }
        format!(
            "{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}",
            u16::from_str_radix(ip_parts[0], 16).ok()?,
            u16::from_str_radix(ip_parts[1], 16).ok()?,
            u16::from_str_radix(ip_parts[2], 16).ok()?,
            u16::from_str_radix(ip_parts[3], 16).ok()?,
            u16::from_str_radix(ip_parts[4], 16).ok()?,
            u16::from_str_radix(ip_parts[5], 16).ok()?,
            u16::from_str_radix(ip_parts[6], 16).ok()?,
            u16::from_str_radix(ip_parts[7], 16).ok()?
        )
    } else {
        return None;
    };

    Some(SocketAddress { ip, port })
}

/// Parse TCP state from numeric value
fn parse_tcp_state(state: u8) -> SocketState {
    match state {
        1 => SocketState::Established,
        2 => SocketState::SynSent,
        3 => SocketState::SynRecv,
        4 => SocketState::FinWait1,
        5 => SocketState::FinWait2,
        6 => SocketState::TimeWait,
        7 => SocketState::Close,
        8 => SocketState::CloseWait,
        9 => SocketState::LastAck,
        10 => SocketState::Listen,
        11 => SocketState::Closing,
        _ => SocketState::Unknown,
    }
}

/// Find PID by inode number
fn find_pid_by_inode(inode: u64) -> Result<u32> {
    use std::path::Path;

    let proc_path = Path::new("/proc");

    for entry in proc_path.read_dir()?.filter_map(|e| e.ok()) {
        let pid_str = entry.file_name();
        let pid: u32 = pid_str.to_string_lossy().parse().ok()?;

        // Check /proc/<pid>/fd for socket inode
        let fd_path = entry.path().join("fd");
        if fd_path.exists() {
            if let Ok(mut fds) = fd_path.read_dir() {
                for fd_entry in fds.filter_map(|e| e.ok()) {
                    if let Ok(link) = fs::read_link(fd_entry.path()) {
                        let link_str = link.to_string_lossy();
                        if link_str.starts_with("socket:[") && link_str.contains(&format!("{}", inode)) {
                            return Ok(pid);
                        }
                    }
                }
            }
        }
    }

    anyhow::bail!("No PID found for inode {}", inode)
}

/// Apply filters to socket list
fn apply_filters(mut sockets: Vec<SocketInfo>, filter: &SocketFilter) -> Vec<SocketInfo> {
    if filter.listen_only {
        sockets.retain(|s| matches!(s.state, SocketState::Listen));
    }

    if let Some(pid) = filter.pid {
        sockets.retain(|s| s.pid.map_or(false, |p| p == pid));
    }

    if let Some(port) = filter.port {
        sockets.retain(|s| {
            s.local_address.port == port
                || s.remote_address
                    .as_ref()
                    .map(|a| a.port == port)
                    .unwrap_or(false)
        });
    }

    sockets
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_socket_address_ipv4() {
        // Test IPv4 address parsing: 127.0.0.1:8080
        let addr = "0100007F:1F90";
        let result = parse_socket_address(addr);
        assert!(result.is_some());
        let socket_addr = result.unwrap();
        assert_eq!(socket_addr.ip, "127.0.0.1");
        assert_eq!(socket_addr.port, 8080);
    }

    #[test]
    fn test_parse_socket_address_ipv4_wildcard() {
        // Test 0.0.0.0:80
        let addr = "00000000:0050";
        let result = parse_socket_address(addr);
        assert!(result.is_some());
        let socket_addr = result.unwrap();
        assert_eq!(socket_addr.ip, "0.0.0.0");
        assert_eq!(socket_addr.port, 80);
    }

    #[test]
    fn test_parse_socket_address_invalid() {
        // Test invalid format
        let addr = "invalid";
        let result = parse_socket_address(addr);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_tcp_state() {
        assert_eq!(parse_tcp_state(1), SocketState::Established);
        assert_eq!(parse_tcp_state(10), SocketState::Listen);
        assert_eq!(parse_tcp_state(99), SocketState::Unknown);
    }

    #[test]
    fn test_apply_filters_listen_only() {
        let sockets = vec![
            SocketInfo {
                protocol: Protocol::Tcp,
                local_address: SocketAddress {
                    ip: "0.0.0.0".to_string(),
                    port: 80,
                },
                remote_address: None,
                state: SocketState::Listen,
                pid: Some(1),
                process_name: Some("test".to_string()),
                command_line: None,
            },
            SocketInfo {
                protocol: Protocol::Tcp,
                local_address: SocketAddress {
                    ip: "192.168.1.1".to_string(),
                    port: 45678,
                },
                remote_address: Some(SocketAddress {
                    ip: "10.0.0.1".to_string(),
                    port: 80,
                }),
                state: SocketState::Established,
                pid: Some(1),
                process_name: Some("test".to_string()),
                command_line: None,
            },
        ];

        let filter = SocketFilter {
            listen_only: true,
            ..Default::default()
        };

        let result = apply_filters(sockets, &filter);
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0].state, SocketState::Listen));
    }

    #[test]
    fn test_apply_filters_by_port() {
        let sockets = vec![
            SocketInfo {
                protocol: Protocol::Tcp,
                local_address: SocketAddress {
                    ip: "0.0.0.0".to_string(),
                    port: 80,
                },
                remote_address: None,
                state: SocketState::Listen,
                pid: Some(1),
                process_name: Some("test".to_string()),
                command_line: None,
            },
            SocketInfo {
                protocol: Protocol::Tcp,
                local_address: SocketAddress {
                    ip: "0.0.0.0".to_string(),
                    port: 443,
                },
                remote_address: None,
                state: SocketState::Listen,
                pid: Some(1),
                process_name: Some("test".to_string()),
                command_line: None,
            },
        ];

        let filter = SocketFilter {
            port: Some(80),
            ..Default::default()
        };

        let result = apply_filters(sockets, &filter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].local_address.port, 80);
    }
}
