use clap::{App, SubCommand};

/// The copy command definition.
pub struct CmdCopy;

impl CmdCopy {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("copy")
            .about("Copy secret to clipboard")
            .alias("c")
    }
}
