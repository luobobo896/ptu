//! Integration tests for ptu
//!
//! These tests verify the end-to-end functionality of ptu commands.
//! Tests run against the actual system state and require appropriate permissions.

use std::process::Command;

/// Helper struct to manage ptu command execution
struct PtuCommand {
    args: Vec<String>,
}

impl PtuCommand {
    fn new() -> Self {
        PtuCommand { args: Vec::new() }
    }

    fn arg(mut self, arg: &str) -> Self {
        self.args.push(arg.to_string());
        self
    }

    fn json(mut self) -> Self {
        self.args.push("-j".to_string());
        self
    }

    fn execute(&self) -> PtuResult {
        // Try multiple possible locations for ptu binary
        let possible_paths = vec![
            std::env::var("CARGO_BIN_EXE_ptu").ok(),
            Some("../target/release/ptu".to_string()),
            Some("./target/release/ptu".to_string()),
            Some("target/release/ptu".to_string()),
            Some("/Users/hanson/ptu/target/release/ptu".to_string()),
        ];

        let ptu_path = possible_paths
            .iter()
            .filter_map(|p| p.as_ref())
            .find(|path| std::path::Path::new(path).exists())
            .expect(&format!(
                "Could not find ptu binary. Tried: {:?}. Please build with: cargo build --release",
                possible_paths
            ));

        let output = Command::new(&ptu_path)
            .args(&self.args)
            .output()
            .expect(&format!("Failed to execute ptu at {}", ptu_path));

        PtuResult {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
        }
    }
}

struct PtuResult {
    success: bool,
    stdout: String,
    stderr: String,
    exit_code: i32,
}

#[test]
fn test_list_all_sockets() {
    let result = PtuCommand::new().arg("list").json().execute();

    assert!(result.success, "Command failed: {}", result.stderr);
    assert!(!result.stdout.is_empty(), "Output should not be empty");

    // Parse JSON output
    let sockets: Vec<serde_json::Value> =
        serde_json::from_str(&result.stdout).expect("Invalid JSON output");

    assert!(!sockets.is_empty(), "Should find at least one socket");
}

#[test]
fn test_list_listen_sockets() {
    let result = PtuCommand::new()
        .arg("list")
        .arg("--listen")
        .json()
        .execute();

    assert!(result.success, "Command failed: {}", result.stderr);

    let sockets: Vec<serde_json::Value> =
        serde_json::from_str(&result.stdout).expect("Invalid JSON output");

    // All returned sockets should be in LISTEN state
    for socket in &sockets {
        let state = socket["state"]
            .as_str()
            .expect("State field missing");
        assert_eq!(
            state, "LISTEN",
            "All sockets should be in LISTEN state, found {}",
            state
        );
    }
}

#[test]
fn test_list_tcp_sockets() {
    let result = PtuCommand::new()
        .arg("list")
        .arg("--tcp")
        .json()
        .execute();

    assert!(result.success, "Command failed: {}", result.stderr);

    let sockets: Vec<serde_json::Value> =
        serde_json::from_str(&result.stdout).expect("Invalid JSON output");

    // All returned sockets should be TCP
    for socket in &sockets {
        let protocol = socket["protocol"]
            .as_str()
            .expect("Protocol field missing");
        assert!(
            protocol.starts_with("tcp"),
            "All sockets should be TCP, found {}",
            protocol
        );
    }
}

#[test]
fn test_list_udp_sockets() {
    let result = PtuCommand::new()
        .arg("list")
        .arg("--udp")
        .json()
        .execute();

    assert!(result.success, "Command failed: {}", result.stderr);

    let sockets: Vec<serde_json::Value> =
        serde_json::from_str(&result.stdout).expect("Invalid JSON output");

    // All returned sockets should be UDP
    for socket in &sockets {
        let protocol = socket["protocol"]
            .as_str()
            .expect("Protocol field missing");
        assert!(
            protocol.starts_with("udp"),
            "All sockets should be UDP, found {}",
            protocol
        );
    }
}

#[test]
fn test_list_ipv4_sockets() {
    let result = PtuCommand::new()
        .arg("list")
        .arg("--ipv4")
        .json()
        .execute();

    assert!(result.success, "Command failed: {}", result.stderr);

    let sockets: Vec<serde_json::Value> =
        serde_json::from_str(&result.stdout).expect("Invalid JSON output");

    // All returned sockets should be IPv4
    for socket in &sockets {
        let protocol = socket["protocol"]
            .as_str()
            .expect("Protocol field missing");
        assert!(
            protocol == "tcp" || protocol == "udp",
            "All sockets should be IPv4 (tcp/udp), found {}",
            protocol
        );
    }
}

#[test]
fn test_list_ipv6_sockets() {
    let result = PtuCommand::new()
        .arg("list")
        .arg("--ipv6")
        .json()
        .execute();

    assert!(result.success, "Command failed: {}", result.stderr);

    let sockets: Vec<serde_json::Value> =
        serde_json::from_str(&result.stdout).expect("Invalid JSON output");

    // All returned sockets should be IPv6
    for socket in &sockets {
        let protocol = socket["protocol"]
            .as_str()
            .expect("Protocol field missing");
        assert!(
            protocol == "tcp6" || protocol == "udp6",
            "All sockets should be IPv6 (tcp6/udp6), found {}",
            protocol
        );
    }
}

#[test]
fn test_list_combined_filters() {
    let result = PtuCommand::new()
        .arg("list")
        .arg("--tcp")
        .arg("--listen")
        .json()
        .execute();

    assert!(result.success, "Command failed: {}", result.stderr);

    let sockets: Vec<serde_json::Value> =
        serde_json::from_str(&result.stdout).expect("Invalid JSON output");

    // All sockets should be TCP and in LISTEN state
    for socket in &sockets {
        let protocol = socket["protocol"]
            .as_str()
            .expect("Protocol field missing");
        let state = socket["state"]
            .as_str()
            .expect("State field missing");

        assert!(
            protocol.starts_with("tcp"),
            "Sockets should be TCP, found {}",
            protocol
        );
        assert_eq!(
            state, "LISTEN",
            "Sockets should be in LISTEN state, found {}",
            state
        );
    }
}

#[test]
fn test_get_port_invalid() {
    // Test with a port that's unlikely to be in use
    let result = PtuCommand::new()
        .arg("get")
        .arg("port")
        .arg("1")
        .execute();

    // Should succeed but report no sockets found
    assert!(result.success || result.exit_code == 0);

    let output = &result.stdout;
    assert!(
        output.contains("No sockets found") ||
        output.contains("Sockets using port 1"),
        "Expected 'No sockets found' or socket info"
    );
}

#[test]
fn test_get_pid_current_process() {
    // Get current process ID
    let current_pid = std::process::id();

    let result = PtuCommand::new()
        .arg("get")
        .arg("pid")
        .arg(&current_pid.to_string())
        .json()
        .execute();

    // This might fail if ptu itself doesn't have open sockets,
    // but the command should execute without crashing
    assert!(result.success, "Command failed: {}", result.stderr);

    // Output should be valid JSON
    let _process: serde_json::Value =
        serde_json::from_str(&result.stdout).expect("Invalid JSON output");

    // Process should have our PID
    let process: serde_json::Value =
        serde_json::from_str(&result.stdout).expect("Invalid JSON output");

    assert_eq!(
        process["pid"]
            .as_u64()
            .expect("PID field missing") as u32,
        current_pid,
        "PID should match current process"
    );
}

#[test]
fn test_json_output_format() {
    let result = PtuCommand::new()
        .arg("list")
        .arg("--listen")
        .json()
        .execute();

    assert!(result.success, "Command failed: {}", result.stderr);

    // Should be valid JSON array
    let sockets: Vec<serde_json::Value> =
        serde_json::from_str(&result.stdout).expect("Invalid JSON output");

    // Each socket should have required fields
    for socket in &sockets {
        assert!(
            socket.get("protocol").is_some(),
            "Missing protocol field"
        );
        assert!(
            socket.get("local_address").is_some(),
            "Missing local_address field"
        );
        assert!(
            socket.get("state").is_some(),
            "Missing state field"
        );
    }
}

#[test]
fn test_table_output_format() {
    let result = PtuCommand::new()
        .arg("list")
        .arg("--listen")
        .execute();

    assert!(result.success, "Command failed: {}", result.stderr);

    let output = &result.stdout;

    // Table output should contain headers
    assert!(
        output.contains("PROTOCOL") || output.contains("Protocol"),
        "Output should contain protocol header"
    );
    assert!(
        output.contains("LOCAL ADDRESS") || output.contains("Local"),
        "Output should contain local address header"
    );
    assert!(
        output.contains("STATE") || output.contains("State"),
        "Output should contain state header"
    );
}

#[test]
fn test_help_command() {
    let result = PtuCommand::new()
        .arg("--help")
        .execute();

    assert!(result.success, "Help command failed");

    let output = &result.stdout;
    assert!(
        output.contains("ptu") || output.contains("Usage"),
        "Help output should contain usage information"
    );
    assert!(
        output.contains("list") || output.contains("LIST"),
        "Help should mention list command"
    );
    assert!(
        output.contains("get") || output.contains("GET"),
        "Help should mention get command"
    );
}

#[test]
fn test_version_command() {
    let result = PtuCommand::new()
        .arg("--version")
        .execute();

    assert!(result.success, "Version command failed");

    let output = &result.stdout;
    assert!(
        output.contains("0.1.0") || output.contains("ptu"),
        "Version output should contain version number"
    );
}

#[test]
fn test_invalid_command() {
    let result = PtuCommand::new()
        .arg("invalid-command")
        .execute();

    // Should fail
    assert!(!result.success, "Invalid command should fail");
}

#[test]
fn test_list_empty_output() {
    // Try to list with filters that should return no results
    // Use a valid but unlikely-to-be-used port number
    let result = PtuCommand::new()
        .arg("list")
        .arg("--port")
        .arg("65430") // Valid port but unlikely to be in use
        .json()
        .execute();

    // Should still succeed (return empty array)
    assert!(result.success, "Command failed: {}", result.stderr);

    let sockets: Vec<serde_json::Value> =
        serde_json::from_str(&result.stdout).expect("Invalid JSON output");

    assert_eq!(
        sockets.len(),
        0,
        "Should return empty array for unused port"
    );
}
