pub mod commit;
pub mod init;
pub mod remote;

use clap::Command;

use crate::cmd::arg::{ArgAllowDirty, CmdArg};

/// The sync command definition.
pub struct CmdSync;

impl CmdSync {
    pub fn build() -> Command {
        Command::new("sync")
            .about("Sync password store")
            .subcommand(commit::CmdCommit::build())
            .subcommand(init::CmdInit::build())
            .subcommand(remote::CmdRemote::build())
            .arg(ArgAllowDirty::build())
    }
}
