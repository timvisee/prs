use clap::{App, Arg, SubCommand};

use crate::cmd::arg::{ArgStore, CmdArg};

/// The git command definition.
pub struct CmdGit;

impl CmdGit {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("git")
            .about("Invoke git command in password store")
            .arg(
                Arg::with_name("COMMAND")
                    .help("Git command to invoke")
                    .multiple(true),
            )
            .arg(ArgStore::build())
    }
}
