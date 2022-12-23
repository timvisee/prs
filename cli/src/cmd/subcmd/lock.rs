use clap::Command;

use crate::cmd::arg::{ArgStore, CmdArg};

/// The lock command definition.
pub struct CmdLock;

impl CmdLock {
    pub fn build() -> Command {
        Command::new("lock")
            .alias("lockdown")
            .alias("slam")
            .alias("shut")
            .alias("emergency")
            .alias("sos")
            .about("Aggresively lock password store & keys preventing access (emergency)")
            .arg(ArgStore::build())
    }
}
