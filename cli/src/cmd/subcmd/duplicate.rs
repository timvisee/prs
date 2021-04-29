use clap::{App, Arg};

use crate::cmd::arg::{ArgAllowDirty, ArgNoSync, ArgQuery, ArgStore, CmdArg};

/// The duplicate command definition.
pub struct CmdDuplicate;

impl CmdDuplicate {
    pub fn build<'a>() -> App<'a> {
        App::new("duplicate")
            .alias("dup")
            .about("Duplicate a secret")
            .long_about("Duplicate the contents of a secret to a new file")
            .arg(ArgQuery::build().required(true))
            .arg(
                Arg::new("DEST")
                    .about("Secret destination path")
                    .required(true),
            )
            .arg(ArgStore::build())
            .arg(ArgAllowDirty::build())
            .arg(ArgNoSync::build())
    }
}
