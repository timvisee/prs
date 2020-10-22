use clap::ArgMatches;
use copypasta_ext::prelude::*;
use copypasta_ext::x11_fork::ClipboardContext;

use crate::cmd::matcher::{copy::CopyMatcher, Matcher};
use crate::Store;
use prs::types::Plaintext;

/// A file copy action.
pub struct Copy<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Copy<'a> {
    /// Construct a new copy action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the copy action.
    // TODO: re-implement error handling
    pub fn invoke(&self) -> Result<(), ()> {
        // Create the command matchers
        let matcher_copy = CopyMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(crate::STORE_DEFAULT_ROOT);

        // TODO: do not error on none selected
        let secrets = store.secrets(matcher_copy.query());
        let secret = crate::select_secret(&secrets).expect("no secret selected");

        let mut plaintext = prs::crypto::decrypt_file(&secret.path).expect("failed to decrypt");

        // Trim plaintext to first line
        if !matcher_copy.all() {
            plaintext = plaintext
                .first_line()
                .expect("failed to get first line of secret");
        }

        copy(plaintext);

        Ok(())
    }
}

/// Copy the given plain text to the user clipboard.
// TODO: clear clipboard after timeout
fn copy(plaintext: Plaintext) {
    let mut ctx = ClipboardContext::new().unwrap();
    ctx.set_contents(plaintext.to_str().unwrap().into())
        .unwrap();

    eprintln!("Secret copied to clipboard...");
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
