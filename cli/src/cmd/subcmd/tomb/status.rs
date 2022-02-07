use clap::{App, Arg};

/// The tomb status command definition.
pub struct CmdStatus;

impl CmdStatus {
    pub fn build<'a>() -> App<'a> {
        App::new("status").about("Query tomb status").arg(
            Arg::new("open")
                .long("open")
                .alias("o")
                .help("Open tomb is it is closed"),
        )
    }
}
