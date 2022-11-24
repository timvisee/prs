use clap::Arg;

use super::{CmdArg, CmdArgFlag};

/// The allow-dirty argument.
pub struct ArgAllowDirty {}

impl CmdArg for ArgAllowDirty {
    fn name() -> &'static str {
        "allow-dirty"
    }

    fn build() -> Arg {
        Arg::new("allow-dirty")
            .long("allow-dirty")
            .short('d')
            .alias("dirty")
            .alias("sync-allow-dirty")
            .alias("sync-dirty")
            .num_args(0)
            .global(true)
            .help("Allow commit and sync on dirty store repository")
    }
}

impl CmdArgFlag for ArgAllowDirty {}
