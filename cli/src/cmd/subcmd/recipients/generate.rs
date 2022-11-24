use clap::{Arg, Command};

use crate::cmd::arg::{ArgAllowDirty, ArgNoSync, CmdArg};

/// The recipient generate command definition.
pub struct CmdGenerate;

impl CmdGenerate {
    pub fn build() -> Command {
        Command::new("generate")
            .alias("gen")
            .alias("g")
            .about("Generate new key pair, add it to the store")
            .arg(
                Arg::new("no-add")
                    .long("no-add")
                    .alias("skip-add")
                    .num_args(0)
                    .help("Skip adding key pair to store"),
            )
            .arg(
                Arg::new("no-recrypt")
                    .long("no-recrypt")
                    .alias("no-reencrypt")
                    .alias("skip-recrypt")
                    .alias("skip-reencrypt")
                    .num_args(0)
                    .help("Skip re-encrypting all secrets")
                    .conflicts_with("no-add"),
            )
            .arg(ArgAllowDirty::build())
            .arg(ArgNoSync::build())
    }
}
