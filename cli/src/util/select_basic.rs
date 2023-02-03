use std::collections::HashMap;

use crate::util::error;
use prs_lib::{Key, Secret};

/// Select secret.
pub fn select_secret(secrets: &[Secret]) -> Option<&Secret> {
    // Return if theres just one to choose
    if secrets.len() == 1 {
        return secrets.get(0);
    }

    let map: HashMap<_, _> = secrets
        .iter()
        .map(|secret| (secret.name.clone(), secret))
        .collect();
    let items: Vec<_> = map.keys().collect();
    select_item("Select key", &items)
        .as_ref()
        .map(|item| map[item])
}

/// Select key.
pub fn select_key<'a>(keys: &'a [Key], prompt: Option<&'a str>) -> Option<&'a Key> {
    let map: HashMap<_, _> = keys.iter().map(|key| (key.to_string(), key)).collect();
    let items: Vec<_> = map.keys().collect();
    select_item(prompt.unwrap_or("Select key"), &items)
        .as_ref()
        .map(|item| map[item])
}

/// Interactively select one of the given items.
fn select_item<'a, S: AsRef<str>>(prompt: &'a str, items: &'a [S]) -> Option<String> {
    // Build sorted list of string references as items
    let mut items = items.iter().map(|i| i.as_ref()).collect::<Vec<_>>();
    items.sort_unstable();

    loop {
        // Print options and prompt
        items
            .iter()
            .enumerate()
            .for_each(|(i, item)| eprintln!("{}: {}", i + 1, item));
        eprint!("{prompt} (number/empty): ");

        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("failed to read user input from stdin");

        // If empty, we selected none
        if input.trim().is_empty() {
            return None;
        }

        // Try to parse number, select item, or show error and retry
        match input.trim().parse::<usize>().ok() {
            Some(n) if n > 0 && n <= items.len() => return Some(items[n - 1].into()),
            _ => {
                error::print_error_msg("invalid selection input");
                eprintln!();
            }
        }
    }
}
