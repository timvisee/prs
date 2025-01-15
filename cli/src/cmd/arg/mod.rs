pub mod allow_dirty;
pub mod no_sync;
pub mod property;
pub mod query;
pub mod store;
pub mod timeout;
pub mod viewer;

use clap::{parser::ValuesRef, Arg, ArgMatches};

// Re-export to arg module
pub use self::allow_dirty::ArgAllowDirty;
pub use self::no_sync::ArgNoSync;
pub use self::property::ArgProperty;
pub use self::query::ArgQuery;
pub use self::store::ArgStore;
pub use self::timeout::ArgTimeout;
pub use self::viewer::ArgViewer;

/// A generic trait, for a reusable command argument struct.
/// The `CmdArgFlag` and `CmdArgOption` traits further specify what kind of
/// argument this is.
pub trait CmdArg {
    /// Get the argument name that is used as main identifier.
    fn name() -> &'static str;

    /// Build the argument.
    fn build() -> Arg;
}

/// This `CmdArg` specification defines that this argument may be tested as
/// flag. This will allow to test whether the flag is present in the given
/// matches.
pub trait CmdArgFlag: CmdArg {
    /// Check whether the argument is present in the given matches.
    fn is_present(matches: &ArgMatches) -> bool {
        matches.get_flag(Self::name())
    }
}

/// This `CmdArg` specification defines that this argument may be tested as
/// option. This will allow to fetch the value of the argument.
pub trait CmdArgOption<'a>: CmdArg {
    /// The type of the argument value.
    type Value;

    /// Get the argument value.
    fn value(matches: &'a ArgMatches) -> Self::Value;

    /// Get the raw argument value, as a string reference.
    fn value_raw(matches: &'a ArgMatches) -> Option<&'a String> {
        matches.get_one(Self::name())
    }

    /// Get the raw argument values, as a string reference.
    fn values_raw(matches: &'a ArgMatches) -> Option<ValuesRef<'a, String>> {
        matches.get_many(Self::name())
    }
}
