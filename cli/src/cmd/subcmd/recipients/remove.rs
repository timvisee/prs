use clap::{Arg, Command};

use crate::cmd::arg::{ArgAllowDirty, ArgNoSync, CmdArg};

/// The recipient remove command definition.
pub struct CmdRemove;

impl CmdRemove {
    pub fn build() -> Command {
        Command::new("remove")
            .alias("rm")
            .alias("delete")
            .alias("del")
            .about("Remove store recipient")
            .arg(
                Arg::new("recrypt")
                    .long("recrypt")
                    .alias("reencrypt")
                    .num_args(0)
                    .help("Re-encrypting all secrets"),
            )
            .arg(ArgAllowDirty::build())
            .arg(ArgNoSync::build())
    }
}
