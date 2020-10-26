use clap::{App, Arg, SubCommand};

use crate::cmd::arg::{ArgQuery, ArgStore, CmdArg};

/// The move command definition.
pub struct CmdMove;

impl CmdMove {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("move")
            .alias("mov")
            .alias("mv")
            .alias("rename")
            .alias("ren")
            .about("Move a secret")
            .arg(ArgQuery::build().required(true))
            .arg(
                Arg::with_name("DEST")
                    .help("Secret destination path")
                    .required(true),
            )
            .arg(ArgStore::build())
    }
}
