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

    /// Access current wireguard status
    #[command()]
    Status {
        /// format output as json
        #[arg(long)]
        json: bool,
    },

    /// Register new client and return it's assigned IP
    #[command()]
    Register {
        /// client public key
        #[arg(short, long, required = true)]
        public_key: String,
        /// format output as json
        #[arg(long)]
        json: bool,
    },
}

pub fn parse() -> Cli {
    Cli::parse()
}
