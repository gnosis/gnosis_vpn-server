use clap::{Parser, Subcommand};
use std::net::Ipv4Addr;
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
    /// Start http server listening for client requests.
    /// Allows periodic wireguard interface management and configuration sync.
    #[command()]
    Serve {
        /// Periodically run cleanup job, interval is taken from configuration file
        #[arg(long)]
        periodically_run_cleanup: bool,
        /// Manage wg interface by using wg-quick up/down commands
        #[arg(long)]
        sync_wg_interface: bool,
    },

    /// Access current wireguard status of all clients or a single client
    #[command()]
    Status {
        /// Determine status only for this client
        public_key: Option<String>,
        /// Format output as json
        #[arg(long)]
        json: bool,
    },

    /// Register new client into wg interface and return assigned IP
    #[command()]
    Register {
        /// Client public key
        public_key: String,
        /// Force specific IP address instead of using random available one
        force_ip: Option<Ipv4Addr>,
        /// Format output as json
        #[arg(long)]
        json: bool,
        /// Persist changes to configuration file
        #[arg(long)]
        persist_config: bool,
    },

    /// Unregister client from wg interface
    #[command()]
    Unregister {
        /// Client public key
        public_key: String,
        /// Format output as json
        #[arg(long)]
        json: bool,
        /// Persist changes to configuration file
        #[arg(long)]
        persist_config: bool,
    },

    /// Remove expired clients that have been connected before
    #[command()]
    RemoveExpired {
        /// Overwrite expiration (last client handshake) timeout
        #[arg(long)]
        client_handshake_timeout_s: Option<u64>,
        /// Format output as json
        #[arg(long)]
        json: bool,
        /// Persist changes to configuration file
        #[arg(long)]
        persist_config: bool,
    },

    /// Remove clients that registered but never connected
    #[command()]
    RemoveNeverConnected {
        /// Format output as json
        #[arg(long)]
        json: bool,
        /// Persist changes to configuration file
        #[arg(long)]
        persist_config: bool,
    },
}

pub fn parse() -> Cli {
    Cli::parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_serve_command_flags() -> anyhow::Result<()> {
        let cli = Cli::parse_from(["bin", "serve", "--periodically-run-cleanup", "--sync-wg-interface"]);
        match cli.command {
            Command::Serve {
                periodically_run_cleanup,
                sync_wg_interface,
            } => {
                assert!(periodically_run_cleanup);
                assert!(sync_wg_interface);
            }
            _ => panic!("expected Serve command"),
        }

        Ok(())
    }

    #[test]
    fn should_parse_register_command_with_forced_ip_and_json() -> anyhow::Result<()> {
        let cli = Cli::parse_from(["bin", "register", "pubkey", "10.0.0.2", "--json", "--persist-config"]);
        match cli.command {
            Command::Register {
                public_key,
                force_ip,
                json,
                persist_config,
            } => {
                assert_eq!(public_key, "pubkey");
                assert_eq!(force_ip, Some(Ipv4Addr::new(10, 0, 0, 2)));
                assert!(json);
                assert!(persist_config);
            }
            _ => panic!("expected Register command"),
        }

        Ok(())
    }
}
