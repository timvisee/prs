use clap::{App, Arg, SubCommand};

use crate::cmd::arg::{ArgQuery, CmdArg};

/// The copy command definition.
pub struct CmdCopy;

impl CmdCopy {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("copy")
            .alias("cp")
            .alias("c")
            .about("Copy secret to clipboard")
            .arg(
                Arg::with_name("all")
                    .long("all")
                    .short("a")
                    .help("Copy whole secret, not just first line"),
            )
            .arg(ArgQuery::build())
    }
}
