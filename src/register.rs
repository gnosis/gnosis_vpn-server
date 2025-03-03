use serde::Serialize;
use std::collections::HashSet;
use std::net::Ipv4Addr;
use std::process::Command;

use crate::dump;
use crate::ops::Ops;

#[derive(Debug, Serialize)]
pub struct Register {
    pub ip: Ipv4Addr,
}

#[derive(Debug, Serialize)]
pub enum Error {
    NoDevice,
    NoFreeIp,
    Generic(String),
    Dump(dump::Error),
}

pub fn register(ops: &Ops, rng: &mut rand::rngs::ThreadRng, public_key: &str) -> Result<Register, Error> {
    let device = match ops.device() {
        Some(device) => device,
        None => return Err(Error::NoDevice),
    };
    let dump = dump::dump(device).map_err(Error::Dump)?;
    let res_peer = dump.peers.iter().find(|peer| peer.public_key == public_key);
    if let Some(peer) = res_peer {
        return Ok(Register { ip: peer.ip });
    }

    let existing_ips: HashSet<Ipv4Addr> = HashSet::from_iter(dump.peers.iter().map(|peer| peer.ip));
    let res_ip = ops.client_address_range.find_free_ip(&existing_ips, rng);
    let ip = match res_ip {
        Some(ip) => ip,
        None => return Err(Error::NoFreeIp),
    };

    let res_output = Command::new("wg")
        .arg("set")
        .arg(device)
        .arg("peer")
        .arg(public_key)
        .arg("allowed-ips")
        .arg(format!("{}/32", ip))
        .output();

    let output = match res_output {
        Ok(output) => output,
        Err(err) => {
            return Err(Error::Generic(format!(
                "wg set peer {} allowed-ips {}/32 failed: {}",
                public_key, ip, err
            )));
        }
    };

    if output.status.success() {
        Ok(Register { ip })
    } else {
        Err(Error::Generic(format!("wg add peer failed: {:?}", output)))
    }
}
