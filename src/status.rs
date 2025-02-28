use serde::Serialize;

use crate::ops::Ops;
use crate::wg_server;
use crate::wg_server::{Dump, Peer};

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
    DumpError(wg_server::DumpError),
}

pub fn status(ops: &Ops) -> Result<Status, Error> {
    let device = match ops.device() {
        Some(device) => device,
        None => return Err(Error::NoDevice),
    };
    let wg_server = wg_server::WgServer::new(device);
    let dump = wg_server.dump().map_err(Error::DumpError)?;
    let status = Status::from_dump(&dump, &ops);
    Ok(status)
}

impl Status {
    pub fn from_dump(dump: &Dump, ops: &Ops) -> Self {
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

        Self {
            total_client_slots,
            registered_clients,
            free_client_slots,
            expired_clients,
            never_connected_clients,
            clients_outside_of_slots_range,
        }
    }
}
