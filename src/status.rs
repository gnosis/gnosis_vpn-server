use serde::Serialize;

use crate::dump;
use crate::dump::Peer;
use crate::ops::Ops;

#[derive(Debug, Serialize)]
pub struct Status {
    pub slots: IpSlots,
    pub public_keys: PublicKeys,
    pub clients_outside_of_slots_range: u32,
}

#[derive(Debug, Serialize)]
pub struct IpSlots {
    total: u32,
    free: u32,
    healthy: u32,
    expired: u32,
    neverConnected: u32,
}

#[derive(Debug, Serialize)]
pub struct PublicKeys {
    healthy: Vec<String>,
    expired: Vec<String>,
    neverConnected: Vec<String>,
}

#[derive(Debug, Serialize)]
pub enum Error {
    NoDevice,
    Dump(dump::Error),
    SystemTime(String),
}

pub fn run(ops: &Ops) -> Result<Status, Error> {
    let device = match ops.device() {
        Some(device) => device,
        None => return Err(Error::NoDevice),
    };
    let dump = dump::run(device).map_err(Error::Dump)?;

    let (inside, outside): (Vec<&Peer>, Vec<&Peer>) = dump
        .peers
        .iter()
        .partition(|peer| ops.client_address_range.contains(peer.ip));

    let (handshaked_peers, never_connected_peers): (Vec<&Peer>, Vec<&Peer>) =
        inside.iter().partition(|peer| peer.has_handshaked());

    let (good_handshaked_public_keys, bad_handshaked_public_keys) = handshaked_peers
        .iter()
        .map(|peer| (peer.public_key.clone(), peer.timed_out(&ops.client_handshake_timeout)))
        .partition::<Vec<_>, _>(|(_, res_timed_out)| res_timed_out.is_ok());

    // fail if any system time error occured
    for (_, err) in bad_handshaked_public_keys {
        if let Err(err) = err {
            return Err(Error::SystemTime(err.to_string()));
        }
    }

    let (healthy_good_public_keys, expired_good_public_keys) = good_handshaked_public_keys
        .iter()
        .partition::<Vec<_>, _>(|(_, res_timed_out)| {
            if let Ok(timed_out) = res_timed_out {
                return !*timed_out;
            }
            return false;
        });

    let total = ops.client_address_range.count();
    let slots = IpSlots {
        total,
        free: total - inside.len() as u32,
        healthy: healthy_good_public_keys.len() as u32,
        expired: expired_good_public_keys.len() as u32,
        neverConnected: never_connected_peers.len() as u32,
    };

    let public_keys = PublicKeys {
        healthy: healthy_good_public_keys
            .iter()
            .map(|(public_key, _)| public_key.clone())
            .collect(),
        expired: expired_good_public_keys
            .iter()
            .map(|(public_key, _)| public_key.clone())
            .collect(),
        neverConnected: never_connected_peers
            .iter()
            .map(|peer| peer.public_key.clone())
            .collect(),
    };

    Ok(Status {
        slots,
        public_keys,
        clients_outside_of_slots_range: outside.len() as u32,
    })
}
