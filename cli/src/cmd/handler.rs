use clap::{App, AppSettings, Arg, ArgMatches};

use super::matcher::{self, Matcher};
use super::subcmd;

/// CLI argument handler.
pub struct Handler<'a> {
    /// The CLI matches.
    matches: ArgMatches<'a>,
}

impl<'a: 'b, 'b> Handler<'a> {
    /// Build the application CLI definition.
    pub fn build() -> App<'a, 'b> {
        // Build the CLI application definition
        let app = App::new(crate_name!())
            .version(crate_version!())
            .author(crate_authors!())
            .about(crate_description!())
            .global_setting(AppSettings::GlobalVersion)
            .global_setting(AppSettings::VersionlessSubcommands)
            .arg(
                Arg::with_name("force")
                    .long("force")
                    .short("f")
                    .global(true)
                    .help("Force the action, ignore warnings"),
            )
            .arg(
                Arg::with_name("no-interact")
                    .long("no-interact")
                    .short("I")
                    .alias("no-interactive")
                    .alias("non-interactive")
                    .global(true)
                    .help("Not interactive, do not prompt"),
            )
            .arg(
                Arg::with_name("yes")
                    .long("yes")
                    .short("y")
                    .alias("assume-yes")
                    .global(true)
                    .help("Assume yes for prompts"),
            )
            .arg(
                Arg::with_name("quiet")
                    .long("quiet")
                    .short("q")
                    .global(true)
                    .help("Produce output suitable for logging and automation"),
            )
            .arg(
                Arg::with_name("verbose")
                    .long("verbose")
                    .short("v")
                    .multiple(true)
                    .global(true)
                    .help("Enable verbose information and logging"),
            )
            .subcommand(subcmd::CmdAdd::build())
            .subcommand(subcmd::CmdAlias::build())
            .subcommand(subcmd::CmdClone::build())
            .subcommand(subcmd::CmdDuplicate::build())
            .subcommand(subcmd::CmdEdit::build())
            .subcommand(subcmd::CmdGenerate::build())
            .subcommand(subcmd::CmdGit::build())
            .subcommand(subcmd::CmdHousekeeping::build())
            .subcommand(subcmd::CmdInit::build())
            .subcommand(subcmd::CmdInternal::build())
            .subcommand(subcmd::CmdList::build())
            .subcommand(subcmd::CmdMove::build())
            .subcommand(subcmd::CmdRecipients::build())
            .subcommand(subcmd::CmdRemove::build())
            .subcommand(subcmd::CmdShow::build())
            .subcommand(subcmd::CmdSync::build());

        #[cfg(feature = "clipboard")]
        let app = app.subcommand(subcmd::CmdCopy::build());

        // Disable color usage if compiled without color support
        // TODO: do not use feature, pull from env var instead
        #[cfg(feature = "no-color")]
        let app = app.global_setting(AppSettings::ColorNever);

        app
    }

    /// Parse CLI arguments.
    pub fn parse() -> Handler<'a> {
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
}
