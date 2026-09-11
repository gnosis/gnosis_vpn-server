use rustix::fs::{FlockOperation, flock};
use thiserror::Error;

use std::fs::File;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("flock failed: {0}")]
    Flock(#[from] rustix::io::Errno),
}

/// Dropping this, or the process dying, releases the lock.
#[derive(Debug)]
#[allow(dead_code)]
pub struct Guard(File);

/// Blocks until no other thread or process is mutating the wg interface.
///
/// Not reentrant: acquiring twice in one process deadlocks, so only the outermost caller may take it.
pub fn acquire(wg_config: &Path) -> Result<Guard, Error> {
    let path = lock_path(wg_config);
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(&path)?;
    flock(&file, FlockOperation::LockExclusive)?;
    Ok(Guard(file))
}

// Suffix is appended rather than substituted: the interface name is the config file stem.
fn lock_path(wg_config: &Path) -> PathBuf {
    let mut path = wg_config.as_os_str().to_os_string();
    path.push(".lock");
    PathBuf::from(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn should_block_second_acquire_until_guard_is_dropped() -> anyhow::Result<()> {
        let wg_config = std::env::temp_dir().join("gnosis_vpn-server-test-interface.conf");
        let guard = acquire(&wg_config)?;

        let (tx, rx) = mpsc::channel();
        let contender = {
            let wg_config = wg_config.clone();
            thread::spawn(move || {
                let _guard = acquire(&wg_config).expect("contender failed to acquire");
                tx.send(()).expect("contender failed to signal");
            })
        };

        assert!(rx.recv_timeout(Duration::from_millis(200)).is_err());
        drop(guard);
        assert!(rx.recv_timeout(Duration::from_secs(5)).is_ok());

        contender.join().expect("contender panicked");
        Ok(())
    }

    #[test]
    fn should_append_lock_suffix_to_keep_interface_name_intact() {
        let path = lock_path(Path::new("/etc/wireguard/wggnosisvpn.conf"));
        assert_eq!(path, Path::new("/etc/wireguard/wggnosisvpn.conf.lock"));
    }
}
