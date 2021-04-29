pub mod init;
pub mod remote;

use clap::App;

use crate::cmd::arg::{ArgAllowDirty, ArgStore, CmdArg};

/// The sync command definition.
pub struct CmdSync;

impl CmdSync {
    pub fn build<'a>() -> App<'a> {
        App::new("sync")
            .alias("s")
            .about("Sync password store")
            .subcommand(init::CmdInit::build())
            .subcommand(remote::CmdRemote::build())
            .arg(ArgStore::build())
            .arg(ArgAllowDirty::build())
    }
}
