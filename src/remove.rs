use serde::Serialize;
use std::collections::HashSet;
use std::time::Duration;

use crate::ops::Ops;
use crate::unregister;
use crate::wg::show;

#[derive(Debug, Serialize)]
pub struct RemoveExpired {
    pub expired_public_keys: Vec<String>,
    pub total: u32,
}

#[derive(Debug, Serialize)]
pub struct RemoveNeverConnected {
    pub never_connected_public_keys: Vec<String>,
    pub total: u32,
}

#[derive(Debug, Serialize)]
pub struct RemoveDisconnected {
    pub newly_found: Vec<String>,
    pub removed: Vec<String>,
}

#[derive(Debug, Serialize)]
pub enum Error {
    NoInterface,
    WgShow(show::Error),
    Unregister(unregister::Error),
    SystemTime(String),
}

pub fn previously_disconnected(ops: &Ops, once_not_connected: &[String]) -> Result<RemoveDisconnected, Error> {
    // determine never connected
    let interface = match ops.interface() {
        Some(interface) => interface,
        None => return Err(Error::NoInterface),
    };
    let dump = show::dump(interface).map_err(Error::WgShow)?;
    let public_keys = dump
        .peers
        .iter()
        .filter(|peer| !peer.has_handshaked())
        .map(|peer| &peer.public_key)
        .collect::<Vec<&String>>();

    let newly_found: HashSet<&String> = public_keys.into_iter().collect();
    let existing: HashSet<&String> = once_not_connected.iter().collect();

    // restrict to only removing peers that were not connected during last run
    let removable: HashSet<&String> = existing.intersection(&newly_found).copied().collect();
    for key in &removable {
        let _ = unregister::run(ops, key).map_err(Error::Unregister)?;
    }

    let remaining: Vec<String> = newly_found.difference(&removable).map(|&key| key.clone()).collect();
    Ok(RemoveDisconnected {
        newly_found: remaining,
        removed: removable.into_iter().map(|s| s.to_string()).collect(),
    })
}

pub fn expired(ops: &Ops, overwrite_client_handshake_timeout_s: &Option<u64>) -> Result<RemoveExpired, Error> {
    let interface = match ops.interface() {
        Some(interface) => interface,
        None => return Err(Error::NoInterface),
    };
    let client_handshake_timeout = overwrite_client_handshake_timeout_s
        .map(Duration::from_secs)
        .unwrap_or(ops.client_handshake_timeout);
    let dump = show::dump(interface).map_err(Error::WgShow)?;
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
            false
        })
        .map(|(public_key, _)| public_key)
        .collect::<Vec<&String>>();

    for key in &public_keys {
        let _ = unregister::run(ops, key).map_err(Error::Unregister)?;
    }

    Ok(RemoveExpired {
        expired_public_keys: public_keys.iter().map(|s| s.to_string()).collect(),
        total: public_keys.len() as u32,
    })
}

pub fn never_connected(ops: &Ops) -> Result<RemoveNeverConnected, Error> {
    let interface = match ops.interface() {
        Some(interface) => interface,
        None => return Err(Error::NoInterface),
    };
    let dump = show::dump(interface).map_err(Error::WgShow)?;
    let public_keys = dump
        .peers
        .iter()
        .filter(|peer| !peer.has_handshaked())
        .map(|peer| &peer.public_key)
        .collect::<Vec<&String>>();
    for key in &public_keys {
        let _ = unregister::run(ops, key).map_err(Error::Unregister)?;
    }
    Ok(RemoveNeverConnected {
        never_connected_public_keys: public_keys.iter().map(|s| s.to_string()).collect(),
        total: public_keys.len() as u32,
    })
}
