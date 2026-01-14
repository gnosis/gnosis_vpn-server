use thiserror::Error;

use std::fs::File;
use std::io::Error as IOError;
use std::io::Write;
use std::process::Command;

use crate::ops::Ops;
use crate::shell_command_ext::{self, ShellCommandExt};
use crate::wg::peer::Peer;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Dump {
    private_key: String,
    public_key: String,
    listen_port: u16,
    fwmark: String,
    pub peers: Vec<Peer>,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    IO(#[from] IOError),
    #[error("Command failed: {0}")]
    Command(#[from] shell_command_ext::Error),
}

pub fn save_file(ops: &Ops) -> Result<(), Error> {
    // wg showconf omits parts that wg-quick needs
    // in order to keep existing values, the interface section comes from the on disk wg config
    let wg_stdout = Command::new("wg")
        .arg("showconf")
        .arg(ops.interface_name.clone())
        .run_stdout()?;

    let peers_content = extract_peers(wg_stdout);

    // Prepend with maintainer information
    let prepend_str = format!("# Maintained by {}", env!("CARGO_PKG_NAME"));

    let mut lines: Vec<String> = Vec::new();
    lines.push(prepend_str);
    lines.push("".to_string());
    lines.extend(ops.interface_section.clone());
    lines.push("".to_string());
    lines.extend(peers_content);

    let content = lines.join("\n").into_bytes();
    let mut f = File::create(&ops.wg_config)?;
    f.write_all(&content)?;
    Ok(())
}

fn extract_peers(content: String) -> Vec<String> {
    let mut lines = Vec::new();
    let mut in_peer_section = false;

    for line in content.lines() {
        let trimmed_line = line.trim();
        if trimmed_line.starts_with("[Peer]") {
            in_peer_section = true;
            lines.push(trimmed_line.to_string());
        } else if trimmed_line.starts_with('[') {
            in_peer_section = false;
        } else if in_peer_section {
            lines.push(line.to_string());
        }
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_extract_peers() -> anyhow::Result<()> {
        let content = r#"
            [Interface]
            Address = 10.128.0.0/32
            PrivateKey = someprivatekey
            [Peer]
            PublicKey = somepublickey
            AllowedIPs = 10.128.0.120/32
            [Peer]
            PublicKey = anotherpublickey
            AllowedIPs = 10.128.0.122/32
        "#;
        let section = extract_peers(content.to_string());
        assert_eq!(section.len(), 7);
        assert!(section.iter().any(|line| line.contains("PublicKey = somepublickey")));
        assert!(section.iter().any(|line| line.contains("PublicKey = anotherpublickey")));
        assert!(section.iter().any(|line| line.contains("AllowedIPs = 10.128.0.120/32")));
        assert!(section.iter().any(|line| line.contains("AllowedIPs = 10.128.0.122/32")));
        assert!(section.iter().all(|line| !line.contains("[Interface]")));
        assert!(section.iter().all(|line| !line.contains("Address =")));
        assert!(section.iter().all(|line| !line.contains("PrivateKey =")));
        Ok(())
    }
}
