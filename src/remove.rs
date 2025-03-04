use serde::Serialize;
use std::time::{Duration, SystemTimeError};

use crate::dump;
use crate::dump::Peer;
use crate::ops::Ops;
use crate::unregister;

#[derive(Debug, Serialize)]
pub struct RemoveExpired {
    Removed: u32,
}

#[derive(Debug, Serialize)]
pub struct RemoveNeverConnected {
    Removed: u32,
}

#[derive(Debug, Serialize)]
pub enum Error {
    NoDevice,
    Generic(String),
    Dump(dump::Error),
    Unregister(unregister::Error),
}

pub fn expired(ops: &Ops, client_handshake_timeout_s: &Option<u64>) -> Result<RemoveExpired, Error> {
    let device = match ops.device() {
        Some(device) => device,
        None => return Err(Error::NoDevice),
    };
    let client_handshake_timeout = client_handshake_timeout_s
        .map(Duration::from_secs)
        .unwrap_or(ops.client_handshake_timeout);
    let dump = dump::run(device).map_err(Error::Dump)?;
    let (good_peers, bad_peers) = dump
        .peers
        .iter()
        .map(|peer| (peer.has_handshaked(), peer.timed_out(&client_handshake_timeout)))
        .partition::<Vec<_>, _>(|(_good, bad)| bad.is_ok());

    for p in &peers {
        unregister::run(ops, &p.public_key).map_err(Error::Unregister)?
    }
    Ok(RemoveExpired {
        Removed: peers.len() as u32,
    })
}

pub fn never_connected(ops: &Ops) -> Result<RemoveNeverConnected, Error> {
    let device = match ops.device() {
        Some(device) => device,
        None => return Err(Error::NoDevice),
    };
    let dump = dump::run(device).map_err(Error::Dump)?;
    let peers = dump
        .peers
        .iter()
        .filter(|peer| !peer.has_handshaked())
        .collect::<Vec<&Peer>>();
    for p in &peers {
        unregister::run(ops, &p.public_key).map_err(Error::Unregister)?
    }
    Ok(RemoveNeverConnected {
        Removed: peers.len() as u32,
    })
}
