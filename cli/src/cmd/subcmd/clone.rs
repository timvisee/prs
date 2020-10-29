use clap::{App, Arg, SubCommand};

use crate::cmd::arg::{ArgStore, CmdArg};

/// The clone command definition.
pub struct CmdClone;

impl CmdClone {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("clone")
            .about("Clone existing password store")
            .arg(
                Arg::with_name("GIT_URL")
                    .help("Remote git URL to clone from")
                    .required(true),
            )
            .arg(ArgStore::build())
    }
}
