use std::net::Ipv4Addr;
use std::process::Command;
use thiserror::Error;

use crate::shell_command_ext::{self, ShellCommandExt};
use crate::wg::peer::Peer;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Command failed: {0}")]
    Command(#[from] shell_command_ext::Error),
}

pub fn add_peer(interface: &str, public_key: &str, ip: &Ipv4Addr) -> Result<(), Error> {
    // add peer to interface
    Command::new("wg")
        .arg("set")
        .arg(interface)
        .arg("peer")
        .arg(public_key)
        .arg("allowed-ips")
        .arg(format!("{ip}/32"))
        .run()?;

    // add routing for added peer
    Command::new("ip")
        .arg("-4")
        .arg("route")
        .arg("add")
        .arg(format!("{ip}/32"))
        .arg("dev")
        .arg(interface)
        .run()?;

    Ok(())
}

pub fn remove_peer(interface: &str, peer: &Peer) -> Result<(), Error> {
    // remove peer from interface
    Command::new("wg")
        .arg("set")
        .arg(interface)
        .arg("peer")
        .arg(peer.public_key.clone())
        .arg("remove")
        .run()?;

    // delete routing for removed peer
    Command::new("ip")
        .arg("-4")
        .arg("route")
        .arg("del")
        .arg(peer.ip.to_string())
        .run()?;

    Ok(())
}
