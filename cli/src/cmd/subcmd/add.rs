use clap::{App, Arg, SubCommand};

use crate::cmd::arg::{ArgStore, CmdArg};

/// The add command definition.
pub struct CmdAdd;

impl CmdAdd {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("add")
            .alias("a")
            .alias("new")
            .alias("n")
            .alias("create")
            .alias("insert")
            .alias("ins")
            .about("Add a secret")
            .arg(
                Arg::with_name("DEST")
                    .help("Secret destination path")
                    .required(true),
            )
            .arg(
                Arg::with_name("empty")
                    .long("empty")
                    .short("e")
                    .help("Add empty secret, do not edit"),
            )
            .arg(
                Arg::with_name("stdin")
                    .long("stdin")
                    .short("S")
                    .alias("from-stdin")
                    .help("Read secret from stdin, do not open editor")
                    .conflicts_with("empty"),
            )
            .arg(ArgStore::build())
    }
}
