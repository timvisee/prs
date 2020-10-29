use clap::{App, Arg, SubCommand};

/// The sync remote command definition.
pub struct CmdRemote;

impl CmdRemote {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("remote")
            .about("Get or set git remote URL for sync")
            .arg(Arg::with_name("GIT_URL").help("Remote git URL to set"))
    }
}
