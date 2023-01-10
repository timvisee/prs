use clap::Command;

use crate::cmd::arg::{ArgTimeout, CmdArg};

/// The internal TOTP recopy command definition.
pub struct CmdTotpRecopy;

impl CmdTotpRecopy {
    pub fn build() -> Command {
        Command::new("totp-recopy")
            .about("Copy TOTP tokens, recopy on change")
            .arg(ArgTimeout::build().required(true))
    }
}
