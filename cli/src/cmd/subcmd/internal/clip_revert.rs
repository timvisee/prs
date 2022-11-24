use clap::{Arg, Command};

use crate::cmd::arg::{ArgTimeout, CmdArg};

/// The internal clipboard revert command definition.
pub struct CmdClipRevert;

impl CmdClipRevert {
    pub fn build() -> Command {
        Command::new("clip-revert")
            .about("Revert clipboard after timeout")
            .arg(
                Arg::new("previous-base64-stdin")
                    .long("previous-base64-stdin")
                    .num_args(0)
                    .help("Read previous contents from stdin as base64 line"),
            )
            .arg(ArgTimeout::build())
    }
}
