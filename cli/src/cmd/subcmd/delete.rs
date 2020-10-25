use clap::{App, SubCommand};

use crate::cmd::arg::{ArgQuery, CmdArg};

/// The delete command definition.
pub struct CmdDelete;

impl CmdDelete {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("delete")
            .about("Delete secret")
            .alias("del")
            .alias("remove")
            .alias("rm")
            .arg(ArgQuery::build())
    }
}
