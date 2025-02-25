use clap::{Parser, Subcommand};

/// Gnosis VPN server - orchestrate WireGuard server for GnosisVPN connections
#[derive(Debug, Parser)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Start http server listening for client requests
    #[command()]
    Serve {},
}

pub fn parse() -> Cli {
    Cli::parse()
}
