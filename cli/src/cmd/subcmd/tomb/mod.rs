pub mod close;
pub mod init;
pub mod open;

use clap::{App, AppSettings};

use crate::cmd::arg::{ArgStore, CmdArg};

/// The tomb command definition.
pub struct CmdTomb;

impl CmdTomb {
    pub fn build<'a>() -> App<'a> {
        App::new("tomb")
            .about("Manage password store Tomb")
            .setting(AppSettings::SubcommandRequiredElseHelp)
            .subcommand(init::CmdInit::build())
            .subcommand(open::CmdOpen::build())
            .subcommand(close::CmdClose::build())
            .arg(ArgStore::build())
    }
}
