use clap::Command;

use crate::cmd::arg::{ArgAllowDirty, ArgNoSync, ArgQuery, ArgStore, CmdArg};

/// The remove command definition.
pub struct CmdRemove;

impl CmdRemove {
    pub fn build() -> Command {
        Command::new("remove")
            .alias("rm")
            .alias("delete")
            .alias("del")
            .alias("yeet")
            .about("Remove a secret")
            .arg(ArgQuery::build())
            .arg(ArgStore::build())
            .arg(ArgAllowDirty::build())
            .arg(ArgNoSync::build())
    }
}
