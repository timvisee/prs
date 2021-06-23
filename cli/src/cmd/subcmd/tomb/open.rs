use clap::App;

/// The tomb open command definition.
pub struct CmdOpen;

impl CmdOpen {
    pub fn build<'a>() -> App<'a> {
        // TODO: add parameters for setting auto close timer
        App::new("open").alias("o").about("Open tomb")
    }
}
