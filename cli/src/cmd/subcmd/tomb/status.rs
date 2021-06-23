use clap::App;

/// The tomb status command definition.
pub struct CmdStatus;

impl CmdStatus {
    pub fn build<'a>() -> App<'a> {
        App::new("status").about("Query tomb status")
    }
}
