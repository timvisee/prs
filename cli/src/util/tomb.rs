use anyhow::Result;
use prs_lib::tomb::Tomb;

use crate::cmd::matcher::MainMatcher;
use crate::util::{self, error, style};

/// Prepare Tomb.
pub fn prepare_tomb(tomb: &mut Tomb, matcher_main: &MainMatcher) -> Result<()> {
    // When opening a Tomb the user must force when SWAP is available, ask whether to force
    if !tomb.settings.force && tomb.is_tomb() {
        // Tomb must not be open yet, ignore errors
        if let Ok(false) = tomb.is_open() {
            if ask_to_force(matcher_main) {
                tomb.settings.force = true;
            }
        }
    }

    // Prepare as normal
    tomb.prepare()
}

/// Finalize Tomb.
pub fn finalize_tomb(tomb: &mut Tomb, matcher_main: &MainMatcher, changed: bool) -> Result<()> {
    // Ask to enlarge Tomb if it gets too small when contents changed
    if changed && !matcher_main.quiet() && tomb.is_tomb() && tomb.is_open().unwrap_or(false) {
        if let Ok(sizes) = tomb.fetch_size_stats() {
            if sizes.should_resize() {
                let bin = crate::util::bin_name();
                eprintln!();
                error::print_warning(
                    "your Tomb may not have enough space left for new password store changes.",
                );
                error::print_warning(format!(
                    "use '{}' to make your Tomb larger",
                    style::highlight(format!("{bin} tomb resize"))
                ));
            }
        }
    }

    // Finalize as normal
    tomb.finalize()
}

/// Ask user to force Tomb command.
///
/// This will only prompt if:
/// - the system has SWAP
/// - we're in interactive mode
///
/// This will not check whether the Tomb is already open, in which case forcing would not be
/// required.
pub fn ask_to_force(matcher_main: &MainMatcher) -> bool {
    // Skip if already forced
    if matcher_main.force() {
        return true;
    }

    // Skip if no swap is active, assume yes
    if !util::fs::has_swap().unwrap_or(true) {
        return false;
    }

    // Prompt
    eprintln!("To open a Tomb with active swap you must force, this may be insecure.");
    util::cli::prompt_yes("Force open?", None, matcher_main)
}
