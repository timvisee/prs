use std::fs;

use clap::ArgMatches;

use crate::cmd::matcher::{duplicate::DuplicateMatcher, Matcher};
use crate::Store;

/// A file duplicate action.
pub struct Duplicate<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Duplicate<'a> {
    /// Construct a new duplicate action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the duplicate action.
    // TODO: re-implement error handling
    pub fn invoke(&self) -> Result<(), ()> {
        // Create the command matchers
        let matcher_duplicate = DuplicateMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(crate::STORE_DEFAULT_ROOT);

        // TODO: do not error on none selected
        let secrets = store.secrets(matcher_duplicate.query());
        let secret = crate::select_secret(&secrets).expect("no secret selected");

        let target = matcher_duplicate.target();

        // TODO: move this into normalize function below
        // TODO: do not unwrap here
        let target = shellexpand::full(target).expect("failed to expand target path");

        // Normalize target path
        let path = store.normalize_secret_path(
            target.as_ref(),
            secret.path.file_name().and_then(|p| p.to_str()),
            true,
        );

        // Check if target already exists
        if path.is_file() {
            // TODO: show prompt to override?
        }

        // Copy secret, show error
        if let Err(err) = fs::copy(&secret.path, path) {
            // TODO: show proper error here
            eprintln!("Failed to duplicate secret: {:?}", err);
        }

        Ok(())
    }
}

// #[derive(Debug, Fail)]
// pub enum Error {
//     /// Failed to parse a share URL, it was invalid.
//     /// This error is not related to a specific action.
//     #[fail(display = "invalid share link")]
//     InvalidUrl(#[cause] FileParseError),

//     /// An error occurred while checking if the file exists.
//     #[fail(display = "failed to check whether the file exists")]
//     Exists(#[cause] ExistsError),

//     /// An error occurred while fetching the file showrmation.
//     #[fail(display = "failed to fetch file show")]
//     Show(#[cause] ShowError),

//     /// The given Send file has expired, or did never exist in the first place.
//     #[fail(display = "the file has expired or did never exist")]
//     Expired,
// }

// impl From<FileParseError> for Error {
//     fn from(err: FileParseError) -> Error {
//         Error::InvalidUrl(err)
//     }
// }

// impl From<ExistsError> for Error {
//     fn from(err: ExistsError) -> Error {
//         Error::Exists(err)
//     }
// }

// impl From<ShowError> for Error {
//     fn from(err: ShowError) -> Error {
//         Error::Show(err)
//     }
// }
