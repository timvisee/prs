pub mod recrypt;
pub mod run;
pub mod sync_keys;

use clap::Command;

use crate::cmd::arg::{ArgStore, CmdArg};

/// The housekeeping command definition.
pub struct CmdHousekeeping;

impl CmdHousekeeping {
    pub fn build<'a>() -> Command<'a> {
        Command::new("housekeeping")
            .about("Housekeeping utilities")
            .alias("housekeep")
            .alias("hk")
            .arg_required_else_help(true)
            .subcommand(recrypt::CmdRecrypt::build())
            .subcommand(run::CmdRun::build())
            .subcommand(sync_keys::CmdSyncKeys::build())
            .arg(ArgStore::build())
    }
}
