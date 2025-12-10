use anyhow::Context;

use std::fs;
use std::net::{IpAddr, Ipv4Addr};
use std::path::{Path, PathBuf};
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
    pub interface_section: Vec<String>,
    pub client_handshake_timeout: Duration,
    pub client_cleanup_interval: Duration,
}

const DEFAULT_ROCKET_ADDRESS: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
const DEFAULT_ROCKET_PORT: u16 = 8000;
const DEFAULT_CLIENT_HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(5 * 60);
const DEFAULT_CLIENT_CLEANUP_INTERVAL: Duration = Duration::from_secs(3 * 60);

impl Ops {
    pub fn from_config(config: Config, config_path: &Path) -> Result<Self, anyhow::Error> {
        let rocket_address = config.endpoint.map(|addr| addr.ip()).unwrap_or(DEFAULT_ROCKET_ADDRESS);
        let rocket_port = config.endpoint.map(|addr| addr.port()).unwrap_or(DEFAULT_ROCKET_PORT);
        let config_parent = config_path.parent().ok_or(anyhow::anyhow!(
            "Config path has no parent directory: {:?}",
            config_path
        ))?;
        let wg_config_path = config_parent.join(&config.wireguard_config_path);
        let wg_config = fs::canonicalize(wg_config_path).context("Canonicalizing WireGuard config path")?;
        let interface_name = wg_config
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .ok_or(anyhow::anyhow!("Invalid WireGuard interface name"))?;

        let content = fs::read_to_string(&wg_config).context("Reading WireGuard config file")?;
        let interface_section = extract_interface_section(content)?;

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
            interface_section,
            client_handshake_timeout,
            client_cleanup_interval,
        })
    }
}

fn extract_interface_section(content: String) -> Result<Vec<String>, anyhow::Error> {
    let mut lines = Vec::new();
    let mut in_interface_section = false;

    // minimal fields needed
    let mut found_private_key = false;
    let mut found_address = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            if in_interface_section {
                break;
            }
            if trimmed == "[Interface]" {
                in_interface_section = true;
            }
        } else if in_interface_section {
            lines.push(line.to_string());
            if line.starts_with("PrivateKey =") {
                found_private_key = true;
            } else if line.starts_with("Address =") {
                found_address = true;
            }
        }
    }

    if !found_private_key {
        return Err(anyhow::anyhow!("Missing PrivateKey in [Interface] section"));
    }

    if !found_address {
        return Err(anyhow::anyhow!("Missing Address in [Interface] section"));
    }

    if lines.is_empty() {
        Err(anyhow::anyhow!("No [Interface] section found in WireGuard config"))
    } else {
        Ok(lines)
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

    #[test]
    fn should_extract_interface_section() -> anyhow::Result<()> {
        let content = r#"
        [Interface]
        Address = 10.128.0.0/32
        PrivateKey = someprivatekey
        [Peer]
        PublicKey = somepublickey
        AllowedIPs = 10.128.0.120/32
        [Peer]
        PublicKey = anotherpublickey
        AllowedIPs = 10.128.0.122/32
        "#;
        let section = extract_interface_section(content.to_string())?;
        assert_eq!(section.len(), 2);
        assert!(section.iter().any(|line| line.contains("PrivateKey")));
        assert!(section.iter().any(|line| line.contains("Address")));
        Ok(())
    }
}
