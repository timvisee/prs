use clap::Command;

use crate::cmd::arg::{ArgTimeout, CmdArg};

/// The internal clipboard revert command definition.
pub struct CmdClipRevert;

impl CmdClipRevert {
    pub fn build() -> Command {
        Command::new("clip-revert")
            .about("Revert clipboard after timeout")
            .arg(ArgTimeout::build())
    }
}
