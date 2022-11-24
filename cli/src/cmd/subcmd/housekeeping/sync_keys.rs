use clap::{Arg, Command};

use crate::cmd::arg::{ArgAllowDirty, ArgNoSync, CmdArg};

/// The housekeeping sync-keys command definition.
pub struct CmdSyncKeys;

impl CmdSyncKeys {
    pub fn build() -> Command {
        Command::new("sync-keys")
            .alias("sync-recipients")
            .about("Sync public keys in store, import missing keys")
            .arg(
                Arg::new("no-import")
                    .long("no-import")
                    .alias("skip-import")
                    .num_args(0)
                    .help("Skip importing missing keys to keychain"),
            )
            .arg(ArgAllowDirty::build())
            .arg(ArgNoSync::build())
    }
}
