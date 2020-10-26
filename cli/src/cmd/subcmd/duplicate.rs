use clap::{App, Arg, SubCommand};

use crate::cmd::arg::{ArgQuery, ArgStore, CmdArg};

/// The duplicate command definition.
pub struct CmdDuplicate;

impl CmdDuplicate {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("duplicate")
            .alias("dup")
            .about("Duplicate a secret")
            .arg(ArgQuery::build().required(true))
            .arg(
                Arg::with_name("DEST")
                    .help("Secret destination path")
                    .required(true),
            )
            .arg(ArgStore::build())
    }
}
