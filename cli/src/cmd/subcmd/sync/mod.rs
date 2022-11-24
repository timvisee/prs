pub mod init;
pub mod remote;

use clap::Command;

use crate::cmd::arg::{ArgAllowDirty, ArgStore, CmdArg};

/// The sync command definition.
pub struct CmdSync;

impl CmdSync {
    pub fn build() -> Command {
        Command::new("sync")
            .about("Sync password store")
            .subcommand(init::CmdInit::build())
            .subcommand(remote::CmdRemote::build())
            .arg(ArgStore::build())
            .arg(ArgAllowDirty::build())
    }
}
