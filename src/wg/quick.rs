use thiserror::Error;

use std::process::Command;

use crate::ops::Ops;
use crate::shell_command_ext::{self, ShellCommandExt};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Command failed: {0}")]
    Command(#[from] shell_command_ext::Error),
}

/// Holds the WireGuard interface up for as long as it is alive.
pub struct Interface {
    ops: Ops,
}

impl Interface {
    pub fn up(ops: &Ops) -> Result<Self, Error> {
        Command::new("wg-quick")
            .arg("up")
            .arg(ops.wg_config.to_string_lossy().to_string())
            .run()?;
        Ok(Self { ops: ops.clone() })
    }
}

impl Drop for Interface {
    fn drop(&mut self) {
        // Drop cannot propagate; systemd's ExecStopPost is the backstop.
        if let Err(err) = down(&self.ops) {
            tracing::error!(?err, interface = %self.ops.interface_name, "Taking interface down failed");
        }
    }
}

fn down(ops: &Ops) -> Result<(), Error> {
    Command::new("wg-quick")
        .arg("down")
        .arg(ops.wg_config.to_string_lossy().to_string())
        .run()?;
    Ok(())
}
