//! Implementation of the get command
//!
//! This module provides functionality to query specific ports or processes,
//! returning detailed information in either human-readable or JSON format.

use crate::cli::GetCommands;
use crate::core::{SocketFilter, format_sockets_json, format_process_table, format_process_json};
use crate::platform::{get_sockets, get_process_info};
use anyhow::Result;

/// Execute the get command with the specified subcommand
///
/// # Arguments
/// * `command` - The get subcommand to execute (port or pid)
/// * `json_output` - Whether to output in JSON format
///
/// # Returns
/// * `Ok(())` if the command executed successfully
/// * `Err(e)` if an error occurred
pub fn execute(command: &GetCommands, json_output: bool) -> Result<()> {
    match command {
        GetCommands::Port { port, verbose } => {
            execute_get_port(*port, *verbose, json_output)
        }
        GetCommands::Pid { pid } => {
            execute_get_pid(*pid, json_output)
        }
    }
}

/// Get information about sockets using a specific port
///
/// # Arguments
/// * `port` - The port number to query
/// * `_verbose` - Whether to show verbose output (currently unused)
/// * `json_output` - Whether to output in JSON format
///
/// # Returns
/// * `Ok(())` if the command executed successfully
/// * `Err(e)` if an error occurred
fn execute_get_port(port: u16, _verbose: bool, json_output: bool) -> Result<()> {
    let mut filter = SocketFilter::default();
    filter.port = Some(port);

    let sockets = get_sockets(&filter)?;

    if sockets.is_empty() {
        println!("No sockets found using port {}", port);
        return Ok(());
    }

    if json_output {
        let output = format_sockets_json(&sockets)?;
        println!("{}", output);
    } else {
        println!("Sockets using port {}:\n", port);
        for socket in &sockets {
            println!("Protocol: {}", socket.protocol);
            println!("Local Address: {}", socket.local_address);
            if let Some(ref remote) = socket.remote_address {
                println!("Remote Address: {}", remote);
            }
            println!("State: {}", socket.state);
            if let Some(pid) = socket.pid {
                println!("PID: {}", pid);
                if let Some(ref name) = socket.process_name {
                    println!("Process: {}", name);
                }
                if let Some(ref cmd) = socket.command_line {
                    println!("Command: {}", cmd);
                }
            }
            println!();
        }
    }

    Ok(())
}

/// Get information about a specific process and its open sockets
///
/// # Arguments
/// * `pid` - The process ID to query
/// * `json_output` - Whether to output in JSON format
///
/// # Returns
/// * `Ok(())` if the command executed successfully
/// * `Err(e)` if an error occurred
fn execute_get_pid(pid: u32, json_output: bool) -> Result<()> {
    let process = get_process_info(pid)?;

    if json_output {
        let output = format_process_json(&process)?;
        println!("{}", output);
    } else {
        let output = format_process_table(&process);
        println!("{}", output);
    }

    Ok(())
}
