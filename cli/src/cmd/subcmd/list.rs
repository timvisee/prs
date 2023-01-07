use clap::{Arg, Command};

use crate::cmd::arg::{ArgQuery, CmdArg};

/// The list command definition.
pub struct CmdList;

impl CmdList {
    pub fn build() -> Command {
        Command::new("list")
            .alias("ls")
            .alias("l")
            .about("List all secrets")
            .arg(ArgQuery::build())
            .arg(
                Arg::new("list")
                    .long("list")
                    .short('l')
                    .alias("no-tree")
                    .num_args(0)
                    .help("Show as list, not as tree"),
            )
            .arg(
                Arg::new("aliases")
                    .long("aliases")
                    .short('a')
                    .alias("symlinks")
                    .alias("only-aliases")
                    .alias("only-symlinks")
                    .num_args(0)
                    .help("Show only alises"),
            )
            .arg(
                Arg::new("non-aliases")
                    .long("non-aliases")
                    .short('A')
                    .alias("non-symlinks")
                    .alias("no-aliases")
                    .alias("no-symlinks")
                    .alias("only-non-aliases")
                    .alias("only-non-symlinks")
                    .num_args(0)
                    .help("Show only non-alises")
                    .conflicts_with("aliases"),
            )
    }
}
