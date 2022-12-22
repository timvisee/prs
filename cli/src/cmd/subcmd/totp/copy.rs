use clap::Command;

use crate::cmd::arg::{ArgProperty, ArgQuery, ArgStore, ArgTimeout, CmdArg};

/// The TOTP copy command definition.
pub struct CmdCopy;

impl CmdCopy {
    pub fn build() -> Command {
        Command::new("copy")
            .alias("cp")
            .alias("c")
            .alias("yank")
            .alias("clip")
            .alias("clipboard")
            .about("Copy TOTP token to clipboard")
            .arg(ArgQuery::build())
            .arg(ArgTimeout::build())
            .arg(ArgStore::build())
            .arg(ArgProperty::build())
    }
}
