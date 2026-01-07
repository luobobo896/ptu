//! Output formatting for ptu

use crate::core::types::{SocketInfo, ProcessInfo};
use serde_json;
use tabled::{
    settings::style::Style,
    Table,
    Tabled,
};

/// Format socket info as a table
pub fn format_sockets_table(sockets: &[SocketInfo]) -> String {
    if sockets.is_empty() {
        return String::from("No sockets found.");
    }

    let rows: Vec<SocketTableRow> = sockets
        .iter()
        .map(|s| SocketTableRow {
            protocol: s.protocol.to_string(),
            local_addr: s.local_address.to_string(),
            remote_addr: s
                .remote_address
                .as_ref()
                .map(|a| a.to_string())
                .unwrap_or_else(|| "-".to_string()),
            state: s.state.to_string(),
            pid: s.pid.map(|p| p.to_string()).unwrap_or_else(|| "-".to_string()),
            process_name: s.process_name.clone().unwrap_or_else(|| "-".to_string()),
        })
        .collect();

    Table::new(rows)
        .with(Style::sharp())
        .to_string()
}

/// Format process info as a table
pub fn format_process_table(process: &ProcessInfo) -> String {
    format!(
        "Process {} (PID: {})\nCommand: {}\n\nOpen Sockets:\n{}",
        process.name,
        process.pid,
        process
            .command_line
            .as_deref()
            .unwrap_or("(unknown)"),
        format_sockets_table(&process.sockets)
    )
}

/// Format socket info as JSON
pub fn format_sockets_json(sockets: &[SocketInfo]) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(sockets)
}

/// Format process info as JSON
pub fn format_process_json(process: &ProcessInfo) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(process)
}

#[derive(Tabled)]
struct SocketTableRow {
    #[tabled(rename = "PROTOCOL")]
    protocol: String,
    #[tabled(rename = "LOCAL ADDRESS")]
    local_addr: String,
    #[tabled(rename = "REMOTE ADDRESS")]
    remote_addr: String,
    #[tabled(rename = "STATE")]
    state: String,
    #[tabled(rename = "PID")]
    pid: String,
    #[tabled(rename = "PROCESS NAME")]
    process_name: String,
}
