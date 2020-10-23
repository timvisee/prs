use clap::{App, SubCommand};

use crate::cmd::arg::{ArgQuery, CmdArg};

/// The list command definition.
pub struct CmdList;

impl CmdList {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("list")
            .about("List all secrets")
            .alias("ls")
            .alias("l")
            .arg(ArgQuery::build())
    }
}
