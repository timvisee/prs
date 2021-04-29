use clap::{App, Arg};

use crate::cmd::arg::{ArgAllowDirty, ArgNoSync, ArgStore, CmdArg};

/// The add command definition.
pub struct CmdAdd;

impl CmdAdd {
    pub fn build<'a>() -> App<'a> {
        App::new("add")
            .alias("a")
            .alias("new")
            .alias("n")
            .alias("create")
            .alias("insert")
            .alias("ins")
            .about("Add a secret")
            .arg(
                Arg::new("DEST")
                    .about("Secret destination path")
                    .required(true),
            )
            .arg(
                Arg::new("empty")
                    .long("empty")
                    .short('e')
                    .about("Add empty secret, do not edit"),
            )
            .arg(
                Arg::new("stdin")
                    .long("stdin")
                    .short('S')
                    .alias("from-stdin")
                    .about("Read secret from stdin, do not open editor")
                    .conflicts_with("empty"),
            )
            .arg(ArgStore::build())
            .arg(ArgAllowDirty::build())
            .arg(ArgNoSync::build())
    }
}
