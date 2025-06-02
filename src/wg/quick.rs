use thiserror::Error;

use std::io::Error as IOError;
use std::process::Command;

use crate::ops::Ops;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Generic error: {0}")]
    Generic(String),
    #[error("IO error: {0}")]
    IO(#[from] IOError),
}

pub fn up(ops: &Ops) -> Result<(), Error> {
    let interface_file = ops.wg_interface_config.clone();
    let output = Command::new("wg-quick")
        .arg("up")
        .arg(interface_file.clone())
        .output()?;

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
        .output()?;

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
