use clap::{App, Arg};

use crate::cmd::arg::{ArgStore, CmdArg};

/// The clone command definition.
pub struct CmdClone;

impl CmdClone {
    pub fn build<'a>() -> App<'a> {
        App::new("clone")
            .about("Clone existing password store")
            .arg(
                Arg::new("GIT_URL")
                    .help("Remote git URL to clone from")
                    .required(true),
            )
            .arg(ArgStore::build())
    }
}
