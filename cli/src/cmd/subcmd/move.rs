use clap::{Arg, Command};

use crate::cmd::arg::{ArgAllowDirty, ArgNoSync, ArgQuery, CmdArg};

/// The move command definition.
pub struct CmdMove;

impl CmdMove {
    pub fn build() -> Command {
        Command::new("move")
            .alias("mov")
            .alias("mv")
            .alias("rename")
            .alias("ren")
            .about("Move a secret")
            .arg(ArgQuery::build().required(true))
            .arg(
                Arg::new("DEST")
                    .help("Secret destination path")
                    .required(true),
            )
            .arg(ArgAllowDirty::build())
            .arg(ArgNoSync::build())
    }
}
