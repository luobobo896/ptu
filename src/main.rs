//! ptu - A cross-platform network port management tool
//!
//! A modern alternative to ss, lsof, and netstat.

use anyhow::Result;
use clap::Parser;
use env_logger::Env;
use log::info;
use ptu::cli::{Cli, Commands};
use ptu::commands::{execute_get, execute_list};

fn main() -> Result<()> {
    // Initialize logger
    env_logger::Builder::from_env(Env::default().default_filter_or("warn")).init();

    // Parse command-line arguments
    let cli = Cli::parse();

    info!("ptu started with command: {:?}", cli.command);

    // Execute the appropriate command
    match cli.command {
        Commands::List(options) => {
            info!("Executing list command with options: {:?}", options);
            execute_list(&options, cli.json)?;
        }
        Commands::Get(get_command) => {
            info!("Executing get command: {:?}", get_command);
            execute_get(&get_command, cli.json)?;
        }
    }

    Ok(())
}
