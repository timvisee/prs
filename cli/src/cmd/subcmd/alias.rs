use clap::{App, Arg};

use crate::cmd::arg::{ArgAllowDirty, ArgNoSync, ArgQuery, ArgStore, CmdArg};

/// The alias command definition.
pub struct CmdAlias;

impl CmdAlias {
    pub fn build<'a>() -> App<'a> {
        App::new("alias")
            .alias("ln")
            .alias("link")
            .alias("symlink")
            .about("Alias/symlink a secret")
            .long_about("Alias/symlink a secret without duplicating its content")
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
