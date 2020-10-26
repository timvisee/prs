use clap::{App, SubCommand};

/// The recipient list command definition.
pub struct CmdList;

impl CmdList {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("list")
            .alias("ls")
            .alias("l")
            .about("List store recipients")
    }
}
