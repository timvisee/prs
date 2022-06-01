pub mod add;
pub mod export;
pub mod generate;
pub mod list;
pub mod remove;

use clap::Command;

use crate::cmd::arg::{ArgStore, CmdArg};

/// The recipients command definition.
pub struct CmdRecipients;

impl CmdRecipients {
    pub fn build<'a>() -> Command<'a> {
        Command::new("recipients")
            .about("Manage store recipients")
            .alias("recipient")
            .alias("recip")
            .alias("rec")
            .alias("keys")
            .alias("kes")
            .arg_required_else_help(true)
            .subcommand(add::CmdAdd::build())
            .subcommand(export::CmdExport::build())
            .subcommand(generate::CmdGenerate::build())
            .subcommand(list::CmdList::build())
            .subcommand(remove::CmdRemove::build())
            .arg(ArgStore::build())
    }
}
