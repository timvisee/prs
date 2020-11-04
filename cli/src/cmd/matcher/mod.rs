pub mod add;
pub mod clone;
#[cfg(feature = "clipboard")]
pub mod copy;
pub mod duplicate;
pub mod edit;
pub mod generate;
pub mod git;
pub mod housekeeping;
pub mod init;
pub mod list;
pub mod main;
pub mod r#move;
pub mod recipients;
pub mod remove;
pub mod show;
pub mod sync;

// Re-export to matcher module
pub use self::add::AddMatcher;
pub use self::clone::CloneMatcher;
#[cfg(feature = "clipboard")]
pub use self::copy::CopyMatcher;
pub use self::duplicate::DuplicateMatcher;
pub use self::edit::EditMatcher;
pub use self::generate::GenerateMatcher;
pub use self::git::GitMatcher;
pub use self::housekeeping::HousekeepingMatcher;
pub use self::init::InitMatcher;
pub use self::list::ListMatcher;
pub use self::main::MainMatcher;
pub use self::r#move::MoveMatcher;
pub use self::recipients::RecipientsMatcher;
pub use self::remove::RemoveMatcher;
pub use self::show::ShowMatcher;
pub use self::sync::SyncMatcher;

use clap::ArgMatches;

pub trait Matcher<'a>: Sized {
    // Construct a new matcher instance from these argument matches.
    fn with(matches: &'a ArgMatches) -> Option<Self>;
}
