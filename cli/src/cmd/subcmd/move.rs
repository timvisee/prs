use clap::{App, Arg};

use crate::cmd::arg::{ArgAllowDirty, ArgNoSync, ArgQuery, ArgStore, CmdArg};

/// The move command definition.
pub struct CmdMove;

impl CmdMove {
    pub fn build<'a>() -> App<'a> {
        App::new("move")
            .alias("mov")
            .alias("mv")
            .alias("rename")
            .alias("ren")
            .about("Move a secret")
            .arg(ArgQuery::build().required(true))
            .arg(
                Arg::new("DEST")
                    .about("Secret destination path")
                    .required(true),
            )
            .arg(ArgStore::build())
            .arg(ArgAllowDirty::build())
            .arg(ArgNoSync::build())
    }
}
