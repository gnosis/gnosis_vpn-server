use std::net::{Ipv4Addr, SocketAddr};
use std::time::{Duration, SystemTime, SystemTimeError, UNIX_EPOCH};

#[derive(Debug)]
#[allow(dead_code)]
pub struct Peer {
    pub public_key: String,
    pub preshared_key: String,
    pub endpoint: Option<SocketAddr>,
    pub ip: Ipv4Addr,
    pub latest_handshake: u64,
    pub transfer_rx: u64,
    pub transfer_tx: u64,
    pub persistent_keepalive: u64,
}

impl Peer {
    pub fn timed_out(&self, timeout: &Duration) -> Result<bool, SystemTimeError> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
        let dur = Duration::from_micros(self.latest_handshake);
        let valid = dur + *timeout > now;
        Ok(!valid)
    }

    pub fn has_handshaked(&self) -> bool {
        self.latest_handshake > 0
    }
}
