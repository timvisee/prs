use clap::{Arg, Command};

/// The tomb status command definition.
pub struct CmdStatus;

impl CmdStatus {
    pub fn build() -> Command {
        Command::new("status").about("Query tomb status").arg(
            Arg::new("open")
                .long("open")
                .alias("o")
                .num_args(0)
                .help("Open tomb is it is closed"),
        )
    }
}
