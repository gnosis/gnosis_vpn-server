use std::ffi::OsStr;
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
    pub wg_interface_config: PathBuf,
    pub client_handshake_timeout: Duration,
    pub client_cleanup_interval: Duration,
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
        }
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
