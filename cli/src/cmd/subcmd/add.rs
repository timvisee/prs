use clap::{Arg, Command};

use crate::cmd::arg::{ArgAllowDirty, ArgNoSync, CmdArg};

/// The add command definition.
pub struct CmdAdd;

impl CmdAdd {
    pub fn build() -> Command {
        Command::new("add")
            .alias("a")
            .alias("new")
            .alias("n")
            .alias("create")
            .alias("insert")
            .alias("ins")
            .about("Add a secret")
            .arg(Arg::new("NAME").help("Secret name and path").required(true))
            .arg(
                Arg::new("empty")
                    .long("empty")
                    .short('e')
                    .num_args(0)
                    .help("Add empty secret, do not edit"),
            )
            .arg(
                Arg::new("stdin")
                    .long("stdin")
                    .short('S')
                    .alias("from-stdin")
                    .num_args(0)
                    .help("Read secret from stdin, do not open editor")
                    .conflicts_with("empty"),
            )
            .arg(ArgAllowDirty::build())
            .arg(ArgNoSync::build())
    }
}
