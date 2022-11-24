use clap::{Arg, Command};

/// The tomb resize command definition.
pub struct CmdResize;

impl CmdResize {
    pub fn build() -> Command {
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
                    .num_args(1)
                    .help("Resize tomb to megabytes"),
            )
    }
}
