use clap::{Arg, Command};

use crate::cmd::arg::{ArgProperty, ArgQuery, CmdArg};

/// The TOTP live command definition.
pub struct CmdLive;

impl CmdLive {
    pub fn build() -> Command {
        Command::new("live")
            .alias("watch")
            .alias("follow")
            .alias("l")
            .alias("w")
            .alias("f")
            .about("Watch TOTP token")
            .arg(ArgQuery::build())
            .arg(ArgProperty::build())
            .arg(
                Arg::new("follow")
                    .long("follow")
                    .short('F')
                    .num_args(0)
                    .help("Output new tokens on newline without clearing previous"),
            )
    }
}
