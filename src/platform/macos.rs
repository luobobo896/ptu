//! macOS-specific implementation using lsof command

use crate::core::types::{Protocol, SocketAddress, SocketInfo, SocketState, ProcessInfo, SocketFilter};
use anyhow::{Context, Result};
use std::process::Command;

/// Get all socket information on macOS
pub fn get_sockets(filter: &SocketFilter) -> Result<Vec<SocketInfo>> {
    let mut sockets = Vec::new();

    // Get TCP sockets
    if filter.protocols.as_ref().is_none_or(|p| p.iter().any(|&x| matches!(x, Protocol::Tcp | Protocol::Tcp6))) {
        sockets.extend(get_tcp_sockets(filter)?);
    }

    // Get UDP sockets
    if filter.protocols.as_ref().is_none_or(|p| p.iter().any(|&x| matches!(x, Protocol::Udp | Protocol::Udp6))) {
        sockets.extend(get_udp_sockets(filter)?);
    }

    Ok(sockets)
}

/// Get process information by PID
pub fn get_process_info(pid: u32) -> Result<ProcessInfo> {
    let name = unsafe {
        let mut name_buf = vec![0u8; 256];
        let ret = libc::proc_name(
            pid as i32,
            name_buf.as_mut_ptr() as *mut libc::c_void,
            name_buf.len() as u32,
        );
        if ret == 0 {
            format!("process_{}", pid)
        } else {
            let name = std::ffi::CStr::from_ptr(name_buf.as_ptr() as *const i8)
                .to_string_lossy()
                .to_string();
            name
        }
    };

    // Get command line using ps command
    let command_line = Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "command="])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|cmd| cmd.trim().to_string())
        .filter(|s| !s.is_empty());

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

/// Get TCP sockets on macOS
fn get_tcp_sockets(filter: &SocketFilter) -> Result<Vec<SocketInfo>> {
    // Use lsof without -iTCP to get all internet sockets with protocol info
    let output = Command::new("lsof")
        .args([
            "-i",          // display internet sockets
            "-P",          // don't convert port names
            "-n",          // don't convert host names
            "-w",          // wide output
            "-a",          // all sockets
            "-iTCP",       // TCP sockets (this adds "TCP" prefix)
        ])
        .output()
        .context("Failed to execute lsof command. Please ensure lsof is installed.")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("lsof command failed: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Pass Protocol::Tcp to parser since we know all results are TCP
    parse_lsof_output(&stdout, filter, Protocol::Tcp)
}

/// Get UDP sockets on macOS
fn get_udp_sockets(filter: &SocketFilter) -> Result<Vec<SocketInfo>> {
    let output = Command::new("lsof")
        .args([
            "-i",          // display internet sockets
            "-P",          // don't convert port names
            "-n",          // don't convert host names
            "-w",          // wide output
            "-a",          // all sockets
            "-iUDP",       // UDP sockets
        ])
        .output()
        .context("Failed to execute lsof command. Please ensure lsof is installed.")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("lsof command failed: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Pass Protocol::Udp to parser since we know all results are UDP
    parse_lsof_output(&stdout, filter, Protocol::Udp)
}

/// Parse lsof output and extract socket information
fn parse_lsof_output(output: &str, filter: &SocketFilter, base_protocol: Protocol) -> Result<Vec<SocketInfo>> {
    let mut sockets = Vec::new();

    for line in output.lines() {
        // Skip header and empty lines
        if line.is_empty() || line.starts_with("COMMAND") {
            continue;
        }

        if let Some(socket) = parse_lsof_line(line, filter, base_protocol) {
            sockets.push(socket);
        }
    }

    Ok(sockets)
}

/// Parse a single line of lsof output
/// Format: COMMAND   PID   USER   FD   TYPE   DEVICE SIZE/OFF NODE NAME
/// Example: node    1234  user   18u  IPv4  0x1234      0t0  TCP *:8080 (LISTEN)
/// Note: Number of columns may vary, NAME is always the last column
fn parse_lsof_line(line: &str, filter: &SocketFilter, base_protocol: Protocol) -> Option<SocketInfo> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 9 {
        return None;
    }

    let command = parts[0].to_string();
    let pid: u32 = parts[1].parse().ok()?;

    // Find NAME field (always last, but position may vary)
    // Also find TYPE field (usually at index 4, but may vary)
    let type_field = parts[4]; // TYPE field

    // NAME field starts at index 8 (after NODE) but may contain spaces
    // Format: COMMAND PID USER FD TYPE DEVICE SIZE/OFF NODE NAME
    let name_field = if parts.len() > 9 {
        parts[8..].join(" ")
    } else {
        parts[parts.len() - 1].to_string()
    };

    // Determine if IPv6 or IPv4 from TYPE field, and combine with base protocol
    let protocol = if type_field.contains("IPv6") {
        match base_protocol {
            Protocol::Tcp => Protocol::Tcp6,
            Protocol::Udp => Protocol::Udp6,
            _ => base_protocol,
        }
    } else {
        match base_protocol {
            Protocol::Tcp6 => Protocol::Tcp,
            Protocol::Udp6 => Protocol::Udp,
            _ => base_protocol,
        }
    };

    let is_tcp = matches!(protocol, Protocol::Tcp | Protocol::Tcp6);

    // Apply protocol filter
    if let Some(ref protocols) = filter.protocols {
        if !protocols.contains(&protocol) {
            return None;
        }
    }

    // Parse addresses from name_field
    // Format: *:8080 (LISTEN) or 192.168.1.1:22->10.0.0.1:54321 (ESTABLISHED)
    let (local_address, remote_address, state) = parse_lsof_addresses(&name_field, is_tcp)?;

    // Apply port filter
    if let Some(port) = filter.port {
        if local_address.port != port
            && remote_address.as_ref().map(|a| a.port != port).unwrap_or(true)
        {
            return None;
        }
    }

    // Apply PID filter
    if let Some(filter_pid) = filter.pid {
        if pid != filter_pid {
            return None;
        }
    }

    // Apply listen filter
    if filter.listen_only && !matches!(state, SocketState::Listen) {
        return None;
    }

    Some(SocketInfo {
        protocol,
        local_address,
        remote_address,
        state,
        pid: Some(pid),
        process_name: Some(command),
        command_line: None, // Will be filled in if needed
    })
}

/// Parse lsof address information
/// Formats:
///   "*:8080 (LISTEN)" - listening on all interfaces
///   "192.168.1.1:22" - local address only
///   "192.168.1.1:22->10.0.0.1:54321 (ESTABLISHED)" - full connection
fn parse_lsof_addresses(info: &str, is_tcp: bool) -> Option<(SocketAddress, Option<SocketAddress>, SocketState)> {
    // Remove protocol prefix if present
    let info = info
        .strip_prefix("TCP")
        .or_else(|| info.strip_prefix("UDP"))
        .unwrap_or(info)
        .trim();

    // Parse state if present (for TCP)
    let state = if is_tcp {
        if info.contains("(LISTEN)") {
            SocketState::Listen
        } else if info.contains("(ESTABLISHED)") {
            SocketState::Established
        } else if info.contains("(TIME_WAIT)") {
            SocketState::TimeWait
        } else if info.contains("(CLOSE_WAIT)") {
            SocketState::CloseWait
        } else {
            SocketState::Established // Default to established for active connections
        }
    } else {
        SocketState::Unknown // UDP doesn't have connection states
    };

    // Remove state suffix
    let info = info
        .split('(')
        .next()
        .unwrap_or(info)
        .trim();

    // Check for connection pair (local->remote)
    if info.contains("->") {
        let addrs: Vec<&str> = info.split("->").collect();
        if addrs.len() == 2 {
            let local = parse_lsof_address(addrs[0])?;
            let remote = parse_lsof_address(addrs[1])?;
            return Some((local, Some(remote), state));
        }
    }

    // Just local address
    let local = parse_lsof_address(info)?;
    Some((local, None, state))
}

/// Parse a single lsof address
/// Format: "*:8080" or "192.168.1.1:22" or "[::]:8080"
fn parse_lsof_address(addr: &str) -> Option<SocketAddress> {
    let addr = addr.trim();

    // Handle wildcard address
    if addr == "*" {
        return Some(SocketAddress {
            ip: "0.0.0.0".to_string(),
            port: 0,
        });
    }

    // Handle IPv6 addresses [::]:port
    if addr.starts_with('[') {
        if let Some(closing_bracket) = addr.find(']') {
            let ip = &addr[1..closing_bracket];
            let port_str = addr.get(closing_bracket + 2..).unwrap_or("0");
            let port = port_str.parse::<u16>().ok().unwrap_or(0);
            return Some(SocketAddress {
                ip: ip.to_string(),
                port,
            });
        }
    }

    // Handle IPv4 addresses and ports
    if let Some(colon_pos) = addr.rfind(':') {
        let ip = &addr[..colon_pos];
        let port_str = &addr[colon_pos + 1..];
        let port = port_str.parse::<u16>().ok().unwrap_or(0);

        // Convert wildcard to 0.0.0.0
        let ip = if ip == "*" {
            "0.0.0.0"
        } else {
            ip
        };

        return Some(SocketAddress {
            ip: ip.to_string(),
            port,
        });
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_lsof_address_ipv4() {
        let addr = "192.168.1.1:8080";
        let result = parse_lsof_address(addr);
        assert!(result.is_some());
        let socket_addr = result.unwrap();
        assert_eq!(socket_addr.ip, "192.168.1.1");
        assert_eq!(socket_addr.port, 8080);
    }

    #[test]
    fn test_parse_lsof_address_wildcard() {
        let addr = "*:22";
        let result = parse_lsof_address(addr);
        assert!(result.is_some());
        let socket_addr = result.unwrap();
        assert_eq!(socket_addr.ip, "0.0.0.0");
        assert_eq!(socket_addr.port, 22);
    }

    #[test]
    fn test_parse_lsof_address_localhost() {
        let addr = "127.0.0.1:3000";
        let result = parse_lsof_address(addr);
        assert!(result.is_some());
        let socket_addr = result.unwrap();
        assert_eq!(socket_addr.ip, "127.0.0.1");
        assert_eq!(socket_addr.port, 3000);
    }

    #[test]
    fn test_parse_lsof_addresses_listen() {
        let info = "*:8080 (LISTEN)";
        let (local, remote, state) = parse_lsof_addresses(info, true).unwrap();
        assert_eq!(local.ip, "0.0.0.0");
        assert_eq!(local.port, 8080);
        assert!(remote.is_none());
        assert_eq!(state, SocketState::Listen);
    }

    #[test]
    fn test_parse_lsof_addresses_established() {
        let info = "192.168.1.1:22->10.0.0.1:54321 (ESTABLISHED)";
        let (local, remote, state) = parse_lsof_addresses(info, true).unwrap();
        assert_eq!(local.ip, "192.168.1.1");
        assert_eq!(local.port, 22);
        assert!(remote.is_some());
        assert_eq!(remote.unwrap().port, 54321);
        assert_eq!(state, SocketState::Established);
    }

    #[test]
    fn test_parse_lsof_addresses_udp() {
        let info = "*:53";
        let (local, remote, state) = parse_lsof_addresses(info, false).unwrap();
        assert_eq!(local.ip, "0.0.0.0");
        assert_eq!(local.port, 53);
        assert!(remote.is_none());
        assert_eq!(state, SocketState::Unknown);
    }
}

