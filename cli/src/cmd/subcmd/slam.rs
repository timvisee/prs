use clap::Command;

use crate::cmd::arg::{ArgStore, CmdArg};

/// The slam command definition.
pub struct CmdSlam;

impl CmdSlam {
    pub fn build() -> Command {
        Command::new("slam")
            .alias("lock")
            .alias("lockdown")
            .alias("shut")
            .alias("emergency")
            .alias("sos")
            .about("Aggressively lock password store & keys preventing access (emergency)")
            .arg(ArgStore::build())
    }
}
