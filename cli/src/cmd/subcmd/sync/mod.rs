pub mod commit;
pub mod init;
pub mod remote;
pub mod reset;
pub mod status;

use clap::Command;

use crate::cmd::arg::{ArgAllowDirty, CmdArg};

/// The sync command definition.
pub struct CmdSync;

impl CmdSync {
    pub fn build() -> Command {
        Command::new("sync")
            .about("Sync password store")
            .subcommand(init::CmdInit::build())
            .subcommand(remote::CmdRemote::build())
            .subcommand(status::CmdStatus::build())
            .subcommand(commit::CmdCommit::build())
            .subcommand(reset::CmdReset::build())
            .arg(ArgAllowDirty::build())
    }
}
