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
            .setting(AppSettings::SubcommandRequiredElseHelp)
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
            .subcommand(subcmd::CmdCopy::build())
            .subcommand(subcmd::CmdDelete::build())
            .subcommand(subcmd::CmdDuplicate::build())
            .subcommand(subcmd::CmdEdit::build())
            .subcommand(subcmd::CmdList::build())
            .subcommand(subcmd::CmdMove::build())
            .subcommand(subcmd::CmdShow::build());

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

    /// Get the copy sub command, if matched.
    pub fn copy(&'a self) -> Option<matcher::CopyMatcher> {
        matcher::CopyMatcher::with(&self.matches)
    }

    /// Get the delete sub command, if matched.
    pub fn delete(&'a self) -> Option<matcher::DeleteMatcher> {
        matcher::DeleteMatcher::with(&self.matches)
    }

    /// Get the duplicate sub command, if matched.
    pub fn duplicate(&'a self) -> Option<matcher::DuplicateMatcher> {
        matcher::DuplicateMatcher::with(&self.matches)
    }

    /// Get the edit sub command, if matched.
    pub fn edit(&'a self) -> Option<matcher::EditMatcher> {
        matcher::EditMatcher::with(&self.matches)
    }

    /// Get the list sub command, if matched.
    pub fn list(&'a self) -> Option<matcher::ListMatcher> {
        matcher::ListMatcher::with(&self.matches)
    }

    /// Get the move sub command, if matched.
    pub fn r#move(&'a self) -> Option<matcher::MoveMatcher> {
        matcher::MoveMatcher::with(&self.matches)
    }

    /// Get the show sub command, if matched.
    pub fn show(&'a self) -> Option<matcher::ShowMatcher> {
        matcher::ShowMatcher::with(&self.matches)
    }
}
