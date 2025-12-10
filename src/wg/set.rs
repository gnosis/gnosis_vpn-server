use std::net::Ipv4Addr;
use std::process::Command;

use crate::shell_command_ext::{self, ShellCommandExt};
use crate::wg::peer::Peer;

pub type Error = shell_command_ext::Error;

pub fn add_peer(interface: &str, public_key: &str, ip: &Ipv4Addr) -> Result<(), shell_command_ext::Error> {
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

pub fn remove_peer(interface: &str, peer: &Peer) -> Result<(), shell_command_ext::Error> {
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
