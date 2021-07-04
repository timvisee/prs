use clap::{App, Arg};

/// The tomb init command definition.
pub struct CmdInit;

impl CmdInit {
    pub fn build<'a>() -> App<'a> {
        App::new("init")
            .alias("initialize")
            .about("Initialize tomb in-place for current password store")
            .arg(
                Arg::new("timer")
                    .long("timer")
                    .short('t')
                    .alias("time")
                    .value_name("TIME")
                    // TODO: get value from prs_lib::tomb::TOMB_AUTO_CLOSE_SEC
                    .default_value("5m")
                    .about("Time after which to close the Tomb"),
            )
    }
}
