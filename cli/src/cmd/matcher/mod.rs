pub mod main;
pub mod show;

// Re-export to matcher module
pub use self::main::MainMatcher;
pub use self::show::ShowMatcher;

use clap::ArgMatches;

pub trait Matcher<'a>: Sized {
    // Construct a new matcher instance from these argument matches.
    fn with(matches: &'a ArgMatches) -> Option<Self>;
}
