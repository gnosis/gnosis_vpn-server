use serde::Serialize;
use std::net::{Ipv4Addr, SocketAddr};
use std::process::Command;

pub use peer::Peer;

mod peer;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Dump {
    private_key: String,
    public_key: String,
    listen_port: u16,
    fwmark: String,
    pub peers: Vec<Peer>,
}

#[derive(Debug, Serialize)]
pub enum Error {
    Generic(String),
    NoOutputLines,
    WrongNumberOfFieldsInServerLine,
    WrongNumberOfFieldsInPeerLine,
}

pub fn dump(device: &str) -> Result<Dump, Error> {
    let res_output = Command::new("wg").arg("show").arg(device).arg("dump").output();

    let output = match res_output {
        Ok(output) => output,
        Err(err) => {
            return Err(Error::Generic(format!("wg show {} dump failed: {}", device, err)));
        }
    };

    if !output.status.success() {
        return Err(Error::Generic(format!("wg show dump failed: {:?}", output)));
    }

    let content = match String::from_utf8(output.stdout) {
        Ok(content) => content,
        Err(err) => {
            return Err(Error::Generic(format!("error parsing wg show output: {}", err)));
        }
    };

    let output_lines: Vec<&str> = content.split('\n').collect();
    let lines: Vec<&str> = output_lines
        .iter()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    if lines.is_empty() {
        return Err(Error::NoOutputLines);
    }

    dump_from_lines(&lines)
}

fn dump_from_lines(lines: &Vec<&str>) -> Result<Dump, Error> {
    let initial_line = lines[0];
    let initial_parts: Vec<&str> = initial_line.split('\t').collect();

    if initial_parts.len() != 4 {
        return Err(Error::WrongNumberOfFieldsInServerLine);
    }

    let private_key = initial_parts[0].to_string();
    let public_key = initial_parts[1].to_string();
    let listen_port = initial_parts[2].parse::<u16>().unwrap_or_default();
    let fwmark = initial_parts[3].to_string();

    Ok(Dump {
        private_key,
        public_key,
        listen_port,
        fwmark,
        peers: peers_from_lines(lines)?,
    })
}

fn peers_from_lines(lines: &Vec<&str>) -> Result<Vec<Peer>, Error> {
    lines
        .iter()
        .skip(1)
        .map(|line| {
            let parts: Vec<&str> = line.split('\t').collect();
            parts
        })
        .filter(|parts| parts.len() > 1)
        .map(|parts| {
            if parts.len() != 8 {
                return Err(Error::WrongNumberOfFieldsInPeerLine);
            }

            let public_key = parts[0].to_string();
            let preshared_key = parts[1].to_string();
            let endpoint = parts[2].parse::<SocketAddr>().ok();
            let allowed_ips = parts[3].to_string();
            let res_ip = allowed_ips.split("/32").take(1).collect::<String>().parse::<Ipv4Addr>();
            let ip = match res_ip {
                Ok(ip) => ip,
                Err(err) => {
                    return Err(Error::Generic(format!(
                        "unable to parse ip from allowed_ips[{}]: {}",
                        allowed_ips, err
                    )))
                }
            };

            let latest_handshake = parts[4].parse::<u64>().unwrap_or_default();
            let transfer_rx = parts[5].parse::<u64>().unwrap_or_default();
            let transfer_tx = parts[6].parse::<u64>().unwrap_or_default();
            let persistent_keepalive = parts[7].parse::<u64>().unwrap_or_default();
            Ok(Peer {
                public_key,
                preshared_key,
                endpoint,
                ip,
                latest_handshake,
                transfer_rx,
                transfer_tx,
                persistent_keepalive,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::dump_from_lines;

    fn sample_output() -> &'static str {
        r###"GEd85EVCEFq5NfKEeTbRUHRutLF0+1WNEP4WG8Aq1kc=	o+5emsMXvOUqeUSmwATZN10v2lYMu/FgMMsABclML3c=	51820	off
hP+s10cZM6oPORcM7YR0lgpxre84Kr1R+EDOb4eg7Qo=	(none)	172.18.0.4:53169	10.128.0.2/32	1740069703	745872	30210620	30
qSF/V46h4fIjpChUU3lAsA13W+E7+uHIB7N2Riu+rVE=	(none)	(none)	10.128.0.3/32	0	0	0	30
TlPYzf9UxM2Jm5K2d0B6SSiekBQlmJ1MgP6YhzivIR4=	(none)	172.18.0.4:47258	10.128.0.6/32	1740484870	20855368	69071528	30
RhJoIbojG5m3+GoNtliiZAVJ0kyxiEPGegwBwE58FmA=	(none)	(none)	10.128.0.7/32	0	0	0	30
"###
    }

    #[test]
    fn test_dump() {
        let lines: Vec<&str> = sample_output().split('\n').collect();
        let res_dump = dump_from_lines(&lines);
        assert!(res_dump.is_ok());
        let dump = res_dump.unwrap();
        assert_eq!(dump.private_key, "GEd85EVCEFq5NfKEeTbRUHRutLF0+1WNEP4WG8Aq1kc=");
        assert_eq!(dump.public_key, "o+5emsMXvOUqeUSmwATZN10v2lYMu/FgMMsABclML3c=");
        assert_eq!(dump.listen_port, 51820);
        assert_eq!(dump.fwmark, "off");
        assert_eq!(dump.peers.len(), 4);
        assert_eq!(dump.peers[0].public_key, "hP+s10cZM6oPORcM7YR0lgpxre84Kr1R+EDOb4eg7Qo=");
        assert_eq!(dump.peers[1].public_key, "qSF/V46h4fIjpChUU3lAsA13W+E7+uHIB7N2Riu+rVE=");
        assert_eq!(dump.peers[2].public_key, "TlPYzf9UxM2Jm5K2d0B6SSiekBQlmJ1MgP6YhzivIR4=");
        assert_eq!(dump.peers[3].public_key, "RhJoIbojG5m3+GoNtliiZAVJ0kyxiEPGegwBwE58FmA=");
    }
}
