use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;

use crate::ip_range::IpRange;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    // wireguard interface server address
    pub server_address: Ipv4Addr,
    // this effectively determines the number of clients that are concurrently allowed to connect
    pub allowed_client_ips: IpRange,
    // web server endpoint
    pub endpoint: Option<SocketAddr>,
    // wg interface name and configuration file to store server state
    pub wireguard_config_path: PathBuf,
    // determines when a client is considered disconnected
    pub client_handshake_timeout_s: Option<u64>,
    // interval at which client removal job is run
    pub client_cleanup_interval_s: Option<u64>,
}
