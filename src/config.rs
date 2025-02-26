use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;

use crate::ip_range::IpRange;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    // this effectively determines the number of clients that are concurrently allowed to connect
    pub allowed_client_ips: IpRange,
    // web server endpoint
    pub endpoint: Option<SocketAddr>,
    // wg device name and configuration file to store server state
    pub wireguard_config_path: PathBuf,
    // client last hand_shake removal timeout
    pub client_handshake_timeout_s: Option<u64>,
}
