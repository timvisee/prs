use clap::{App, Arg, SubCommand};

use crate::cmd::arg::{ArgQuery, ArgStore, CmdArg};

/// The alias command definition.
pub struct CmdAlias;

impl CmdAlias {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("alias")
            .alias("ln")
            .alias("link")
            .alias("symlink")
            .about("Alias/symlink a secret")
            .arg(ArgQuery::build().required(true))
            .arg(
                Arg::with_name("DEST")
                    .help("Secret destination path")
                    .required(true),
            )
            .arg(ArgStore::build())
    }
}
