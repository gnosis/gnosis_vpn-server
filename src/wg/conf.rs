use thiserror::Error;

use std::fs::File;
use std::io::Error as IOError;
use std::io::Write;
use std::process::Command;

use crate::ops::Ops;
use crate::shell_command_ext::{self, ShellCommandExt};
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

#[derive(Debug, Error)]
pub enum Error {
    #[error("Generic error: {0}")]
    Generic(String),
    #[error(transparent)]
    IO(#[from] IOError),
    #[error("No interface found")]
    NoInterface,
    #[error("Failed parsing interface address")]
    NoAddress,
    #[error(transparent)]
    Command(#[from] shell_command_ext::Error),
}

pub fn save_file(ops: &Ops) -> Result<(), Error> {
    let ip_addr_stdout = Command::new("ip")
        .arg("-f")
        .arg("inet")
        .arg("addr")
        .arg("show")
        .arg(ops.interface_name)
        .run_stdout()?;

    let wg_stdout = Command::new("wg")
        .arg("showconf")
        .arg(ops.interface_name)
        .run_stdout()?;

    // Prepend with maintainer information
    let prepend_str = format!("# Maintained by {}\n\n", env!("CARGO_PKG_NAME"));
    let prepend = prepend_str.as_bytes();

    let interface_address = ip_addr_stdout
        .split('\n')
        .find(|line| line.contains("inet "))
        .and_then(|line| line.trim().split(' ').nth(1))
        .ok_or_else(|| {
            tracing::error!(ops.interface_name, stdout = ?ip_addr_stdout, "Failed to parse address");
            Error::NoAddress
        })?;

    let mut lines: Vec<String> = wg_stdout.lines().map(String::from).collect();

    // Add interface address into the config
    if let Some(index) = lines.iter().position(|line| line == "[Interface]") {
        let line_addr = format!("Address = {interface_address}");
        lines.insert(index + 1, line_addr);
    }

    let modified_output = lines.join("\n");
    let modified_output_bytes = modified_output.as_bytes();

    let mut content = Vec::with_capacity(prepend.len() + modified_output_bytes.len());
    content.extend_from_slice(prepend);
    content.extend_from_slice(modified_output_bytes);
    let mut f = File::create(&ops.wg_config)?;
    f.write_all(&content)?;
    Ok(())
}
