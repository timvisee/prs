use colored::{ColoredString, Colorize};

/// Highlight the given text with a color.
pub fn highlight<S: AsRef<str>>(msg: S) -> ColoredString {
    msg.as_ref().yellow()
}

/// Highlight the given text with an error color.
pub fn highlight_error(msg: impl AsRef<str>) -> ColoredString {
    msg.as_ref().red().bold()
}

/// Highlight the given text with an warning color.
pub fn highlight_warning(msg: impl AsRef<str>) -> ColoredString {
    highlight(msg).bold()
}

/// Highlight the given text with an info color
pub fn highlight_info(msg: impl AsRef<str>) -> ColoredString {
    msg.as_ref().cyan()
}
