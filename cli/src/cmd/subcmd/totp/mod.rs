pub mod show;

use clap::Command;

use crate::cmd::arg::{ArgStore, CmdArg};

/// The TOTP command definition.
pub struct CmdTotp;

impl CmdTotp {
    pub fn build() -> Command {
        Command::new("totp")
            .about("Manage password store TOTP tokens")
            .arg_required_else_help(true)
            .subcommand_required(true)
            .subcommand_value_name("CMD")
            .subcommand(show::CmdShow::build())
            .arg(ArgStore::build())
    }
}
