pub mod query;

use clap::{Arg, ArgMatches};

// Re-eexport to arg module
pub use self::query::ArgQuery;

/// A generic trait, for a reusable command argument struct.
/// The `CmdArgFlag` and `CmdArgOption` traits further specify what kind of
/// argument this is.
pub trait CmdArg {
    /// Get the argument name that is used as main identifier.
    fn name() -> &'static str;

    /// Build the argument.
    fn build<'a, 'b>() -> Arg<'a, 'b>;
}

/// This `CmdArg` specification defines that this argument may be tested as
/// flag. This will allow to test whether the flag is present in the given
/// matches.
pub trait CmdArgFlag: CmdArg {
    /// Check whether the argument is present in the given matches.
    fn is_present<'a>(matches: &ArgMatches<'a>) -> bool {
        matches.is_present(Self::name())
    }
}

/// This `CmdArg` specification defines that this argument may be tested as
/// option. This will allow to fetch the value of the argument.
pub trait CmdArgOption<'a>: CmdArg {
    /// The type of the argument value.
    type Value;

    /// Get the argument value.
    fn value<'b: 'a>(matches: &'a ArgMatches<'b>) -> Self::Value;

    /// Get the raw argument value, as a string reference.
    fn value_raw<'b: 'a>(matches: &'a ArgMatches<'b>) -> Option<&'a str> {
        matches.value_of(Self::name())
    }

    /// Get the raw argument values, as a string reference.
    fn values_raw<'b: 'a>(matches: &'a ArgMatches<'b>) -> Option<clap::Values<'a>> {
        matches.values_of(Self::name())
    }
}