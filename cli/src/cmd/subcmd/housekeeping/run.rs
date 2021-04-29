use clap::App;

use crate::cmd::arg::{ArgAllowDirty, ArgNoSync, CmdArg};

/// The housekeeping run command definition.
pub struct CmdRun;

impl CmdRun {
    pub fn build<'a>() -> App<'a> {
        App::new("run")
            .about("Run housekeeping tasks")
            .arg(ArgAllowDirty::build())
            .arg(ArgNoSync::build())
    }
}
