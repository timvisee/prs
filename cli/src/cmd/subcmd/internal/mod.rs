#[cfg(feature = "clipboard")]
pub mod clip_revert;
pub mod completions;

use clap::Command;

/// The internal command definition.
pub struct CmdInternal;

impl CmdInternal {
    pub fn build<'a>() -> Command<'a> {
        #[allow(unused_mut)]
        let mut cmd = Command::new("internal")
            .about("Commands used by prs internally")
            .hide(true)
            .arg_required_else_help(true)
            .subcommand(completions::CmdCompletions::build());

        #[cfg(feature = "clipboard")]
        {
            cmd = cmd.subcommand(clip_revert::CmdClipRevert::build());
        }

        cmd
    }
}
