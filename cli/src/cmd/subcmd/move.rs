use clap::{App, Arg, SubCommand};

use crate::cmd::arg::{ArgQuery, CmdArg};

/// The move command definition.
pub struct CmdMove;

impl CmdMove {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("move")
            .about("Move secret")
            .alias("mov")
            .alias("mv")
            .alias("rename")
            .alias("ren")
            .arg(ArgQuery::build().required(true))
            .arg(
                Arg::with_name("TARGET")
                    .help("Secret target path")
                    .required(true),
            )
    }
}
