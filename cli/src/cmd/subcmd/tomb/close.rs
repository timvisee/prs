use clap::App;

/// The tomb close command definition.
pub struct CmdClose;

impl CmdClose {
    pub fn build<'a>() -> App<'a> {
        App::new("close")
            .alias("c")
            .alias("stop")
            .about("Close tomb")
    }
}
