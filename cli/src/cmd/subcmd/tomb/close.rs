use clap::{App, Arg};

/// The tomb close command definition.
pub struct CmdClose;

impl CmdClose {
    pub fn build<'a>() -> App<'a> {
        App::new("close")
            .alias("c")
            .alias("stop")
            .about("Close tomb")
            .arg(
                Arg::new("try")
                    .long("try")
                    .about("Try to close, don't fail if already closed"),
            )
    }
}
