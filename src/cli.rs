//! Command-line interface definition

use clap::{Parser, Subcommand};
use clap::Args;

/// ptu - A cross-platform network port management tool
#[derive(Parser, Debug)]
#[command(name = "ptu")]
#[command(author = "ptu contributors")]
#[command(version = "0.1.0")]
#[command(about = "A modern alternative to ss, lsof, and netstat", long_about = None)]
pub struct Cli {
    /// Output in JSON format
    #[arg(short, long, global = true)]
    pub json: bool,

    /// Subcommand to execute (list or get)
    #[command(subcommand)]
    pub command: Commands,
}

/// Available subcommands for ptu
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List network sockets with various filters
    List(ListOptions),

    /// Get detailed information about a specific port or process
    #[command(subcommand)]
    Get(GetCommands),
}

/// Options for the list command
#[derive(Args, Debug, Clone)]
pub struct ListOptions {
    /// Show only TCP sockets
    #[arg(short = 't', long)]
    pub tcp: bool,

    /// Show only UDP sockets
    #[arg(short = 'u', long)]
    pub udp: bool,

    /// Show only listening sockets
    #[arg(short = 'l', long)]
    pub listen: bool,

    /// Filter by process ID
    #[arg(short = 'p', long, value_name = "PID")]
    pub pid: Option<u32>,

    /// Filter by port number
    #[arg(short = 'P', long, value_name = "PORT")]
    pub port: Option<u16>,

    /// Show IPv4 sockets
    #[arg(short = '4', long)]
    pub ipv4: bool,

    /// Show IPv6 sockets
    #[arg(short = '6', long)]
    pub ipv6: bool,
}

/// Get command subcommands
#[derive(Subcommand, Debug)]
pub enum GetCommands {
    /// Get information about a specific port
    Port {
        /// Port number to query
        port: u16,

        /// Show process details including command line
        #[arg(short, long)]
        verbose: bool,
    },

    /// Get information about a specific process
    Pid {
        /// Process ID to query
        pid: u32,
    },
}
