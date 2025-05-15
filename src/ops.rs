use std::ffi::OsStr;
use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;
use std::time::Duration;

use crate::config::Config;
use crate::ip_range::IpRange;
use crate::metrics::Metrics;

#[derive(Debug, Clone)]
pub struct Ops {
    pub client_address_range: IpRange,
    pub rocket_address: IpAddr,
    pub rocket_port: u16,
    pub wg_interface_config: PathBuf,
    pub client_handshake_timeout: Duration,
    pub client_cleanup_interval: Duration,
    pub metrics: Metrics,
}

impl Ops {
    pub fn interface(&self) -> Option<&str> {
        self.wg_interface_config.file_stem().and_then(OsStr::to_str)
    }
}

impl From<Config> for Ops {
    fn from(config: Config) -> Self {
        let def_rocket_address = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let def_rocket_port = 8000;
        let def_client_handshake_timeout = Duration::from_secs(5 * 60);
        let def_client_cleanup_interval = Duration::from_secs(3 * 60);

        Self {
            client_address_range: config.allowed_client_ips.clone(),
            rocket_address: config.endpoint.map(|addr| addr.ip()).unwrap_or(def_rocket_address),
            rocket_port: config.endpoint.map(|addr| addr.port()).unwrap_or(def_rocket_port),
            wg_interface_config: config.wireguard_config_path.clone(),
            client_handshake_timeout: config
                .client_handshake_timeout_s
                .map(Duration::from_secs)
                .unwrap_or(def_client_handshake_timeout),
            client_cleanup_interval: config
                .client_cleanup_interval_s
                .map(Duration::from_secs)
                .unwrap_or(def_client_cleanup_interval),
            metrics: Metrics::new(),
        }
    }
}
