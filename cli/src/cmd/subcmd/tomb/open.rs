use clap::{Arg, Command};

/// The tomb open command definition.
pub struct CmdOpen;

impl CmdOpen {
    pub fn build<'a>() -> Command<'a> {
        Command::new("open")
            .alias("o")
            .alias("lock")
            .about("Open tomb")
            .arg(
                Arg::new("timer")
                    .long("timer")
                    .short('t')
                    .alias("time")
                    .value_name("TIME")
                    .help("Time after which to close the Tomb"),
            )
    }
}
