use serde::Serialize;

use crate::dump;
use crate::dump::Peer;
use crate::ops::Ops;

#[derive(Debug, Serialize)]
pub struct Status {
    pub total_client_slots: u32,
    pub free_client_slots: u32,
    pub registered_clients: u32,
    pub expired_clients: u32,
    pub never_connected_clients: u32,
    pub clients_outside_of_slots_range: u32,
}

#[derive(Debug, Serialize)]
pub enum Error {
    NoDevice,
    Dump(dump::Error),
}

pub fn run(ops: &Ops) -> Result<Status, Error> {
    let device = match ops.device() {
        Some(device) => device,
        None => return Err(Error::NoDevice),
    };
    let dump = dump::dump(device).map_err(Error::Dump)?;

    let total_client_slots = ops.client_address_range.count();
    let (inside, outside): (Vec<&Peer>, Vec<&Peer>) = dump
        .peers
        .iter()
        .partition(|peer| ops.client_address_range.contains(peer.ip));

    let registered_clients = inside.len() as u32;
    let free_client_slots = total_client_slots - registered_clients;
    let never_connected_clients = inside.iter().filter(|peer| !peer.has_handshaked()).count() as u32;

    let expired_clients = inside
        .iter()
        .filter(|peer| peer.has_handshaked() && peer.timed_out(&ops.client_handshake_timeout).unwrap_or(false))
        .count() as u32;
    let clients_outside_of_slots_range = outside.len() as u32;

    Ok(Status {
        total_client_slots,
        registered_clients,
        free_client_slots,
        expired_clients,
        never_connected_clients,
        clients_outside_of_slots_range,
    })
}
