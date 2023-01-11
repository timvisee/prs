use indicatif::{ProgressBar, ProgressStyle};

const MSG_LEN: usize = 20;

pub(crate) fn progress_bar(len: u64, quiet: bool) -> ProgressBar {
    if quiet {
        ProgressBar::hidden()
    } else {
        let pb = ProgressBar::new(len);
        pb.set_style(
            // TODO: 20 from len
            ProgressStyle::with_template("{msg:20} {wide_bar} {pos}/{len} ({eta})").unwrap(),
        );
        pb
    }
}

/// Truncate the progress prefix to the given length limit.
fn trunc_msg(msg: &str, limit: usize) -> String {
    // Return if already within limits
    if msg.len() <= limit {
        return msg.to_string();
    }

    // Truncate each part to 1 until limit is satisfied
    let mut parts = msg
        .split('/')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    let parts_len = parts.len();
    while parts_len - 1 + parts.iter().map(|n| n.len()).sum::<usize>() > limit {
        match parts.iter_mut().take(parts_len - 1).find(|p| p.len() > 1) {
            Some(part) => part.truncate(1),
            None => break,
        }
    }

    // Rebuild path string and truncate a final time
    let mut msg = parts.join("/");
    msg.truncate(limit);
    msg
}

pub(crate) trait ProgressBarExt {
    /// Set the progress bar message and truncate it.
    fn set_message_trunc(&self, msg: &str);

    /// Always print a newline, even if hidden.
    fn println_always<I: AsRef<str>>(&self, msg: I);
}

impl ProgressBarExt for ProgressBar {
    fn set_message_trunc(&self, msg: &str) {
        self.set_message(trunc_msg(msg, MSG_LEN));
    }

    fn println_always<I: AsRef<str>>(&self, msg: I) {
        if self.is_hidden() {
            println!("{}", msg.as_ref())
        } else {
            self.println(msg)
        }
    }
}
