pub mod list;

use clap::{App, AppSettings, SubCommand};

use list::CmdList;

/// The recipients command definition.
pub struct CmdRecipients;

impl CmdRecipients {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("recipients")
            .about("Manage store recipients")
            .alias("recipient")
            .alias("recip")
            .alias("rec")
            .alias("keys")
            .alias("kes")
            .setting(AppSettings::SubcommandRequiredElseHelp)
            .subcommand(CmdList::build())
    }
}
