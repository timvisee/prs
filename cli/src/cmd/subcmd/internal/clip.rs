use clap::Command;

/// The internal clipboard command definition.
pub struct CmdClip;

impl CmdClip {
    pub fn build() -> Command {
        Command::new("clip").about("Set clipboard contents from stdin")
    }
}
