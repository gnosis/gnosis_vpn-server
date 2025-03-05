use serde::Serialize;
use std::time::Duration;

use crate::dump;
use crate::ops::Ops;
use crate::unregister;

#[derive(Debug, Serialize)]
pub struct RemoveExpired {
    RemovedPublicKeys: Vec<String>,
    Total: u32,
}

#[derive(Debug, Serialize)]
pub struct RemoveNeverConnected {
    RemovedPublicKeys: Vec<String>,
    Total: u32,
}

#[derive(Debug, Serialize)]
pub enum Error {
    NoDevice,
    Generic(String),
    Dump(dump::Error),
    Unregister(unregister::Error),
    SystemTime(String),
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
    let (hand_shaked_peers, bad_peers) = dump
        .peers
        .iter()
        .filter(|peer| peer.has_handshaked())
        .map(|peer| (peer.public_key.clone(), peer.timed_out(&client_handshake_timeout)))
        .partition::<Vec<_>, _>(|(_, res_timed_out)| res_timed_out.is_ok());

    // fail if any system time error occured
    for (_, err) in bad_peers {
        if let Err(err) = err {
            return Err(Error::SystemTime(err.to_string()));
        }
    }

    let public_keys = hand_shaked_peers
        .iter()
        .filter(|(_, res_timed_out)| {
            if let Ok(timed_out) = res_timed_out {
                return *timed_out;
            }
            return false;
        })
        .map(|(public_key, _)| public_key)
        .collect::<Vec<&String>>();

    for key in &public_keys {
        unregister::run(ops, &key).map_err(Error::Unregister)?
    }

    Ok(RemoveExpired {
        RemovedPublicKeys: public_keys.iter().map(|s| s.to_string()).collect(),
        Total: public_keys.len() as u32,
    })
}

pub fn never_connected(ops: &Ops) -> Result<RemoveNeverConnected, Error> {
    let device = match ops.device() {
        Some(device) => device,
        None => return Err(Error::NoDevice),
    };
    let dump = dump::run(device).map_err(Error::Dump)?;
    let public_keys = dump
        .peers
        .iter()
        .filter(|peer| !peer.has_handshaked())
        .map(|peer| &peer.public_key)
        .collect::<Vec<&String>>();
    for key in &public_keys {
        unregister::run(ops, &key).map_err(Error::Unregister)?
    }
    Ok(RemoveNeverConnected {
        RemovedPublicKeys: public_keys.iter().map(|s| s.to_string()).collect(),
        Total: public_keys.len() as u32,
    })
}
