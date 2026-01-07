//! Platform-specific implementations

use crate::core::types::{ProcessInfo, SocketFilter, SocketInfo};
use anyhow::Result;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
use macos as platform_impl;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
use linux as platform_impl;

/// Get all socket information for the current platform
pub fn get_sockets(filter: &SocketFilter) -> Result<Vec<SocketInfo>> {
    platform_impl::get_sockets(filter)
}

/// Get process information by PID for the current platform
pub fn get_process_info(pid: u32) -> Result<ProcessInfo> {
    platform_impl::get_process_info(pid)
}
