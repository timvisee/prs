pub mod add;
pub mod export;
pub mod generate;
pub mod list;
pub mod remove;

use clap::Command;

/// The recipients command definition.
pub struct CmdRecipients;

impl CmdRecipients {
    pub fn build() -> Command {
        Command::new("recipients")
            .about("Manage store recipients")
            .alias("recipient")
            .alias("recip")
            .alias("rec")
            .alias("keys")
            .alias("kes")
            .arg_required_else_help(true)
            .subcommand_required(true)
            .subcommand_value_name("CMD")
            .subcommand(add::CmdAdd::build())
            .subcommand(generate::CmdGenerate::build())
            .subcommand(list::CmdList::build())
            .subcommand(remove::CmdRemove::build())
            .subcommand(export::CmdExport::build())
    }
}
