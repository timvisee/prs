use clap::Command;

use crate::cmd::arg::{ArgNoSync, CmdArg};

/// The sync reset command definition.
pub struct CmdReset;

impl CmdReset {
    pub fn build() -> Command {
        Command::new("reset")
            .about("Reset all non-committed changes")
            .arg(ArgNoSync::build().help("Do not sync changes after reset"))
    }
}
