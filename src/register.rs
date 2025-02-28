use std::net::{Ipv4Addr, SocketAddr};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Register {
    pub ip: Ipv4Addr,
}

#[derive(Debug, Serialize)]
pub enum Error {}


    pub fn register(&mut self, ops: &Ops, public_key: &str) -> Result<Ipv4Addr, RegisterError> {
        let dump = self.dump().map_err(RegisterError::Generic)?;
        let res_peer = dump.peers.iter().find(|peer| peer.public_key == public_key);
        if let Some(peer) = res_peer {
            return Ok(peer.ip);
        }

        let existing_ips: HashSet<Ipv4Addr> = HashSet::from_iter(dump.peers.iter().map(|peer| peer.ip));
        let res_ip = ops.client_address_range.find_free_ip(&existing_ips, &mut self.rng);
        let ip = match res_ip {
            Some(ip) => ip,
            None => return Err(RegisterError::NoFreeIp),
        };

        let output = Command::new("wg")
            .arg("set")
            .arg(&self.device)
            .arg("peer")
            .arg(public_key)
            .arg("allowed-ips")
            .arg(format!("{}/32", ip))
            .output()
            .with_context(|| format!("error executing wg set peer {} allowed-ips {}/32", public_key, ip))
            .map_err(RegisterError::Generic)?;

        if output.status.success() {
            Ok(ip)
        } else {
            Err(RegisterError::Generic(format!(
                "wg set peer {} allowed-ips {}/32 failed: {:?}",
                public_key, ip, output
            ))))
        }
    }
            let device = ops.device().ok_or(anyhow::anyhow!("failed to determine device name"))?;
            let mut wg_server = wg_server::WgServer::new(device);
            let result = wg_server.register(&ops, &public_key);
            match result {
                Ok(ip) => {
                    if json {
                        println!("{{\"ip\": \"{}\"}}", ip);
                    } else {
                        println!("{}", ip);
                    }
                }
                Err(err) => match err {
                    wg_server::RegisterError::NoFreeIp => {
                        println!("no free IP available");
                    }
                    wg_server::RegisterError::Generic(err) => {
                        eprintln!("error: {}", err);
                    }
                },
            }
