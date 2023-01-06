use clap::Command;

use crate::cmd::arg::{ArgProperty, ArgQuery, CmdArg};

/// The TOTP QR code command definition.
pub struct CmdQr;

impl CmdQr {
    pub fn build() -> Command {
         Command::new("qr")
            .alias("q")
            .alias("qrcode")
            .alias("qr-code")
            .alias("share")
            .about("Show TOTP QR code")
            .arg(ArgQuery::build())
            .arg(ArgProperty::build())
    }
}
