use thiserror::Error;

use std::process::Command;

use crate::ops::Ops;
use crate::shell_command_ext::{self, ShellCommandExt};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Command failed: {0}")]
    Command(#[from] shell_command_ext::Error),
}

pub fn up(ops: &Ops) -> Result<(), Error> {
    Command::new("wg-quick")
        .arg("up")
        .arg(ops.wg_config.to_string_lossy().to_string())
        .run()?;
    Ok(())
}

pub fn down(ops: &Ops) -> Result<(), Error> {
    Command::new("wg-quick")
        .arg("down")
        .arg(ops.wg_config.to_string_lossy().to_string())
        .run()?;
    Ok(())
}
