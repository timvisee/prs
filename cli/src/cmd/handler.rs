use clap::{Arg, ArgAction, ArgMatches, Command};

use super::arg::{ArgStore, CmdArg};
use super::matcher::{self, Matcher};
use super::subcmd;

/// Custom template for help
const HELP_TEMPLATE: &str = "\
{before-help}{name} {version}
{author-with-newline}{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}
";

/// CLI argument handler.
pub struct Handler {
    /// The CLI matches.
    matches: ArgMatches,
}

impl<'a> Handler {
    /// Build the application CLI definition.
    pub fn build() -> Command {
        // Build the CLI application definition
        let app = Command::new(crate_name!())
            .version(crate_version!())
            .author(crate_authors!())
            .about(crate_description!())
            .help_template(HELP_TEMPLATE)
            .arg(
                Arg::new("force")
                    .long("force")
                    .short('f')
                    .num_args(0)
                    .global(true)
                    .help("Force the action, ignore warnings"),
            )
            .arg(
                Arg::new("no-interact")
                    .long("no-interact")
                    .short('I')
                    .alias("no-interactive")
                    .alias("non-interactive")
                    .num_args(0)
                    .global(true)
                    .help("Not interactive, do not prompt"),
            )
            .arg(
                Arg::new("yes")
                    .long("yes")
                    .short('y')
                    .alias("assume-yes")
                    .num_args(0)
                    .global(true)
                    .help("Assume yes for prompts"),
            )
            .arg(
                Arg::new("quiet")
                    .long("quiet")
                    .short('q')
                    .num_args(0)
                    .global(true)
                    .help("Produce output suitable for logging and automation"),
            )
            .arg(
                Arg::new("verbose")
                    .long("verbose")
                    .short('v')
                    .action(ArgAction::Count)
                    .num_args(0)
                    .global(true)
                    .help("Enable verbose information and logging"),
            )
            .arg(ArgStore::build())
            .arg(
                Arg::new("gpg-tty")
                    .long("gpg-tty")
                    .num_args(0)
                    .global(true)
                    .help("Instruct GPG to ask passphrase in TTY rather than pinentry"),
            )
            .subcommand(subcmd::CmdShow::build());

        #[cfg(feature = "clipboard")]
        let app = app.subcommand(subcmd::CmdCopy::build());

        let app = app
            .subcommand(subcmd::CmdGenerate::build())
            .subcommand(subcmd::CmdAdd::build())
            .subcommand(subcmd::CmdEdit::build())
            .subcommand(subcmd::CmdDuplicate::build());

        #[cfg(feature = "alias")]
        let app = app.subcommand(subcmd::CmdAlias::build());

        let app = app
            .subcommand(subcmd::CmdMove::build())
            .subcommand(subcmd::CmdRemove::build())
            .subcommand(subcmd::CmdList::build())
            .subcommand(subcmd::CmdInit::build())
            .subcommand(subcmd::CmdClone::build())
            .subcommand(subcmd::CmdSync::build())
            .subcommand(subcmd::CmdSlam::build());

        #[cfg(feature = "totp")]
        let app = app.subcommand(subcmd::CmdTotp::build());

        let app = app.subcommand(subcmd::CmdRecipients::build())
            .subcommand(subcmd::CmdGit::build());

        #[cfg(all(feature = "tomb", target_os = "linux"))]
        let app = app.subcommand(subcmd::CmdTomb::build());

        #[allow(clippy::let_and_return)]
        let app = app
            .subcommand(subcmd::CmdHousekeeping::build())
            .subcommand(subcmd::CmdInternal::build());

        // Disable color usage if compiled without color support
        // TODO: do not use feature, pull from env var instead
        #[cfg(feature = "no-color")]
        let app = app.global_setting(AppSettings::ColorNever);

        app
    }

    /// Parse CLI arguments.
    pub fn parse() -> Handler {
        Handler {
            matches: Handler::build().get_matches(),
        }
    }

    /// Get the raw matches.
    pub fn matches(&'a self) -> &'a ArgMatches {
        &self.matches
    }

    /// Get the add sub command, if matched.
    pub fn add(&'a self) -> Option<matcher::AddMatcher> {
        matcher::AddMatcher::with(&self.matches)
    }

    /// Get the alias sub command, if matched.
    #[cfg(feature = "alias")]
    pub fn alias(&'a self) -> Option<matcher::AliasMatcher> {
        matcher::AliasMatcher::with(&self.matches)
    }

    /// Get the clone sub command, if matched.
    pub fn clone(&'a self) -> Option<matcher::CloneMatcher> {
        matcher::CloneMatcher::with(&self.matches)
    }

    /// Get the copy sub command, if matched.
    #[cfg(feature = "clipboard")]
    pub fn copy(&'a self) -> Option<matcher::CopyMatcher> {
        matcher::CopyMatcher::with(&self.matches)
    }

    /// Get the duplicate sub command, if matched.
    pub fn duplicate(&'a self) -> Option<matcher::DuplicateMatcher> {
        matcher::DuplicateMatcher::with(&self.matches)
    }

    /// Get the edit sub command, if matched.
    pub fn edit(&'a self) -> Option<matcher::EditMatcher> {
        matcher::EditMatcher::with(&self.matches)
    }

    /// Get the generate sub command, if matched.
    pub fn generate(&'a self) -> Option<matcher::GenerateMatcher> {
        matcher::GenerateMatcher::with(&self.matches)
    }

    /// Get the git sub command, if matched.
    pub fn git(&'a self) -> Option<matcher::GitMatcher> {
        matcher::GitMatcher::with(&self.matches)
    }

    /// Get the housekeeping sub command, if matched.
    pub fn housekeeping(&'a self) -> Option<matcher::HousekeepingMatcher> {
        matcher::HousekeepingMatcher::with(&self.matches)
    }

    /// Get the init sub command, if matched.
    pub fn init(&'a self) -> Option<matcher::InitMatcher> {
        matcher::InitMatcher::with(&self.matches)
    }

    /// Get the internal sub command, if matched.
    pub fn internal(&'a self) -> Option<matcher::InternalMatcher> {
        matcher::InternalMatcher::with(&self.matches)
    }

    /// Get the list sub command, if matched.
    pub fn list(&'a self) -> Option<matcher::ListMatcher> {
        matcher::ListMatcher::with(&self.matches)
    }

    /// Get the slam sub command, if matched.
    pub fn slam(&'a self) -> Option<matcher::SlamMatcher> {
        matcher::SlamMatcher::with(&self.matches)
    }

    /// Get the move sub command, if matched.
    pub fn r#move(&'a self) -> Option<matcher::MoveMatcher> {
        matcher::MoveMatcher::with(&self.matches)
    }

    /// Get the recipients sub command, if matched.
    pub fn recipients(&'a self) -> Option<matcher::RecipientsMatcher> {
        matcher::RecipientsMatcher::with(&self.matches)
    }

    /// Get the remove sub command, if matched.
    pub fn remove(&'a self) -> Option<matcher::RemoveMatcher> {
        matcher::RemoveMatcher::with(&self.matches)
    }

    /// Get the show sub command, if matched.
    pub fn show(&'a self) -> Option<matcher::ShowMatcher> {
        matcher::ShowMatcher::with(&self.matches)
    }

    /// Get the sync sub command, if matched.
    pub fn sync(&'a self) -> Option<matcher::SyncMatcher> {
        matcher::SyncMatcher::with(&self.matches)
    }

    /// Get the tomb sub command, if matched.
    #[cfg(all(feature = "tomb", target_os = "linux"))]
    pub fn tomb(&'a self) -> Option<matcher::TombMatcher> {
        matcher::TombMatcher::with(&self.matches)
    }

    /// Get the TOTP sub command, if matched.
    #[cfg(feature = "totp")]
    pub fn totp(&'a self) -> Option<matcher::TotpMatcher> {
        matcher::TotpMatcher::with(&self.matches)
    }
}
