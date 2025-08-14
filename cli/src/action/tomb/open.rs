use anyhow::{Result, anyhow};
use clap::ArgMatches;
use prs_lib::Store;
use prs_lib::tomb::Tomb;
use thiserror::Error;

use crate::{
    cmd::matcher::{
        MainMatcher, Matcher,
        tomb::{TombMatcher, open::OpenMatcher},
    },
    util::{
        self,
        error::{self, ErrorHintsBuilder},
        style,
    },
};

/// A tomb open action.
pub struct Open<'a> {
    cmd_matches: &'a ArgMatches,
}

impl<'a> Open<'a> {
    /// Construct a new init action.
    pub fn new(cmd_matches: &'a ArgMatches) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the init action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let _matcher_tomb = TombMatcher::with(self.cmd_matches).unwrap();
        let matcher_open = OpenMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(matcher_main.store()).map_err(Err::Store)?;
        let mut tomb = store.tomb(
            !matcher_main.verbose(),
            matcher_main.verbose(),
            matcher_main.force(),
        );
        let timer = matcher_open.timer();

        // TODO: show warning if there already are files in tomb directory?

        // Must be a tomb
        if !tomb.is_tomb() && !matcher_main.force() {
            // TODO: error hint to initialize tomb
            error::quit_error_msg(
                "password store is not a tomb",
                ErrorHintsBuilder::from_matcher(&matcher_main)
                    .force(true)
                    .build()
                    .unwrap(),
            );
        }

        // Must not be already open
        if tomb.is_open().map_err(Err::Open)? && !matcher_main.force() {
            error::quit_error_msg(
                "password store tomb is already open",
                ErrorHintsBuilder::from_matcher(&matcher_main)
                    .force(true)
                    .build()
                    .unwrap(),
            );
        }

        // Open the tomb
        open(&mut tomb, &matcher_main)?;

        // Start timer
        if let Some(timer) = timer {
            if let Err(err) = tomb.stop_timer() {
                error::print_error(err.context(
                    "failed to stop existing timer to automatically close password store tomb, ignoring",
                ));
            }
            tomb.start_timer(timer, true).map_err(Err::Timer)?;
        }

        if !matcher_main.quiet() {
            if let Some(timer) = timer {
                eprintln!(
                    "Password store Tomb opened, will close in {}",
                    util::time::format_duration(timer)
                );
            } else {
                eprintln!("Password store Tomb opened");
            }
            eprintln!();
            eprintln!("To close the Tomb, use:");
            eprintln!(
                "    {}",
                style::highlight(format!("{} tomb close", util::bin_name()))
            );
        }

        Ok(())
    }
}

/// Open the tomb.
pub(crate) fn open(tomb: &mut Tomb, matcher_main: &MainMatcher) -> Result<(), Err> {
    // Prompt user to add force flag
    if !tomb.settings.force && util::tomb::ask_to_force(matcher_main) {
        tomb.settings.force = true;
    }

    if matcher_main.verbose() {
        eprintln!("Opening Tomb...");
    }

    // Open the tomb
    let errs = tomb.open().map_err(Err::Open)?;

    // Report soft-fail errors to the user
    let show_error_hints = !errs.is_empty();
    for err in errs {
        error::print_error(
            anyhow!(err).context("failed to run housekeeping task after opening tomb, ignoring"),
        );
    }
    if show_error_hints {
        error::ErrorHintsBuilder::from_matcher(matcher_main)
            .force(true)
            .verbose(true)
            .build()
            .unwrap()
            .print(true);
    }

    Ok(())
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),

    #[error("failed to open password store tomb")]
    Open(#[source] anyhow::Error),

    #[error("failed to start timer to automatically close password store tomb")]
    Timer(#[source] anyhow::Error),
}
