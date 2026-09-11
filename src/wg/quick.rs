use thiserror::Error;

use std::process::Command;

use crate::ops::Ops;
use crate::shell_command_ext::{self, ShellCommandExt};
use crate::wg::lock;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Command failed: {0}")]
    Command(#[from] shell_command_ext::Error),
    #[error("wg lock error: {0}")]
    WgLock(#[from] lock::Error),
}

/// Holds the WireGuard interface up for as long as it is alive.
pub struct Interface {
    ops: Ops,
}

impl Interface {
    pub fn up(ops: &Ops) -> Result<Self, Error> {
        let _wg_lock = lock::acquire(&ops.wg_config)?;
        Command::new("wg-quick").arg("up").arg(&ops.wg_config).run()?;
        Ok(Self { ops: ops.clone() })
    }
}

impl Drop for Interface {
    fn drop(&mut self) {
        // taking the interface down unlocked beats leaving it up, so a failed acquire is not fatal
        let _wg_lock = lock::acquire(&self.ops.wg_config)
            .inspect_err(|err| tracing::error!(?err, "Locking wg interface for teardown failed"));

        // Drop cannot propagate; systemd's ExecStopPost is the backstop.
        if let Err(err) = down(&self.ops) {
            tracing::error!(?err, interface = %self.ops.interface_name, "Taking interface down failed");
        }
    }
}

fn down(ops: &Ops) -> Result<(), Error> {
    Command::new("wg-quick").arg("down").arg(&ops.wg_config).run()?;
    Ok(())
}
