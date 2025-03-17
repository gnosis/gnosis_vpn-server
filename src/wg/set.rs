use serde::Serialize;
use std::net::Ipv4Addr;
use std::process::Command;

#[derive(Debug, Serialize)]
pub enum Error {
    IO(String),
}

pub fn add_peer(interface: &str, public_key: &str, ip: &Ipv4Addr) -> Result<(), Error> {
    let output = Command::new("wg")
        .arg("set")
        .arg(interface)
        .arg("peer")
        .arg(public_key)
        .arg("allowed-ips")
        .arg(format!("{}/32", ip))
        .output()
        .map_err(|err| {
            Error::IO(format!(
                "wg set {} peer {} allowed-ips {}/32 failed: {:?}",
                interface, public_key, ip, err
            ))
        })?;

    if !output.stderr.is_empty() {
        tracing::warn!(
            stderr = String::from_utf8_lossy(&output.stderr).to_string(),
            interface,
            ?ip,
            "wg set peer"
        );
    }

    Ok(())
}

pub fn remove_peer(interface: &str, public_key: &str) -> Result<(), Error> {
    let output = Command::new("wg")
        .arg("set")
        .arg(interface)
        .arg("peer")
        .arg(public_key)
        .arg("remove")
        .output()
        .map_err(|err| {
            Error::IO(format!(
                "wg set {} peer {} remove failed: {:?}",
                interface, public_key, err
            ))
        })?;

    if !output.stderr.is_empty() {
        tracing::warn!(
            stderr = String::from_utf8_lossy(&output.stderr).to_string(),
            interface,
            "wg set peer"
        )
    }

    Ok(())
}
