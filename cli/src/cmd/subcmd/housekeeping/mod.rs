pub mod recrypt;
pub mod run;
pub mod sync_keys;

use clap::{App, AppSettings, SubCommand};

use crate::cmd::arg::{ArgStore, CmdArg};

/// The housekeeping command definition.
pub struct CmdHousekeeping;

impl CmdHousekeeping {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("housekeeping")
            .about("Housekeeping utilities")
            .alias("housekeep")
            .alias("hk")
            .setting(AppSettings::SubcommandRequiredElseHelp)
            .subcommand(recrypt::CmdRecrypt::build())
            .subcommand(run::CmdRun::build())
            .subcommand(sync_keys::CmdSyncKeys::build())
            .arg(ArgStore::build())
    }
}
