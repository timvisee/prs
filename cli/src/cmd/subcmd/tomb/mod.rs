pub mod close;
pub mod init;
pub mod open;
pub mod resize;
pub mod status;

use clap::Command;

use crate::cmd::arg::{ArgStore, CmdArg};

/// The tomb command definition.
pub struct CmdTomb;

impl CmdTomb {
    pub fn build<'a>() -> Command<'a> {
        Command::new("tomb")
            .about("Manage password store Tomb")
            .arg_required_else_help(true)
            .subcommand(init::CmdInit::build())
            .subcommand(open::CmdOpen::build())
            .subcommand(close::CmdClose::build())
            .subcommand(status::CmdStatus::build())
            .subcommand(resize::CmdResize::build())
            .arg(ArgStore::build())
    }
}
