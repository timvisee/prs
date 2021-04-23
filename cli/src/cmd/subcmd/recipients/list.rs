use clap::App;

/// The recipient list command definition.
pub struct CmdList;

impl CmdList {
    pub fn build<'a>() -> App<'a> {
        App::new("list")
            .alias("ls")
            .alias("l")
            .about("List store recipients")
    }
}
