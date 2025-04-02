use serde::Serialize;
use std::fs::File;
use std::io::Write;
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
    NoAddress,
    Generic(String),
    IO(String),
}

pub fn save_file(ops: &Ops) -> Result<(), Error> {
    let interface = match ops.interface() {
        Some(interface) => interface,
        None => return Err(Error::NoInterface),
    };

    let output_ip_addr = Command::new("ip")
        .arg("-f")
        .arg("inet")
        .arg("addr")
        .arg("show")
        .arg(interface)
        .output()
        .map_err(|err| Error::IO(format!("ip -f inet addr show {} failed: {:?}", interface, err)))?;

    if !output_ip_addr.status.success() {
        return Err(Error::Generic(format!(
            "ip -f inet addr show failed: {:?}",
            output_ip_addr
        )));
    }

    if !output_ip_addr.stderr.is_empty() {
        tracing::warn!(
            stderr = String::from_utf8_lossy(&output_ip_addr.stderr).to_string(),
            interface,
            "ip -f inet addr show"
        )
    }

    let output_wg = Command::new("wg")
        .arg("showconf")
        .arg(interface)
        .output()
        .map_err(|err| Error::IO(format!("wg showconf {} failed: {:?}", interface, err)))?;

    if !output_wg.status.success() {
        return Err(Error::Generic(format!("wg showconf failed: {:?}", output_wg)));
    }

    if !output_wg.stderr.is_empty() {
        tracing::warn!(
            stderr = String::from_utf8_lossy(&output_wg.stderr).to_string(),
            interface,
            "wg showconf"
        )
    }

    // Prepend with maintainer information
    let prepend_str = format!("# Maintained by {}\n\n", env!("CARGO_PKG_NAME"));
    let prepend = prepend_str.as_bytes();

    let ip_addr_stdout = String::from_utf8_lossy(&output_ip_addr.stdout);

    let interface_address = ip_addr_stdout
        .split('\n')
        .find(|line| line.contains("inet "))
        .and_then(|line| line.trim().split(' ').nth(1))
        .ok_or_else(|| {
            tracing::error!(?interface, stdout = ?ip_addr_stdout, "Failed to parse address");
            Error::NoAddress
        })?;

    let stdout_str = String::from_utf8_lossy(&output_wg.stdout);
    let mut lines: Vec<String> = stdout_str.lines().map(String::from).collect();

    // Add interface address into the config
    if let Some(index) = lines.iter().position(|line| line == "[Interface]") {
        let line_addr = format!("Address = {}", interface_address);
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
