use clap::{App, SubCommand};

use crate::cmd::arg::{ArgStore, CmdArg};

/// The sync command definition.
pub struct CmdSync;

impl CmdSync {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("sync")
            .alias("s")
            .about("Sync password store")
            .arg(ArgStore::build())
    }
}
