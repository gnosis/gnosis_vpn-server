use thiserror::Error;

use std::io::Error as IOError;
use std::net::Ipv4Addr;
use std::process::Command;

use crate::wg::peer::Peer;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Generic error: {0}")]
    Generic(String),
    #[error("IO error: {0}")]
    IO(#[from] IOError),
}

pub fn add_peer(interface: &str, public_key: &str, ip: &Ipv4Addr) -> Result<(), Error> {
    let output_set = Command::new("wg")
        .arg("set")
        .arg(interface)
        .arg("peer")
        .arg(public_key)
        .arg("allowed-ips")
        .arg(format!("{}/32", ip))
        .output()?;

    if !output_set.stderr.is_empty() {
        tracing::warn!(
            stderr = String::from_utf8_lossy(&output_set.stderr).to_string(),
            interface,
            ?ip,
            "wg set peer"
        );
    }

    if !output_set.status.success() {
        return Err(Error::Generic(format!("wg set peer failed: {:?}", output_set)));
    }

    let output_route = Command::new("ip")
        .arg("-4")
        .arg("route")
        .arg("add")
        .arg(format!("{}/32", ip))
        .arg("dev")
        .arg(interface)
        .output()?;

    if !output_route.stderr.is_empty() {
        tracing::warn!(
            stderr = String::from_utf8_lossy(&output_route.stderr).to_string(),
            interface,
            ?ip,
            "ip route add"
        );
    }

    if !output_route.status.success() {
        return Err(Error::Generic(format!("ip route add failed: {:?}", output_route)));
    }

    Ok(())
}

pub fn remove_peer(interface: &str, peer: &Peer) -> Result<(), Error> {
    let output_set = Command::new("wg")
        .arg("set")
        .arg(interface)
        .arg("peer")
        .arg(peer.public_key.clone())
        .arg("remove")
        .output()?;

    if !output_set.stderr.is_empty() {
        tracing::warn!(
            stderr = String::from_utf8_lossy(&output_set.stderr).to_string(),
            interface,
            "wg set peer"
        )
    }

    if !output_set.status.success() {
        return Err(Error::Generic(format!("wg remove peer failed: {:?}", output_set)));
    }

    let output_route = Command::new("ip")
        .arg("-4")
        .arg("route")
        .arg("del")
        .arg(peer.ip.to_string())
        .output()?;

    if !output_route.stderr.is_empty() {
        tracing::warn!(
            stderr = String::from_utf8_lossy(&output_route.stderr).to_string(),
            interface,
            ?peer,
            "ip route del"
        );
    }

    if !output_route.status.success() {
        return Err(Error::Generic(format!("ip route del failed: {:?}", output_route)));
    }

    Ok(())
}
