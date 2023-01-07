use clap::{Arg, Command};

use crate::cmd::arg::{ArgProperty, ArgQuery, ArgTimeout, CmdArg};

/// The copy command definition.
pub struct CmdCopy;

impl CmdCopy {
    pub fn build() -> Command {
        Command::new("copy")
            .alias("cp")
            .alias("c")
            .alias("yank")
            .alias("clip")
            .alias("clipboard")
            .about("Copy secret to clipboard")
            .arg(
                Arg::new("all")
                    .long("all")
                    .short('a')
                    .num_args(0)
                    .help("Copy whole secret, not just first line"),
            )
            .arg(ArgQuery::build())
            .arg(ArgTimeout::build())
            .arg(ArgProperty::build().conflicts_with("all"))
    }
}
