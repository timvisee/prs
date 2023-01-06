pub mod recrypt;
pub mod run;
pub mod sync_keys;

use clap::Command;

use crate::cmd::arg::{ArgStore, CmdArg};

/// The housekeeping command definition.
pub struct CmdHousekeeping;

impl CmdHousekeeping {
    pub fn build() -> Command {
        Command::new("housekeeping")
            .about("Housekeeping utilities")
            .alias("housekeep")
            .alias("hk")
            .arg_required_else_help(true)
            .subcommand_required(true)
            .subcommand_value_name("ACTION")
            .subcommand(run::CmdRun::build())
            .subcommand(recrypt::CmdRecrypt::build())
            .subcommand(sync_keys::CmdSyncKeys::build())
            .arg(ArgStore::build())
    }
}
