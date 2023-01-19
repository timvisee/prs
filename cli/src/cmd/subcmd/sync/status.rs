use clap::Command;

/// The sync status command definition.
pub struct CmdStatus;

impl CmdStatus {
    pub fn build() -> Command {
        Command::new("status").about("Show sync status")
    }
}
