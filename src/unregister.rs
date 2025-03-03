use serde::Serialize;
use std::process::Command;

use crate::ops::Ops;

#[derive(Debug, Serialize)]
pub enum Error {
    NoDevice,
    Generic(String),
}

pub fn run(ops: &Ops, public_key: &str) -> Result<(), Error> {
    let device = match ops.device() {
        Some(device) => device,
        None => return Err(Error::NoDevice),
    };

    let res_output = Command::new("wg")
        .arg("set")
        .arg(device)
        .arg("peer")
        .arg(public_key)
        .arg("remove")
        .output();

    let output = match res_output {
        Ok(output) => output,
        Err(err) => {
            return Err(Error::Generic(format!(
                "wg set peer {} remove failed: {}",
                public_key, err
            )));
        }
    };

    if output.status.success() {
        Ok(())
    } else {
        Err(Error::Generic(format!("wg remove peer failed: {:?}", output)))
    }
}
