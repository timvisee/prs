use clap::Command;

/// The sync init command definition.
pub struct CmdInit;

impl CmdInit {
    pub fn build() -> Command {
        Command::new("init")
            .alias("initialize")
            .about("Initialize sync")
    }
}
