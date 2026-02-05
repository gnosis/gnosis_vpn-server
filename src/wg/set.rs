use thiserror::Error;

use std::env;
use std::fs;
use std::io;
use std::net::Ipv4Addr;
use std::path::PathBuf;
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

struct TmpFile {
    path: PathBuf,
}

impl TmpFile {
    fn create(ip: &Ipv4Addr, preshared_key: &str) -> Result<Self, io::Error> {
        let path = env::temp_dir().join(format!(".{}.psk", ip));
        fs::write(&path, preshared_key).map_err(|error| {
            tracing::error!(?error, file = %path.display(), "Failed to write temporary psk file");
            error
        })?;
        Ok(TmpFile { path })
    }

    fn path(&self) -> String {
        self.path.to_string_lossy().to_string()
    }
}

impl Drop for TmpFile {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_file(&self.path) {
            tracing::error!(?error, file = %self.path.display(), "Failed to remove temporary file");
        }
    }
}

pub fn add_peer(interface: &str, public_key: &str, ip: &Ipv4Addr) -> Result<String, Error> {
    let preshared_key = Command::new("wg").arg("genpsk").run_stdout()?;
    let tmp_file = TmpFile::create(ip, &preshared_key)?;

    // add peer to interface
    Command::new("wg")
        .arg("set")
        .arg(interface)
        .arg("peer")
        .arg(public_key)
        .arg("preshared-key")
        .arg(tmp_file.path())
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
