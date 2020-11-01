use clap::{App, SubCommand};

use crate::cmd::arg::{ArgStore, CmdArg};

/// The init command definition.
pub struct CmdInit;

impl CmdInit {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("init")
            .alias("initialize")
            .about("Initialize new password store")
            .arg(ArgStore::build())
    }
}
