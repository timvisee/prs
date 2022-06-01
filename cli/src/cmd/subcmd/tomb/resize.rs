use clap::{Arg, Command};

/// The tomb resize command definition.
pub struct CmdResize;

impl CmdResize {
    pub fn build<'a>() -> Command<'a> {
        Command::new("resize")
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
