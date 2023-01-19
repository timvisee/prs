use clap::Command;

use crate::cmd::arg::{ArgNoSync, CmdArg};

/// The sync commit command defcommition.
pub struct CmdCommit;

impl CmdCommit {
    pub fn build() -> Command {
        Command::new("commit")
            .about("Commit all non-committed changes")
            .arg(ArgNoSync::build())
    }
}
