#[cfg(feature = "clipboard")]
pub mod clip_revert;
pub mod completions;

use clap::{App, AppSettings};

/// The internal command definition.
pub struct CmdInternal;

impl CmdInternal {
    pub fn build<'a>() -> App<'a> {
        #[allow(unused)]
        let mut cmd = App::new("internal")
            .about("Commands used by prs internally")
            .setting(AppSettings::Hidden)
            .setting(AppSettings::SubcommandRequiredElseHelp)
            .subcommand(completions::CmdCompletions::build());

        #[cfg(feature = "clipboard")]
        {
            cmd = cmd.subcommand(clip_revert::CmdClipRevert::build());
        }

        cmd
    }
}
