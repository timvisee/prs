use clap::Command;

use crate::cmd::arg::{ArgAllowDirty, ArgNoSync, CmdArg};

/// The housekeeping run command definition.
pub struct CmdRun;

impl CmdRun {
    pub fn build() -> Command {
        Command::new("run")
            .about("Run housekeeping tasks")
            .arg(ArgAllowDirty::build())
            .arg(ArgNoSync::build())
    }
}
