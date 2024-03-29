#[cfg(feature = "clipboard")]
pub mod copy;
pub mod live;
pub mod qr;
pub mod show;

use clap::Command;

/// The TOTP command definition.
pub struct CmdTotp;

impl CmdTotp {
    pub fn build() -> Command {
        let cmd = Command::new("totp")
            .alias("otp")
            .alias("hotp")
            .about("Manage TOTP tokens")
            .arg_required_else_help(true)
            .subcommand_required(true)
            .subcommand_value_name("CMD")
            .subcommand(show::CmdShow::build());

        #[cfg(feature = "clipboard")]
        let cmd = cmd.subcommand(copy::CmdCopy::build());

        cmd.subcommand(live::CmdLive::build())
            .subcommand(qr::CmdQr::build())
    }
}
