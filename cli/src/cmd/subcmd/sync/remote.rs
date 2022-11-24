use clap::{Arg, Command};

/// The sync remote command definition.
pub struct CmdRemote;

impl CmdRemote {
    pub fn build() -> Command {
        Command::new("remote")
            .about("Get or set git remote URL for sync")
            .arg(Arg::new("GIT_URL").help("Remote git URL to set"))
    }
}
