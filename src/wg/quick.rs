use std::process::Command;

use crate::ops::Ops;
use crate::shell_command_ext::{self, ShellCommandExt};

pub fn up(ops: &Ops) -> Result<(), shell_command_ext::Error> {
    Command::new("wg-quick")
        .arg("up")
        .arg(ops.wg_config.to_string_lossy().to_string())
        .run()
}

pub fn down(ops: &Ops) -> Result<(), shell_command_ext::Error> {
    Command::new("wg-quick")
        .arg("down")
        .arg(ops.wg_config.to_string_lossy().to_string())
        .run()
}
