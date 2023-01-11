#[cfg(feature = "clipboard")]
pub mod clip;
#[cfg(feature = "clipboard")]
pub mod clip_revert;
pub mod completions;
#[cfg(all(feature = "clipboard", feature = "totp"))]
pub mod totp_recopy;

use clap::Command;

/// The internal command definition.
pub struct CmdInternal;

impl CmdInternal {
    pub fn build() -> Command {
        #[allow(unused_mut)]
        let mut cmd = Command::new("internal")
            .about("Commands used by prs internally")
            .hide(true)
            .arg_required_else_help(true)
            .subcommand_required(true)
            .subcommand_value_name("ACTION")
            .subcommand(completions::CmdCompletions::build());

        #[cfg(feature = "clipboard")]
        {
            cmd = cmd
                .subcommand(clip::CmdClip::build())
                .subcommand(clip_revert::CmdClipRevert::build());
        }

        #[cfg(all(feature = "clipboard", feature = "totp"))]
        {
            cmd = cmd.subcommand(totp_recopy::CmdTotpRecopy::build());
        }

        cmd
    }
}
