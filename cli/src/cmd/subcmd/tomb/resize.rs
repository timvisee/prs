use clap::{App, Arg};

/// The tomb resize command definition.
pub struct CmdResize;

impl CmdResize {
    pub fn build<'a>() -> App<'a> {
        App::new("resize")
            .alias("r")
            .alias("size")
            .alias("grow")
            .about("Resize tomb")
            .arg(
                Arg::new("size")
                    .long("size")
                    .short('S')
                    .value_name("MEGABYTE")
                    .help("Resize tomb to megabytes"),
            )
    }
}
