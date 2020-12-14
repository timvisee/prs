use clap::{App, Arg, SubCommand};

use crate::cmd::arg::{ArgTimeout, CmdArg};

/// The internal clipboard revert command definition.
pub struct CmdClipRevert;

impl CmdClipRevert {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("clip-revert")
            .about("Revert clipboard after timeout")
            .arg(
                Arg::with_name("previous-base64-stdin")
                    .long("previous-base64-stdin")
                    .help("Read previous contents from stdin as base64 line"),
            )
            .arg(ArgTimeout::build())
    }
}
