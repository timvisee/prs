use clap::{App, Arg, SubCommand};

use crate::cmd::arg::{ArgQuery, ArgStore, CmdArg};

/// The list command definition.
pub struct CmdList;

impl CmdList {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("list")
            .alias("ls")
            .alias("l")
            .about("List all secrets")
            .arg(ArgQuery::build())
            .arg(ArgStore::build())
            .arg(
                Arg::with_name("aliases")
                    .long("aliases")
                    .short("a")
                    .alias("symlinks")
                    .alias("only-aliases")
                    .alias("only-symlinks")
                    .help("Show only alises"),
            )
            .arg(
                Arg::with_name("non-aliases")
                    .long("non-aliases")
                    .short("A")
                    .alias("non-symlinks")
                    .alias("no-aliases")
                    .alias("no-symlinks")
                    .alias("only-non-aliases")
                    .alias("only-non-symlinks")
                    .help("Show only non-alises")
                    .conflicts_with("aliases"),
            )
    }
}
