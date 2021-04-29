use clap::{App, Arg};

use crate::cmd::arg::{ArgAllowDirty, ArgNoSync, CmdArg};

/// The recipient remove command definition.
pub struct CmdRemove;

impl CmdRemove {
    pub fn build<'a>() -> App<'a> {
        App::new("remove")
            .alias("rm")
            .alias("delete")
            .alias("del")
            .about("Remove store recipient")
            .arg(
                Arg::new("recrypt")
                    .long("recrypt")
                    .alias("reencrypt")
                    .about("Re-encrypting all secrets"),
            )
            .arg(ArgAllowDirty::build())
            .arg(ArgNoSync::build())
    }
}
