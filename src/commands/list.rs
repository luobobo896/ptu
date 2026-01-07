//! Implementation of the list command
//!
//! This module provides functionality to list network sockets with various filters,
//! supporting protocol, port, process, and address family filtering.

use crate::cli::ListOptions;
use crate::core::{SocketFilter, Protocol, format_sockets_table, format_sockets_json};
use crate::platform::get_sockets;
use anyhow::Result;

/// Execute the list command with the specified options
///
/// # Arguments
/// * `options` - The list command options (filters and output settings)
/// * `json_output` - Whether to output in JSON format
///
/// # Returns
/// * `Ok(())` if the command executed successfully
/// * `Err(e)` if an error occurred
pub fn execute(options: &ListOptions, json_output: bool) -> Result<()> {
    let mut filter = SocketFilter::default();

    // Determine which protocols to show
    let show_tcp = !options.udp; // Show TCP unless UDP is explicitly specified
    let show_udp = !options.tcp; // Show UDP unless TCP is explicitly specified

    let mut protocols = Vec::new();

    if show_tcp {
        // Show TCP4 if ipv4 is specified OR neither ipv4 nor ipv6 is specified
        if options.ipv4 || (!options.ipv4 && !options.ipv6) {
            protocols.push(Protocol::Tcp);
        }
        // Show TCP6 if ipv6 is specified OR neither ipv4 nor ipv6 is specified
        if options.ipv6 || (!options.ipv4 && !options.ipv6) {
            protocols.push(Protocol::Tcp6);
        }
    }

    if show_udp {
        // Show UDP4 if ipv4 is specified OR neither ipv4 nor ipv6 is specified
        if options.ipv4 || (!options.ipv4 && !options.ipv6) {
            protocols.push(Protocol::Udp);
        }
        // Show UDP6 if ipv6 is specified OR neither ipv4 nor ipv6 is specified
        if options.ipv6 || (!options.ipv4 && !options.ipv6) {
            protocols.push(Protocol::Udp6);
        }
    }

    if !protocols.is_empty() {
        filter.protocols = Some(protocols);
    }

    filter.listen_only = options.listen;
    filter.pid = options.pid;
    filter.port = options.port;

    let sockets = get_sockets(&filter)?;

    if json_output {
        let output = format_sockets_json(&sockets)?;
        println!("{}", output);
    } else {
        let output = format_sockets_table(&sockets);
        println!("{}", output);
    }

    Ok(())
}
