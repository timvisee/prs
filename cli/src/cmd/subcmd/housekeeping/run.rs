use clap::App;

/// The housekeeping run command definition.
pub struct CmdRun;

impl CmdRun {
    pub fn build<'a>() -> App<'a> {
        App::new("run").about("Run housekeeping tasks")
    }
}
