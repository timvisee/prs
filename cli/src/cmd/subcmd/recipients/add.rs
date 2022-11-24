use clap::{Arg, Command};

use crate::cmd::arg::{ArgAllowDirty, ArgNoSync, CmdArg};

/// The recipient add command definition.
pub struct CmdAdd;

impl CmdAdd {
    pub fn build() -> Command {
        Command::new("add")
            .alias("a")
            .about("Add store recipient")
            .arg(
                Arg::new("secret")
                    .long("secret")
                    .alias("private")
                    .num_args(0)
                    .help("Add public key we have private key for"),
            )
            .arg(
                Arg::new("no-recrypt")
                    .long("no-recrypt")
                    .alias("no-reencrypt")
                    .alias("skip-recrypt")
                    .alias("skip-reencrypt")
                    .num_args(0)
                    .help("Skip re-encrypting all secrets"),
            )
            .arg(ArgAllowDirty::build())
            .arg(ArgNoSync::build())
    }
}
