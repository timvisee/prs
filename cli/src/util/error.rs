use std::borrow::Borrow;
use std::fmt::{Debug, Display};
use std::io::{self, Write};
pub use std::process::exit;

use anyhow::anyhow;

use crate::cmd::matcher::MainMatcher;
use crate::util::style::{highlight, highlight_error, highlight_info, highlight_warning};

// /// Print a success message.
// pub fn print_success(msg: &str) {
//     eprintln!("{}", msg.green());
// }

/// Print the given error in a proper format for the user,
/// with it's causes.
pub fn print_error(err: anyhow::Error) {
    // Report each printable error, count them
    let count = err
        .chain()
        .map(|err| format!("{}", err))
        .filter(|err| !err.is_empty())
        .enumerate()
        .map(|(i, err)| {
            if i == 0 {
                eprintln!("{} {}", highlight_error("error:"), err);
            } else {
                eprintln!("{} {}", highlight_error("caused by:"), err);
            }
        })
        .count();

    // Fall back to a basic message
    if count == 0 {
        eprintln!("{} an undefined error occurred", highlight_error("error:"),);
    }
}

/// Print the given error message in a proper format for the user,
/// with it's causes.
pub fn print_error_msg<S>(err: S)
where
    S: AsRef<str> + Display + Debug + Sync + Send + 'static,
{
    print_error(anyhow!(err));
}

/// Print a warning.
pub fn print_warning<S>(err: S)
where
    S: AsRef<str> + Display + Debug + Sync + Send + 'static,
{
    eprintln!("{} {}", highlight_warning("warning:"), err);
}

/// Quit the application regularly.
pub fn quit() -> ! {
    exit(0);
}

/// Quit the application with an error code,
/// and print the given error.
pub fn quit_error(err: anyhow::Error, hints: impl Borrow<ErrorHints>) -> ! {
    // Print the error
    print_error(err);

    // Print error hints
    hints.borrow().print(false);

    // Quit
    exit(1);
}

/// Quit the application with an error code,
/// and print the given error message.
pub fn quit_error_msg<S>(err: S, hints: impl Borrow<ErrorHints>) -> !
where
    S: AsRef<str> + Display + Debug + Sync + Send + 'static,
{
    quit_error(anyhow!(err), hints);
}

/// The error hint configuration.
#[derive(Clone, Builder)]
#[builder(default)]
pub struct ErrorHints {
    /// A list of info messages to print along with the error.
    info: Vec<String>,

    /// Show about the sync action.
    sync: bool,

    /// Show about the sync init action.
    sync_init: bool,

    /// Show about the sync remote action.
    sync_remote: bool,

    /// Show about the git action.
    git: bool,

    /// Show allow dirty flag.
    allow_dirty: bool,

    /// Show about the force flag.
    force: bool,

    /// Show about the verbose flag.
    verbose: bool,

    /// Show about the help flag.
    help: bool,
}

impl ErrorHints {
    /// Check whether any hint should be printed.
    pub fn any(&self) -> bool {
        !self.info.is_empty()
            || self.sync
            || self.sync_init
            || self.sync_remote
            || self.git
            || self.allow_dirty
            || self.force
            || self.verbose
            || self.help
    }

    /// Print the error hints.
    pub fn print(&self, end_newline: bool) {
        // Print info messages
        for msg in &self.info {
            eprintln!("{} {}", highlight_info("info:"), msg);
        }

        // Stop if nothing should be printed
        if !self.any() {
            return;
        }

        eprintln!();

        // Print hints
        let bin = crate::util::bin_name();
        if self.sync {
            eprintln!(
                "To sync your password store use '{}'",
                highlight(&format!("{} sync", bin))
            );
        }
        if self.sync_init {
            eprintln!(
                "To initialize sync for your password store use '{}'",
                highlight(&format!("{} sync init", bin))
            );
        }
        if self.sync_remote {
            eprintln!(
                "Use '{}' to get or set a remote sync URL",
                highlight(&format!("{} sync remote [GIT_URL]", bin))
            );
        }
        if self.git || self.allow_dirty {
            eprintln!(
                "Use '{}' to show sync status, uncommitted changes and help",
                highlight(&format!("{} sync status", bin))
            );
        }
        if self.git {
            eprintln!(
                "Use '{}' to inspect or resolve this issue using git",
                highlight(&format!("{} git", bin))
            );
        }
        if self.allow_dirty {
            eprintln!(
                "To make changes while the store repository is dirty add '{}'",
                highlight("--allow-dirty")
            );
        }
        if self.force {
            eprintln!("Use '{}' to force", highlight("--force"));
        }
        if self.verbose {
            eprintln!("For a detailed log add '{}'", highlight("--verbose"));
        }
        if self.help {
            eprintln!("For more information add '{}'", highlight("--help"));
        }

        // End with additional newline
        if end_newline {
            eprintln!();
        }

        // Flush
        let _ = io::stderr().flush();
    }

    /// Construct an error hints object with defaults based on the main matcher.
    pub fn from_matcher(matcher_main: &MainMatcher) -> Self {
        ErrorHintsBuilder::from_matcher(matcher_main)
            .build()
            .unwrap()
    }
}

impl Default for ErrorHints {
    fn default() -> Self {
        ErrorHints {
            info: Vec::new(),
            sync: false,
            sync_init: false,
            sync_remote: false,
            git: false,
            allow_dirty: false,
            force: false,
            verbose: true,
            help: true,
        }
    }
}

impl ErrorHintsBuilder {
    /// Construct an error hints object with defaults based on the main matcher.
    pub fn from_matcher(matcher_main: &MainMatcher) -> Self {
        let mut builder = Self::default();

        if matcher_main.force() {
            builder.force = Some(false);
        }
        if matcher_main.verbose() {
            builder.verbose = Some(false);
        }

        builder
    }

    /// Add a single info entry.
    pub fn add_info(mut self, info: String) -> Self {
        // Initialize the info list
        if self.info.is_none() {
            self.info = Some(Vec::new());
        }

        // Add the item to the info list
        if let Some(ref mut list) = self.info {
            list.push(info);
        }

        self
    }
}
