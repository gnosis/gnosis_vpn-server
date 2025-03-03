use serde::Serialize;
use std::process::Command;

use crate::dump::Peer;
use crate::ops::Ops;
use crate::unregister;

#[derive(Debug, Serialize)]
pub struct RemoveExpired {
    Removed: u32,
}

#[derive(Debug, Serialize)]
pub enum Error {
    NoDevice,
    Generic(String),
}

pub fn run(ops: &Ops, client_handshake_timeout_s: &Option<u64>) -> Result<RemoveExpired, Error> {
    let device = match ops.device() {
        Some(device) => device,
        None => return Err(Error::NoDevice),
    };
    let client_handshake_timeout = client_handshake_timeout_s.or(ops.client_handshake_timeout);
    let dump = dump::dump(device).map_err(Error::Dump)?;
    let peers = dump
        .peers
        .iter()
        .filter(|peer| peer.has_handshaked() && peer.timed_out(client_handshake_timeout))
        .collect::<Vec<&Peer>>();
    for p in peers {
        unregster::run(ops, p.public_key).map_err(Error::Generic)?
    }
    Ok(RemoveExpired {
        Removed: peers.len() as u32,
    })
}
