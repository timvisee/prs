use clap::{App, SubCommand};

use crate::cmd::arg::{ArgQuery, CmdArg};

/// The list command definition.
pub struct CmdList;

impl CmdList {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("list")
            .alias("ls")
            .alias("l")
            .about("List all secrets")
            .arg(ArgQuery::build())
    }
}
