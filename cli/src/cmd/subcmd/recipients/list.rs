use clap::Command;

/// The recipient list command definition.
pub struct CmdList;

impl CmdList {
    pub fn build<'a>() -> Command<'a> {
        Command::new("list")
            .alias("ls")
            .alias("l")
            .about("List store recipients")
    }
}
