use clap::{Arg, Command};

/// The tomb status command definition.
pub struct CmdStatus;

impl CmdStatus {
    pub fn build<'a>() -> Command<'a> {
        Command::new("status").about("Query tomb status").arg(
            Arg::new("open")
                .long("open")
                .alias("o")
                .help("Open tomb is it is closed"),
        )
    }
}
