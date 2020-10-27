use clap::{App, SubCommand};

/// The recipient add command definition.
pub struct CmdAdd;

impl CmdAdd {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("add")
            .alias("a")
            .about("Add store recipient")
    }
}
