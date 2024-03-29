use clap::{Arg, Command};

use crate::cmd::arg::{ArgAllowDirty, ArgNoSync, ArgQuery, CmdArg};

/// The duplicate command definition.
pub struct CmdDuplicate;

impl CmdDuplicate {
    pub fn build() -> Command {
        Command::new("duplicate")
            .alias("dup")
            .about("Duplicate a secret")
            .long_about("Duplicate the contents of a secret to a new file")
            .arg(ArgQuery::build().required(true))
            .arg(
                Arg::new("DEST")
                    .help("Secret destination path")
                    .required(true),
            )
            .arg(ArgAllowDirty::build())
            .arg(ArgNoSync::build())
    }
}
