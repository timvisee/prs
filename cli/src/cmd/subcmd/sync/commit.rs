use clap::{Arg, Command};

use crate::cmd::arg::{ArgNoSync, CmdArg};

/// The sync commit command definition.
pub struct CmdCommit;

impl CmdCommit {
    pub fn build() -> Command {
        Command::new("commit")
            .about("Commit all non-committed changes")
            .arg(
                Arg::new("message")
                    .long("message")
                    .short('m')
                    .alias("msg")
                    .value_name("MESSAGE")
                    .num_args(1)
                    .global(true)
                    .help("Custom commit message"),
            )
            .arg(ArgNoSync::build().help("Do not sync changes after commit"))
    }
}
