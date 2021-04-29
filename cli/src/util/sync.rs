use prs_lib::sync::{Readyness, Sync};

use crate::util::error::{quit_error, quit_error_msg, ErrorHintsBuilder};

/// Ensure the store is ready, otherwise quit.
pub fn ensure_ready(sync: &Sync, allow_dirty: bool) {
    let readyness = match sync.readyness() {
        Ok(readyness) => readyness,
        Err(err) => {
            quit_error(
                err.context("failed to query store sync readyness state"),
                ErrorHintsBuilder::default().git(true).build().unwrap(),
            );
        }
    };

    let mut error = ErrorHintsBuilder::default();
    error.git(true);
    if let Readyness::Dirty = readyness {
        error.allow_dirty(true);
    }

    quit_error_msg(
        match readyness {
            Readyness::Ready | Readyness::NoSync => return,
            Readyness::Dirty if allow_dirty => return,
            Readyness::Dirty => "store git repository is dirty and has uncommitted changes".into(),
            Readyness::RepoState(state) => {
                format!("store git repository is in unfinished state: {:?}", state)
            }
        },
        error.build().unwrap(),
    );
}
