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
        let handshaked = Duration::from_secs(self.latest_handshake);
        let valid = handshaked + *timeout > now;
        Ok(!valid)
    }

    pub fn has_handshaked(&self) -> bool {
        self.latest_handshake > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_peer(latest_handshake: u64) -> Peer {
        Peer {
            public_key: "test".to_string(),
            preshared_key: String::new(),
            endpoint: None,
            ip: Ipv4Addr::new(10, 0, 0, 2),
            latest_handshake,
            transfer_rx: 0,
            transfer_tx: 0,
            persistent_keepalive: 0,
        }
    }

    #[test]
    fn should_flag_handshake_presence_by_latest_handshake_value() -> anyhow::Result<()> {
        assert!(base_peer(1).has_handshaked());
        assert!(!base_peer(0).has_handshaked());

        Ok(())
    }

    #[test]
    fn should_mark_peer_as_timed_out_when_handshake_older_than_timeout() -> anyhow::Result<()> {
        let recent = base_peer(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)?
                .saturating_sub(Duration::from_secs(10))
                .as_secs(),
        );
        let stale = base_peer(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)?
                .saturating_sub(Duration::from_secs(600))
                .as_secs(),
        );

        assert!(!recent.timed_out(&Duration::from_secs(60))?);
        assert!(stale.timed_out(&Duration::from_secs(60))?);
        Ok(())
    }
}
