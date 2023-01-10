#[cfg(feature = "clipboard")]
pub mod clip;
#[cfg(feature = "clipboard")]
pub mod clip_revert;
pub mod completions;
#[cfg(all(feature = "clipboard", feature = "totp"))]
pub mod totp_recopy;

use clap::ArgMatches;

use super::Matcher;

/// The internal matcher.
pub struct InternalMatcher<'a> {
    root: &'a ArgMatches,
    _matches: &'a ArgMatches,
}

impl<'a: 'b, 'b> InternalMatcher<'a> {
    /// Get the internal clipboard sub command, if matched.
    #[cfg(feature = "clipboard")]
    pub fn clip(&'a self) -> Option<clip::ClipMatcher> {
        clip::ClipMatcher::with(self.root)
    }

    /// Get the internal clipboard revert sub command, if matched.
    #[cfg(feature = "clipboard")]
    pub fn clip_revert(&'a self) -> Option<clip_revert::ClipRevertMatcher> {
        clip_revert::ClipRevertMatcher::with(self.root)
    }

    /// Get the internal completions generator sub command, if matched.
    pub fn completions(&'a self) -> Option<completions::CompletionsMatcher> {
        completions::CompletionsMatcher::with(self.root)
    }

    /// Get the internal clipboard revert sub command, if matched.
    #[cfg(all(feature = "clipboard", feature = "totp"))]
    pub fn totp_recopy(&'a self) -> Option<totp_recopy::TotpRecopyMatcher> {
        totp_recopy::TotpRecopyMatcher::with(self.root)
    }
}

impl<'a> Matcher<'a> for InternalMatcher<'a> {
    fn with(root: &'a ArgMatches) -> Option<Self> {
        root.subcommand_matches("internal")
            .map(|matches| InternalMatcher {
                root,
                _matches: matches,
            })
    }
}
