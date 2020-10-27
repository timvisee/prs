use clap::{App, SubCommand};

/// The recipient remove command definition.
pub struct CmdRemove;

impl CmdRemove {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("remove")
            .alias("rm")
            .alias("delete")
            .alias("del")
            .about("Remove store recipient")
    }
}
