use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;
use std::time::Duration;

use crate::config::Config;
use crate::ip_range::IpRange;

pub struct Ops {
    pub client_address_range: IpRange,
    pub rocket_address: IpAddr,
    pub rocket_port: u16,
    pub wg_device_config: PathBuf,
    pub client_handshake_timeout: Duration,
}

impl Default for Ops {
    fn default() -> Self {
        Self {
            client_address_range: IpRange::new(Ipv4Addr::new(10, 128, 0, 0), Ipv4Addr::new(10, 128, 0, 10)),
            rocket_address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            rocket_port: 8000,
            wg_device_config: PathBuf::from("/etc/wireguard/wggnosisvpn.conf"),
            client_handshake_timeout: Duration::from_secs(5 * 60),
        }
    }
}

impl From<Config> for Ops {
    fn from(config: Config) -> Self {
        let defaults = Ops::default();
        Self {
            client_address_range: config.allowed_client_ips.unwrap_or(defaults.client_address_range),
            rocket_address: config.endpoint.map(|addr| addr.ip()).unwrap_or(defaults.rocket_address),
            rocket_port: config.endpoint.map(|addr| addr.port()).unwrap_or(defaults.rocket_port),
            wg_device_config: config.wg_config_path.unwrap_or(defaults.wg_device_config),
            client_handshake_timeout: config
                .client_handshake_timeout_s
                .map(Duration::from_secs)
                .unwrap_or(defaults.client_handshake_timeout),
        }
    }
}
