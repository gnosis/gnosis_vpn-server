use serde::Serialize;
use std::net::{Ipv4Addr, SocketAddr};
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
    NoDevice,
    Generic(String),
    NoOutputLines,
    WrongNumberOfFieldsInServerLine,
    WrongNumberOfFieldsInPeerLine,
}

pub fn set_device(ops: &Ops) -> Result<(), Error> {
    let res_output = Command::new("wg-quick").arg("up").arg(ops.wg_device_config).output();

    let output = match res_output {
        Ok(output) => output,
        Err(err) => {
            return Err(Error::Generic(format!("wg-quick up for {} failed: {}", device, err)));
        }
    };

    if !output.status.success() {
        return Err(Error::Generic(format!("wg-quick up failed: {:?}", output)));
    }

    if !output.stderr.is_empty() {
        tracing::warn!("wg-quick up stderr: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}

pub fn save_file(ops: &Ops) -> Result<(), Error> {
    let res_output = Command::new("wg-quick").arg("down").arg(ops.wg_device_config).output();

    let output = match res_output {
        Ok(output) => output,
        Err(err) => {
            return Err(Error::Generic(format!("wg-quick down for {} failed: {}", device, err)));
        }
    };

    if !output.status.success() {
        return Err(Error::Generic(format!("wg-quick down failed: {:?}", output)));
    }

    if !output.stderr.is_empty() {
        tracing::warn!("wg-quick down stderr: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}
