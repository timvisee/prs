use clap::{App, Arg, SubCommand};

use crate::cmd::arg::{ArgQuery, CmdArg};

/// The edit command definition.
pub struct CmdEdit;

impl CmdEdit {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("edit")
            .alias("e")
            .about("Edit a secret")
            .arg(ArgQuery::build())
            .arg(
                Arg::with_name("stdin")
                    .long("stdin")
                    .short("S")
                    .alias("from-stdin")
                    .help("Read secret from stdin, do not open editor")
                    .conflicts_with("empty"),
            )
    }
}
