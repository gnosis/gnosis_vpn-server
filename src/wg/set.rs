use thiserror::Error;

use std::env;
use std::fs;
use std::net::Ipv4Addr;
use std::process::Command;

use crate::shell_command_ext::{self, ShellCommandExt};
use crate::wg::peer::Peer;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Command failed: {0}")]
    Command(#[from] shell_command_ext::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub fn add_peer(interface: &str, public_key: &str, ip: &Ipv4Addr) -> Result<String, Error> {
    let preshared_key = Command::new("wg").arg("genpsk").run_stdout()?;

    let tmp_file = env::temp_dir().join(format!(".{}.psk", public_key));
    fs::write(&tmp_file, preshared_key.clone()).map_err(|error| {
        tracing::error!(?error, file = %tmp_file.display(), "Failed to write temporary psk file");
        error
    })?;

    // add peer to interface
    Command::new("wg")
        .arg("set")
        .arg(interface)
        .arg("peer")
        .arg(public_key)
        .arg("preshared-key")
        .arg(&tmp_file)
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

    fs::remove_file(&tmp_file).map_err(|error| {
        tracing::error!(?error, file = %tmp_file.display(), "Failed to remove temporary psk file");
        error
    })?;

    Ok(preshared_key)
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
