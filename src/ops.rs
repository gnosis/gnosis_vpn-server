use std::fs;
use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;
use std::time::Duration;

use crate::config::Config;
use crate::ip_range::IpRange;

#[derive(Debug, Clone)]
pub struct Ops {
    pub client_address_range: IpRange,
    pub rocket_address: IpAddr,
    pub rocket_port: u16,
    pub wg_config: PathBuf,
    pub interface_name: String,
    pub client_handshake_timeout: Duration,
    pub client_cleanup_interval: Duration,
}

const DEFAULT_ROCKET_ADDRESS: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
const DEFAULT_ROCKET_PORT: u16 = 8000;
const DEFAULT_CLIENT_HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(5 * 60);
const DEFAULT_CLIENT_CLEANUP_INTERVAL: Duration = Duration::from_secs(3 * 60);

impl TryFrom<Config> for Ops {
    type Error = anyhow::Error;

    fn try_from(config: Config) -> Result<Self, Self::Error> {
        let rocket_address = config.endpoint.map(|addr| addr.ip()).unwrap_or(DEFAULT_ROCKET_ADDRESS);
        let rocket_port = config.endpoint.map(|addr| addr.port()).unwrap_or(DEFAULT_ROCKET_PORT);
        let wg_config = fs::canonicalize(&config.wireguard_config_path)?;
        let interface_name = wg_config
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .ok_or(anyhow::anyhow!("Invalid WireGuard interface name"))?;

        let client_handshake_timeout = config
            .client_handshake_timeout_s
            .map(Duration::from_secs)
            .unwrap_or(DEFAULT_CLIENT_HANDSHAKE_TIMEOUT);
        let client_cleanup_interval = config
            .client_cleanup_interval_s
            .map(Duration::from_secs)
            .unwrap_or(DEFAULT_CLIENT_CLEANUP_INTERVAL);
        Ok(Self {
            client_address_range: config.allowed_client_ips.clone(),
            rocket_address,
            rocket_port,
            wg_config,
            interface_name,
            client_handshake_timeout,
            client_cleanup_interval,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::ip_range::IpRange;

    fn sample_range() -> IpRange {
        toml::from_str(
            r#"start = "10.128.0.2"
end = "10.128.0.10""#,
        )
        .expect("range")
    }

    #[test]
    fn should_use_defaults_when_endpoint_and_timeouts_absent() -> anyhow::Result<()> {
        let config = Config {
            allowed_client_ips: sample_range(),
            endpoint: None,
            wireguard_config_path: PathBuf::from("wg0.conf"),
            client_handshake_timeout_s: None,
            client_cleanup_interval_s: None,
        };

        let ops: Ops = config.into();

        assert_eq!(ops.rocket_address, IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
        assert_eq!(ops.rocket_port, 8000);
        assert_eq!(ops.client_handshake_timeout, Duration::from_secs(300));
        assert_eq!(ops.client_cleanup_interval, Duration::from_secs(180));

        Ok(())
    }

    #[test]
    fn should_derive_interface_name_from_config_filename() -> anyhow::Result<()> {
        let ops = Ops {
            client_address_range: sample_range(),
            rocket_address: IpAddr::V4(Ipv4Addr::new(192, 168, 0, 1)),
            rocket_port: 9000,
            wg_interface_config: PathBuf::from("/etc/wireguard/custom0.conf"),
            client_handshake_timeout: Duration::from_secs(100),
            client_cleanup_interval: Duration::from_secs(200),
        };

        assert_eq!(ops.interface(), Some("custom0"));

        Ok(())
    }
}
