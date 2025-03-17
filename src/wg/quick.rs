use serde::Serialize;
use std::process::Command;

use crate::ops::Ops;

#[derive(Debug, Serialize)]
pub enum Error {
    Generic(String),
    IO(String),
}

pub fn up(ops: &Ops) -> Result<(), Error> {
    let interface_file = ops.wg_interface_config.clone();
    let output = Command::new("wg-quick")
        .arg("up")
        .arg(interface_file.clone())
        .output()
        .map_err(|err| Error::IO(format!("wg-quick up {:?} failed: {:?}", interface_file, err)))?;

    if !output.status.success() {
        return Err(Error::Generic(format!("wg-quick up failed: {:?}", output)));
    }

    if !output.stderr.is_empty() {
        tracing::warn!(
            stderr = String::from_utf8_lossy(&output.stderr).to_string(),
            ?interface_file,
            "wg-quick up"
        )
    }

    Ok(())
}

pub fn down(ops: &Ops) -> Result<(), Error> {
    let interface_file = ops.wg_interface_config.clone();
    let output = Command::new("wg-quick")
        .arg("down")
        .arg(interface_file.clone())
        .output()
        .map_err(|err| Error::IO(format!("wg-quick down {:?} failed: {:?}", interface_file, err)))?;

    if !output.status.success() {
        return Err(Error::Generic(format!("wg-quick down failed: {:?}", output)));
    }

    if !output.stderr.is_empty() {
        tracing::warn!(
            stderr = String::from_utf8_lossy(&output.stderr).to_string(),
            ?interface_file,
            "wg-quick down"
        )
    }

    Ok(())
}
