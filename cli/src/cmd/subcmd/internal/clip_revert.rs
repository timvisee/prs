use clap::{App, Arg};

use crate::cmd::arg::{ArgTimeout, CmdArg};

/// The internal clipboard revert command definition.
pub struct CmdClipRevert;

impl CmdClipRevert {
    pub fn build<'a>() -> App<'a> {
        App::new("clip-revert")
            .about("Revert clipboard after timeout")
            .arg(
                Arg::new("previous-base64-stdin")
                    .long("previous-base64-stdin")
                    .help("Read previous contents from stdin as base64 line"),
            )
            .arg(ArgTimeout::build())
    }
}
