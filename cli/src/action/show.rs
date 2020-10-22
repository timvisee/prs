use std::io::Write;

use clap::ArgMatches;

use crate::cmd::matcher::{main::MainMatcher, show::ShowMatcher, Matcher};
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
        let _matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let _matcher_show = ShowMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(crate::STORE_DEFAULT_ROOT);

        // TODO: do not error on none selected
        let entries = store.entries();
        let entry = crate::select_entry(&entries).expect("no entry selected");

        let plaintext = passr::crypto::decrypt_file(entry.path()).expect("failed to decrypt");
        print_plaintext(plaintext);

        Ok(())
    }
}

/// Print the given plaintext to stdout.
fn print_plaintext(plaintext: Plaintext) {
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
