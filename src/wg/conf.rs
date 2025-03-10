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
    IO(String),
}

pub fn set_interface(ops: &Ops) -> Result<(), Error> {
    let interface = match ops.interface() {
        Some(interface) => interface,
        None => return Err(Error::NoInterface),
    };

    // 1. create device using ip link
    let res_iplink = Command::new("ip")
        .arg("link")
        .arg("add")
        .arg(interface)
        .arg("type")
        .arg("wireguard")
        .output();

    let output_iplink = match res_iplink {
        Ok(output) => output,
        Err(err) => {
            return Err(Error::IO(format!(
                "ip link add {} type wireguard failed: {:?}",
                interface, err
            )));
        }
    };

    if !output_iplink.status.success() {
        return Err(Error::Generic(format!("ip link add failed: {:?}", output_iplink)));
    }

    if !output_iplink.stderr.is_empty() {
        tracing::warn!(
            stderr = String::from_utf8_lossy(&output_iplink.stderr).to_string(),
            "ip link add"
        )
    }

    // 2. apply existing configuration using wg setconf
    let res_setconf = Command::new("wg")
        .arg("setconf")
        .arg(interface)
        .arg(&ops.wg_interface_config)
        .output();

    let output_setconf = match res_setconf {
        Ok(output) => output,
        Err(err) => {
            return Err(Error::IO(format!("wg setconf {} failed: {}", interface, err)));
        }
    };

    if !output_setconf.status.success() {
        return Err(Error::Generic(format!("wg setconf failed: {:?}", output_setconf)));
    }

    if !output_setconf.stderr.is_empty() {
        tracing::warn!(
            stderr = String::from_utf8_lossy(&output_setconf.stderr).to_string(),
            "wg setconf"
        )
    }

    // 3. set recommended MTU 1420
    let res_mtu = Command::new("ip")
        .arg("link")
        .arg("set")
        .arg("mtu")
        .arg("1420")
        .arg("up")
        .arg("dev")
        .arg(interface)
        .output();

    match res_mtu {
        Ok(output) => {
            if !output.status.success() {
                tracing::error!(?output, "ip link set mtu 1420 up dev")
            }

            if !output.stderr.is_empty() {
                tracing::warn!(
                    stderr = String::from_utf8_lossy(&output.stderr).to_string(),
                    "ip link set mtu 1420 up dev"
                )
            }
        }
        Err(err) => {
            tracing::error!(?err, interface, "ip link set mtu 1420 up dev")
        }
    };

    Ok(())
}

pub fn remove_interface(ops: &Ops) -> Result<(), Error> {
    let interface = match ops.interface() {
        Some(interface) => interface,
        None => return Err(Error::NoInterface),
    };

    let res_iplink = Command::new("ip")
        .arg("link")
        .arg("delete")
        .arg("dev")
        .arg(interface)
        .output();

    let output_iplink = match res_iplink {
        Ok(output) => output,
        Err(err) => {
            return Err(Error::IO(format!("ip link delete dev {} failed: {:?}", interface, err)));
        }
    };

    if !output_iplink.status.success() {
        return Err(Error::Generic(format!("ip link delete failed: {:?}", output_iplink)));
    }

    if !output_iplink.stderr.is_empty() {
        tracing::warn!(
            "ip link delete stderr: {}",
            String::from_utf8_lossy(&output_iplink.stderr)
        );
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

    let prepend_str = format!("Maintained by {}\n", env!("CARGO_PKG_NAME"));
    let prepend = prepend_str.as_bytes();
    let mut content = Vec::with_capacity(prepend.len() + output.stdout.len());
    content.extend_from_slice(prepend);
    content.extend_from_slice(&output.stdout);
    let mut f = File::create(&ops.wg_interface_config).map_err(|err| Error::IO(err.to_string()))?;
    f.write_all(&content).map_err(|err| Error::IO(err.to_string()))?;
    Ok(())
}
