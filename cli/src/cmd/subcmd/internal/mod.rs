#[cfg(feature = "clipboard")]
pub mod clip_revert;

use clap::{App, AppSettings, SubCommand};

/// The internal command definition.
pub struct CmdInternal;

impl CmdInternal {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        #[allow(unused)]
        let mut cmd = SubCommand::with_name("_internal")
            .about("Commands used by prs internally")
            .setting(AppSettings::Hidden)
            .setting(AppSettings::SubcommandRequiredElseHelp);

        #[cfg(feature = "clipboard")]
        {
            cmd = cmd.subcommand(clip_revert::CmdClipRevert::build());
        }

        cmd
    }
}
