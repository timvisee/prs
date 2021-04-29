use std::io::Write;

use prs_lib::{Plaintext, Secret};

/// Print the given plaintext to stdout.
pub fn print(plaintext: Plaintext) -> Result<(), std::io::Error> {
    let mut stdout = std::io::stdout();

    stdout.write_all(plaintext.unsecure_ref())?;

    // Always finish with newline
    if let Some(&last) = plaintext.unsecure_ref().last() {
        if last != b'\n' {
            stdout.write_all(&[b'\n'])?;
        }
    }

    let _ = stdout.flush();
    Ok(())
}

/// Show full secret name if query was partial.
///
/// This notifies the user on what exact secret is selected when only part of the secret name is
/// entered. This is useful for when a partial (short) query selects the wrong secret.
pub fn print_name(query: Option<String>, secret: &Secret, quiet: bool) {
    if quiet {
        return;
    }

    if query.map(|q| !secret.name.eq(&q)).unwrap_or(true) {
        eprintln!("Secret: {}", secret.name);
    }
}
