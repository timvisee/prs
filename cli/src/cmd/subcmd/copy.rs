use clap::{App, Arg, SubCommand};

use crate::cmd::arg::{ArgProperty, ArgQuery, ArgStore, ArgTimeout, CmdArg};

/// The copy command definition.
pub struct CmdCopy;

impl CmdCopy {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("copy")
            .alias("cp")
            .alias("c")
            .alias("yank")
            .alias("clip")
            .alias("clipboard")
            .about("Copy secret to clipboard")
            .arg(
                Arg::with_name("all")
                    .long("all")
                    .short("a")
                    .help("Copy whole secret, not just first line"),
            )
            .arg(ArgQuery::build())
            .arg(ArgTimeout::build())
            .arg(ArgStore::build())
            .arg(ArgProperty::build().conflicts_with("all"))
    }
}
