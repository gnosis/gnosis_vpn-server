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
    Generic(String),
    IOError(String),
}

pub fn set_interface(ops: &Ops) -> Result<(), Error> {
    let interface = match ops.interface() {
        Some(interface) => interface,
        None => return Err(Error::NoInterface),
    };

    let res_output = Command::new("wg")
        .arg("setconf")
        .arg(interface)
        .arg(&ops.wg_interface_config)
        .output();

    let output = match res_output {
        Ok(output) => output,
        Err(err) => {
            return Err(Error::Generic(format!(
                "wg setconf {} {:?} failed: {}",
                interface, &ops.wg_interface_config, err
            )));
        }
    };

    if !output.status.success() {
        return Err(Error::Generic(format!("wg setconf failed: {:?}", output)));
    }

    if !output.stderr.is_empty() {
        tracing::warn!("wg setconf stderr: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}

pub fn save_file(ops: &Ops) -> Result<(), Error> {
    let interface = match ops.interface() {
        Some(interface) => interface,
        None => return Err(Error::NoInterface),
    };

    let res_output = Command::new("wg").arg("showconf").arg(interface).output();

    let output = match res_output {
        Ok(output) => output,
        Err(err) => {
            return Err(Error::Generic(format!("wg showconf {} failed: {:?}", interface, err)));
        }
    };

    if !output.status.success() {
        return Err(Error::Generic(format!("wg showconf failed: {:?}", output)));
    }

    if !output.stderr.is_empty() {
        tracing::warn!("wg showconf stderr: {}", String::from_utf8_lossy(&output.stderr));
    }

    let mut f = File::create(&ops.wg_interface_config).map_err(|err| Error::IOError(err.to_string()))?;
    f.write_all(&output.stdout)
        .map_err(|err| Error::IOError(err.to_string()))?;

    Ok(())
}
