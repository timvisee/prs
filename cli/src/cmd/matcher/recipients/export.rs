use clap::ArgMatches;

use super::Matcher;

/// The recipients export command matcher.
pub struct ExportMatcher<'a> {
    matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> ExportMatcher<'a> {
    /// File to output to.
    pub fn output_file(&self) -> Option<&str> {
        self.matches.value_of("output-file")
    }

    /// Check whether to copy the key.
    #[cfg(feature = "clipboard")]
    pub fn copy(&self) -> bool {
        self.matches.is_present("copy")
    }
}

impl<'a> Matcher<'a> for ExportMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("recipients")?
            .subcommand_matches("export")
            .map(|matches| ExportMatcher { matches })
    }
}
