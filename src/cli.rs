use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Gnosis VPN server - orchestrate WireGuard server for GnosisVPN connections
#[derive(Debug, Parser)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Specify config file to use
    #[arg(
        short,
        long,
        env = "GNOSISVPN_SERVER_CONFIG_FILE",
        default_value = "/etc/gnosisvpn-server/config.toml"
    )]
    pub config_file: PathBuf,
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
