use serde::Serialize;

use crate::ops::Ops;
use crate::wg_server::{Dump, Peer};

#[derive(Debug, Serialize)]
pub struct Status {
    pub total_allowed_clients: u32,
    pub registered_allowed_clients: u32,
    pub free_client_slots: u32,
    pub removable_allowed_timed_out_clients: u32,
    pub registered_clients_outside_of_range: u32,
}

impl Status {
    pub fn from_dump(dump: &Dump, ops: &Ops) -> Self {
        let total_allowed_clients = ops.client_address_range.count();
        let (inside, outside): (Vec<&Peer>, Vec<&Peer>) = dump
            .peers
            .iter()
            .partition(|peer| ops.client_address_range.contains(peer.ip));

        let registered_allowed_clients = inside.len() as u32;
        let free_client_slots = total_allowed_clients - registered_allowed_clients;
        let removable_allowed_timed_out_clients = inside
            .iter()
            .filter(|peer| peer.timed_out(&ops.client_handshake_timeout).unwrap_or(false))
            .count() as u32;
        let registered_clients_outside_of_range = outside.len() as u32;

        Self {
            total_allowed_clients,
            registered_allowed_clients,
            free_client_slots,
            removable_allowed_timed_out_clients,
            registered_clients_outside_of_range,
        }
    }
}
