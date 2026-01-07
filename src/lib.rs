//! ptu - A cross-platform network port management tool
//!
//! This library provides the core functionality for the ptu command-line tool,
//! which allows users to list and query network sockets and their associated
//! processes on macOS and Linux systems.

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod cli;
pub mod commands;
pub mod core;
pub mod platform;

pub use crate::core::{
    format_process_json, format_process_table, format_sockets_json, format_sockets_table,
    ProcessInfo, Protocol, SocketAddress, SocketFilter, SocketInfo, SocketState,
};
