pub mod init;
pub mod remote;

use clap::Command;

use crate::cmd::arg::{ArgAllowDirty, ArgStore, CmdArg};

/// The sync command definition.
pub struct CmdSync;

impl CmdSync {
    pub fn build<'a>() -> Command<'a> {
        Command::new("sync")
            .alias("s")
            .about("Sync password store")
            .subcommand(init::CmdInit::build())
            .subcommand(remote::CmdRemote::build())
            .arg(ArgStore::build())
            .arg(ArgAllowDirty::build())
    }
}
