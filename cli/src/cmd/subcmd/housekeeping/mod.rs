pub mod recrypt;

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
            .arg(ArgStore::build())
    }
}
