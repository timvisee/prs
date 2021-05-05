use std::io::Write;

use prs_lib::{Plaintext, Secret, Store};

/// Secret alias recursion limit.
const SECRET_ALIAS_DEPTH: u32 = 30;

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
pub fn print_name(query: Option<String>, secret: &Secret, store: &Store, quiet: bool) {
    // If quiet or query matches exact name, do not print it
    if quiet || query.map(|q| secret.name.eq(&q)).unwrap_or(false) {
        return;
    }

    // Show secret with alias target if available
    if let Some(alias) = resolve_alias(secret, store) {
        eprintln!("Secret: {} -> {}", secret.name, alias.name);
    } else {
        eprintln!("Secret: {}", secret.name);
    }
}

/// Resolve secret that is aliased.
///
/// This find the target alias if the given secret is an alias. This uses recursive searching.
/// If the secret is not an alias, `None` is returned.
fn resolve_alias(secret: &Secret, store: &Store) -> Option<Secret> {
    fn f(secret: &Secret, store: &Store, depth: u32) -> Option<Secret> {
        assert!(
            depth < SECRET_ALIAS_DEPTH,
            "failed to resolve secret alias target, recursion limit reached"
        );
        match secret.alias_target(store) {
            Ok(s) => f(&s, store, depth + 1),
            Err(_) if depth > 0 => Some(secret.clone()),
            Err(_) => None,
        }
    }
    f(secret, store, 0)
}
