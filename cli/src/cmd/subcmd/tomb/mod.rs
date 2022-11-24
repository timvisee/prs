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
    pub fn build() -> Command {
        Command::new("tomb")
            .about("Manage password store Tomb")
            .arg_required_else_help(true)
            .subcommand_required(true)
            .subcommand_value_name("CMD")
            .subcommand(init::CmdInit::build())
            .subcommand(open::CmdOpen::build())
            .subcommand(close::CmdClose::build())
            .subcommand(status::CmdStatus::build())
            .subcommand(resize::CmdResize::build())
            .arg(ArgStore::build())
    }
}
