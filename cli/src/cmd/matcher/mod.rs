pub mod copy;
pub mod delete;
pub mod duplicate;
pub mod edit;
pub mod list;
pub mod main;
pub mod r#move;
pub mod show;

// Re-export to matcher module
pub use self::copy::CopyMatcher;
pub use self::delete::DeleteMatcher;
pub use self::duplicate::DuplicateMatcher;
pub use self::edit::EditMatcher;
pub use self::list::ListMatcher;
pub use self::main::MainMatcher;
pub use self::r#move::MoveMatcher;
pub use self::show::ShowMatcher;

use clap::ArgMatches;

pub trait Matcher<'a>: Sized {
    // Construct a new matcher instance from these argument matches.
    fn with(matches: &'a ArgMatches) -> Option<Self>;
}
