use clap::{App, Arg};

/// The tomb open command definition.
pub struct CmdOpen;

impl CmdOpen {
    pub fn build<'a>() -> App<'a> {
        App::new("open").alias("o").about("Open tomb").arg(
            Arg::new("timer")
                .long("timer")
                .short('t')
                .alias("time")
                .value_name("TIME")
                .about("Time after which to close the Tomb"),
        )
    }
}
