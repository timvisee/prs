use clap::Arg;

use super::{CmdArg, CmdArgFlag};

/// The viewer argument.
pub struct ArgViewer {}

impl CmdArg for ArgViewer {
    fn name() -> &'static str {
        "viewer"
    }

    fn build() -> Arg {
        Arg::new("viewer")
            .long("viewer")
            .short('V')
            .alias("pager")
            .num_args(0)
            .global(true)
            .help("Show output in secure viewer")
    }
}

impl CmdArgFlag for ArgViewer {}
