use std::io::Write;

use clap::ArgMatches;

use crate::cmd::matcher::{show::ShowMatcher, Matcher};
use crate::Store;
use passr::types::Plaintext;

/// A file show action.
pub struct Show<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Show<'a> {
    /// Construct a new show action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the show action.
    // TODO: re-implement error handling
    pub fn invoke(&self) -> Result<(), ()> {
        // Create the command matchers
        // TODO: do we need these?
        let matcher_show = ShowMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(crate::STORE_DEFAULT_ROOT);

        // TODO: do not error on none selected
        let secrets = store.secrets();
        let secret = crate::select_secret(&secrets).expect("no secret selected");

        let mut plaintext = passr::crypto::decrypt_file(&secret.path).expect("failed to decrypt");

        // Trim plaintext to first line
        if matcher_show.first_line() {
            plaintext = plaintext
                .first_line()
                .expect("failed to get first line of secret");
        }

        print(plaintext);

        Ok(())
    }
}

/// Print the given plaintext to stdout.
fn print(plaintext: Plaintext) {
    eprintln!("=v=v=v=v=v=v=v=v=v=");
    std::io::stdout().write_all(&plaintext.0).unwrap();
    let _ = std::io::stdout().flush();
    eprintln!("\n=^=^=^=^=^=^=^=^=^=");
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
