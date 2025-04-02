use serde::Serialize;
use std::fs::File;
use std::io::Write;
use std::net::Ipv4Addr;
use std::process::Command;

use crate::ops::Ops;
use crate::wg::peer::Peer;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Dump {
    private_key: String,
    public_key: String,
    listen_port: u16,
    fwmark: String,
    pub peers: Vec<Peer>,
}

#[derive(Debug, Serialize)]
pub enum Error {
    NoInterface,
    Generic(String),
    IO(String),
}

pub fn save_file(ops: &Ops, address: &Ipv4Addr) -> Result<(), Error> {
    let interface = match ops.interface() {
        Some(interface) => interface,
        None => return Err(Error::NoInterface),
    };

    let output = Command::new("wg")
        .arg("showconf")
        .arg(interface)
        .output()
        .map_err(|err| Error::IO(format!("wg showconf {} failed: {:?}", interface, err)))?;

    if !output.status.success() {
        return Err(Error::Generic(format!("wg showconf failed: {:?}", output)));
    }

    if !output.stderr.is_empty() {
        tracing::warn!(
            stderr = String::from_utf8_lossy(&output.stderr).to_string(),
            interface,
            "wg showconf"
        )
    }

    // Prepend with maintainer information
    let prepend_str = format!("# Maintained by {}\n\n", env!("CARGO_PKG_NAME"));
    let prepend = prepend_str.as_bytes();

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let mut lines: Vec<String> = stdout_str.lines().map(String::from).collect();

    // Add interface address into the config
    if let Some(index) = lines.iter().position(|line| line == "[Interface]") {
        let line_addr = format!("Address = {}/32", address);
        lines.insert(index + 1, line_addr);
    }

    let modified_output = lines.join("\n");
    let modified_output_bytes = modified_output.as_bytes();

    let mut content = Vec::with_capacity(prepend.len() + modified_output_bytes.len());
    content.extend_from_slice(prepend);
    content.extend_from_slice(modified_output_bytes);
    let mut f = File::create(&ops.wg_interface_config).map_err(|err| Error::IO(err.to_string()))?;
    f.write_all(&content).map_err(|err| Error::IO(err.to_string()))?;
    Ok(())
}
