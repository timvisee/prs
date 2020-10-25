use clap::{App, SubCommand};

use crate::cmd::arg::{ArgQuery, CmdArg};

/// The edit command definition.
pub struct CmdEdit;

impl CmdEdit {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("edit")
            .about("Edit a secret")
            .alias("e")
            .arg(ArgQuery::build())
    }
}
