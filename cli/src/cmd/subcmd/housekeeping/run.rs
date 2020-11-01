use clap::{App, SubCommand};

/// The housekeeping run command definition.
pub struct CmdRun;

impl CmdRun {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("run").about("Run housekeeping tasks")
    }
}
