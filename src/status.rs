use crate::ops::Ops;
use crate::wg_server::Dump;

#[derive(Debug)]
pub struct Status {
    pub allowed_clients: u32,
    pub registered_clients: u32,
    pub free_client_slots: u32,
    pub removable_timed_out_clients: u32,
}

impl Status {
    pub fn from_dump(dump: &Dump, ops: &Ops) -> Self {
        let registered_clients = dump.peers.len() as u32;

        Self {
            allowed_clients: 0,
            registered_clients,
            free_client_slots: 0,
            removable_timed_out_clients: 0,
        }
    }
}
