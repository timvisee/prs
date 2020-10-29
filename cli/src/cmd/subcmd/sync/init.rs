use clap::{App, SubCommand};

/// The sync init command definition.
pub struct CmdInit;

impl CmdInit {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("init")
            .alias("initialize")
            .about("Initialize sync")
    }
}
