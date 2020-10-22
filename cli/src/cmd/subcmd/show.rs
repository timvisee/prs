use clap::{App, SubCommand};

/// The show command definition.
pub struct CmdShow;

impl CmdShow {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("show")
            .about("Display a secret")
            .alias("s")
            .alias("cat")
            .alias("display")
    }
}
