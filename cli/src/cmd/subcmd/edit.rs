use clap::{Arg, Command};

use crate::cmd::arg::{ArgAllowDirty, ArgNoSync, ArgQuery, CmdArg};

/// The edit command definition.
pub struct CmdEdit;

impl CmdEdit {
    pub fn build() -> Command {
        Command::new("edit")
            .alias("e")
            .about("Edit a secret")
            .arg(ArgQuery::build())
            .arg(
                Arg::new("stdin")
                    .long("stdin")
                    .short('S')
                    .alias("from-stdin")
                    .num_args(0)
                    .help("Read secret from stdin, do not open editor"),
            )
            .arg(ArgAllowDirty::build())
            .arg(ArgNoSync::build())
    }
}
