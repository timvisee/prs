use clap::{App, Arg, SubCommand};

/// The housekeeping sync-keys command definition.
pub struct CmdSyncKeys;

impl CmdSyncKeys {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("sync-keys")
            .alias("sync-recipients")
            .about("Sync public keys in store, import missing keys")
            .arg(
                Arg::with_name("no-import")
                    .long("no-import")
                    .alias("skip-import")
                    .help("Skip importing missing keys to keychain"),
            )
    }
}
