//! Core data structures for ptu

use serde::{Deserialize, Serialize};
use std::fmt;

/// Socket protocol type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    /// Transmission Control Protocol over IPv4
    Tcp,
    /// Transmission Control Protocol over IPv6
    Tcp6,
    /// User Datagram Protocol over IPv4
    Udp,
    /// User Datagram Protocol over IPv6
    Udp6,
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Protocol::Tcp => write!(f, "TCP"),
            Protocol::Tcp6 => write!(f, "TCP6"),
            Protocol::Udp => write!(f, "UDP"),
            Protocol::Udp6 => write!(f, "UDP6"),
        }
    }
}

/// Socket connection state
///
/// Represents the state of a TCP socket connection according to the TCP state machine.
/// UDP sockets will always be in the Unknown state as they are connectionless.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SocketState {
    /// Connection is established and data can be transferred
    Established,
    /// SYN sent, waiting for ACK
    SynSent,
    /// SYN received, waiting for ACK
    SynRecv,
    /// Connection is closing, waiting for remote FIN ACK
    FinWait1,
    /// Connection is closing, waiting for remote FIN
    FinWait2,
    /// Connection is closed, waiting for 2MSL timeout
    TimeWait,
    /// Connection is closed
    Close,
    /// Remote side has closed, waiting for application to close
    CloseWait,
    /// Waiting for last ACK
    LastAck,
    /// Socket is listening for incoming connections
    Listen,
    /// Both sides have closed simultaneously
    Closing,
    /// Unknown state (used for UDP or other protocols)
    Unknown,
}

impl fmt::Display for SocketState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SocketState::Established => write!(f, "ESTABLISHED"),
            SocketState::SynSent => write!(f, "SYN_SENT"),
            SocketState::SynRecv => write!(f, "SYN_RECV"),
            SocketState::FinWait1 => write!(f, "FIN_WAIT1"),
            SocketState::FinWait2 => write!(f, "FIN_WAIT2"),
            SocketState::TimeWait => write!(f, "TIME_WAIT"),
            SocketState::Close => write!(f, "CLOSE"),
            SocketState::CloseWait => write!(f, "CLOSE_WAIT"),
            SocketState::LastAck => write!(f, "LAST_ACK"),
            SocketState::Listen => write!(f, "LISTEN"),
            SocketState::Closing => write!(f, "CLOSING"),
            SocketState::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

/// Network address (IP and port)
///
/// Represents a socket address with an IP address and port number.
/// The IP address can be IPv4, IPv6, or a wildcard (0.0.0.0).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SocketAddress {
    /// IP address (IPv4, IPv6, or wildcard like 0.0.0.0)
    pub ip: String,
    /// Port number
    pub port: u16,
}

impl fmt::Display for SocketAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.ip, self.port)
    }
}

/// Information about a network socket
///
/// Contains comprehensive information about a network socket including
/// protocol, addresses, state, and associated process information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocketInfo {
    /// Protocol type (TCP/UDP over IPv4/IPv6)
    pub protocol: Protocol,
    /// Local address (IP and port)
    pub local_address: SocketAddress,
    /// Remote address (IP and port), None for listening sockets
    pub remote_address: Option<SocketAddress>,
    /// Socket state (e.g., LISTEN, ESTABLISHED)
    pub state: SocketState,
    /// Process ID that owns this socket, if available
    pub pid: Option<u32>,
    /// Process name, if available
    pub process_name: Option<String>,
    /// Full command line of the process, if available
    pub command_line: Option<String>,
}

/// Information about a process and its open sockets
///
/// Contains process identification and all network sockets opened by this process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    /// Process ID
    pub pid: u32,
    /// Process name
    pub name: String,
    /// Full command line used to start the process, if available
    pub command_line: Option<String>,
    /// All network sockets opened by this process
    pub sockets: Vec<SocketInfo>,
}

/// Filter options for listing sockets
///
/// Used to filter network sockets based on various criteria.
/// All fields are optional; sockets must match all specified filters.
#[derive(Debug, Clone, Default)]
pub struct SocketFilter {
    /// Protocols to include (None means all protocols)
    pub protocols: Option<Vec<Protocol>>,
    /// Only show sockets in LISTEN state
    pub listen_only: bool,
    /// Filter by process ID (None means all processes)
    pub pid: Option<u32>,
    /// Filter by port number (None means all ports)
    pub port: Option<u16>,
}
