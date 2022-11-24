use clap::{Arg, Command};

/// The tomb close command definition.
pub struct CmdClose;

impl CmdClose {
    pub fn build() -> Command {
        Command::new("close")
            .alias("c")
            .alias("stop")
            .about("Close tomb")
            .arg(
                Arg::new("try")
                    .long("try")
                    .num_args(0)
                    .help("Try to close, don't fail if already closed"),
            )
    }
}
