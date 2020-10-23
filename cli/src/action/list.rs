use clap::ArgMatches;

use crate::cmd::matcher::{list::ListMatcher, Matcher};
use crate::Store;

/// A file list action.
pub struct List<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> List<'a> {
    /// Construct a new list action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the list action.
    // TODO: re-implement error handling
    pub fn invoke(&self) -> Result<(), ()> {
        // Create the command matchers
        // TODO: do we need these?
        let matcher_list = ListMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(crate::STORE_DEFAULT_ROOT);

        let mut secrets = store.secrets(matcher_list.query());
        secrets.sort_unstable_by(|a, b| a.name.cmp(&b.name));

        secrets.iter().for_each(|s| println!("{}", s.name));

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

//     /// An error occurred while fetching the file listrmation.
//     #[fail(display = "failed to fetch file list")]
//     List(#[cause] ListError),

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

// impl From<ListError> for Error {
//     fn from(err: ListError) -> Error {
//         Error::List(err)
//     }
// }
