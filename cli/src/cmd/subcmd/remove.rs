use clap::App;

use crate::cmd::arg::{ArgQuery, ArgStore, CmdArg};

/// The remove command definition.
pub struct CmdRemove;

impl CmdRemove {
    pub fn build<'a>() -> App<'a> {
        App::new("remove")
            .alias("rm")
            .alias("delete")
            .alias("del")
            .alias("yeet")
            .about("Remove a secret")
            .arg(ArgQuery::build())
            .arg(ArgStore::build())
    }
}
