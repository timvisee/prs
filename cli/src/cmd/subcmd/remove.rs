use clap::{App, SubCommand};

use crate::cmd::arg::{ArgQuery, ArgStore, CmdArg};

/// The remove command definition.
pub struct CmdRemove;

impl CmdRemove {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("remove")
            .alias("rm")
            .alias("delete")
            .alias("del")
            .about("Remove a secret")
            .arg(ArgQuery::build())
            .arg(ArgStore::build())
    }
}
