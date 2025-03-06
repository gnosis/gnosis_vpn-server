use rocket::serde::json::Json;
use rocket::State;
use serde::Serialize;
use std::net::Ipv4Addr;

use crate::api_error::ApiError;
use crate::dump;
use crate::dump::Peer;
use crate::ops::Ops;

#[derive(Debug, Serialize)]
pub struct StatusSingle {
    public_key: String,
    ip: Option<Ipv4Addr>,
    state: ConnectionState,
}

#[derive(Debug, Serialize)]
pub enum ConnectionState {
    NotRegistered,
    Connected,
    Expired,
    NeverConnected,
}

#[derive(Debug, Serialize)]
pub struct Status {
    slots: IpSlots,
    public_keys: PublicKeys,
    clients_outside_of_slots_range: u32,
}

#[derive(Debug, Serialize)]
pub struct ApiStatus {
    free: u32,
}

#[derive(Debug, Serialize)]
pub struct IpSlots {
    total: u32,
    free: u32,
    connected: u32,
    expired: u32,
    never_connected: u32,
}

#[derive(Debug, Serialize)]
pub struct PublicKeys {
    connected: Vec<String>,
    expired: Vec<String>,
    never_connected: Vec<String>,
}

#[derive(Debug, Serialize)]
pub enum Error {
    NoDevice,
    Dump(dump::Error),
    SystemTime(String),
}

#[get("/status/<public_key>")]
pub fn api_single(public_key: String, ops: &State<Ops>) -> Result<Json<StatusSingle>, Json<ApiError>> {
    let res = run_single(ops, &public_key);

    match res {
        Ok(status) => match status.state {
            ConnectionState::NotRegistered => Err(Json(ApiError::new(404, "Not Found", "Client not registered"))),
            _ => Ok(Json(status)),
        },
        Err(err) => {
            tracing::error!("Error during API status: {:?}", err);
            Err(Json(ApiError::internal_server_error()))
        }
    }
}

#[get("/status")]
pub fn api(ops: &State<Ops>) -> Result<Json<ApiStatus>, Json<ApiError>> {
    let res = run(ops);

    match res {
        Ok(status) => Ok(Json(ApiStatus {
            free: status.slots.free,
        })),
        Err(err) => {
            tracing::error!("Error during API status: {:?}", err);
            Err(Json(ApiError::internal_server_error()))
        }
    }
}

pub fn run_single(ops: &Ops, public_key: &str) -> Result<StatusSingle, Error> {
    let device = match ops.device() {
        Some(device) => device,
        None => return Err(Error::NoDevice),
    };
    let dump = dump::run(device).map_err(Error::Dump)?;
    let res_peer = dump.peers.iter().find(|peer| peer.public_key == public_key);
    match res_peer {
        Some(peer) => {
            if peer.has_handshaked() {
                if peer
                    .timed_out(&ops.client_handshake_timeout)
                    .map_err(|err| Error::SystemTime(err.to_string()))?
                {
                    Ok(StatusSingle {
                        public_key: peer.public_key.clone(),
                        ip: Some(peer.ip),
                        state: ConnectionState::Expired,
                    })
                } else {
                    Ok(StatusSingle {
                        public_key: peer.public_key.clone(),
                        ip: Some(peer.ip),
                        state: ConnectionState::Connected,
                    })
                }
            } else {
                Ok(StatusSingle {
                    public_key: peer.public_key.clone(),
                    ip: Some(peer.ip),
                    state: ConnectionState::NeverConnected,
                })
            }
        }
        None => Ok(StatusSingle {
            public_key: public_key.to_string(),
            ip: None,
            state: ConnectionState::NotRegistered,
        }),
    }
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

    let (connected_good_public_keys, expired_good_public_keys) = good_handshaked_public_keys
        .iter()
        .partition::<Vec<_>, _>(|(_, res_timed_out)| {
            if let Ok(timed_out) = res_timed_out {
                return !*timed_out;
            }
            false
        });

    let total = ops.client_address_range.count();
    let slots = IpSlots {
        total,
        free: total - inside.len() as u32,
        connected: connected_good_public_keys.len() as u32,
        expired: expired_good_public_keys.len() as u32,
        never_connected: never_connected_peers.len() as u32,
    };

    let public_keys = PublicKeys {
        connected: connected_good_public_keys
            .iter()
            .map(|(public_key, _)| public_key.clone())
            .collect(),
        expired: expired_good_public_keys
            .iter()
            .map(|(public_key, _)| public_key.clone())
            .collect(),
        never_connected: never_connected_peers
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
