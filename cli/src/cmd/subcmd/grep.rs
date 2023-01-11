use clap::{Arg, Command};

use crate::cmd::arg::{ArgQuery, CmdArg};

/// The grep command definition.
pub struct CmdGrep;

impl CmdGrep {
    pub fn build() -> Command {
        Command::new("grep")
            .alias("find")
            .about("Grep all secrets")
            .arg(Arg::new("PATTERN").required(true).help("Grep pattern"))
            .arg(
                ArgQuery::build()
                    .id("query")
                    .long("query")
                    .short('Q')
                    .help("Limit grep to secrets by query"),
            )
            .arg(
                Arg::new("aliases")
                    .long("aliases")
                    .short('a')
                    .alias("symlinks")
                    .alias("with-aliases")
                    .alias("with-symlinks")
                    .num_args(0)
                    .help("Include grepping aliases"),
            )
    }
}
