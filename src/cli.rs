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
    Serve {
        /// periodically run cleanup job, interval is taken from configuration file
        #[arg(long)]
        periodically_run_cleanup: bool,
    },

    /// Access current wireguard status of all clients or a single client
    #[command()]
    Status {
        /// determine status only for this client
        public_key: Option<String>,
        /// format output as json
        #[arg(long)]
        json: bool,
    },

    /// Register new client and return assigned IP
    #[command()]
    Register {
        /// client public key
        public_key: String,
        /// format output as json
        #[arg(long)]
        json: bool,
    },

    /// Unregister client
    #[command()]
    Unregister {
        /// client public key
        public_key: String,
        /// format output as json
        #[arg(long)]
        json: bool,
    },

    /// Remove expired clients that have been connected before
    #[command()]
    RemoveExpired {
        /// overwrite configured or default client handshake timeout
        #[arg(long)]
        client_handshake_timeout_s: Option<u64>,
        /// format output as json
        #[arg(long)]
        json: bool,
    },

    /// Remove clients that registered but never connected
    #[command()]
    RemoveNeverConnected {
        /// format output as json
        #[arg(long)]
        json: bool,
    },
}

pub fn parse() -> Cli {
    Cli::parse()
}
