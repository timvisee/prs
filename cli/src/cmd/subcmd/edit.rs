use clap::{App, Arg};

use crate::cmd::arg::{ArgAllowDirty, ArgNoSync, ArgQuery, ArgStore, CmdArg};

/// The edit command definition.
pub struct CmdEdit;

impl CmdEdit {
    pub fn build<'a>() -> App<'a> {
        App::new("edit")
            .alias("e")
            .about("Edit a secret")
            .arg(ArgQuery::build())
            .arg(
                Arg::new("stdin")
                    .long("stdin")
                    .short('S')
                    .alias("from-stdin")
                    .help("Read secret from stdin, do not open editor"),
            )
            .arg(ArgStore::build())
            .arg(ArgAllowDirty::build())
            .arg(ArgNoSync::build())
    }
}
