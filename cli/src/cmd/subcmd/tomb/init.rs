use clap::{App, Arg};

use crate::util::time;

lazy_static! {
    /// Default value for timer.
    static ref TIMER_DEFAULT: String = time::format_duration(prs_lib::tomb::TOMB_AUTO_CLOSE_SEC);
}

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
                    .default_value(&TIMER_DEFAULT)
                    .help("Time after which to close the Tomb"),
            )
    }
}
