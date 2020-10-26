use clap::{App, SubCommand};

use crate::cmd::arg::{ArgQuery, ArgStore, CmdArg};

/// The delete command definition.
pub struct CmdDelete;

impl CmdDelete {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("delete")
            .alias("del")
            .alias("remove")
            .alias("rm")
            .about("Delete a secret")
            .arg(ArgQuery::build())
            .arg(ArgStore::build())
    }
}
