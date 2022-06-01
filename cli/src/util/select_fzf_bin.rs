use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};

use prs_lib::{Key, Secret};

/// Binary name.
#[cfg(not(windows))]
const BIN_NAME: &str = "fzf";
#[cfg(windows)]
const BIN_NAME: &str = "fzf.exe";

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

    // Spawn fzf
    let mut child = Command::new(BIN_NAME)
        .arg("--prompt")
        .arg(format!("{}: ", prompt))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("failed to spawn fzf");

    // Communicate list of items to fzf
    let data = items.join("\n");
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(data.as_bytes())
        .expect("failed to communicate list of items to fzf");

    let output = child.wait_with_output().expect("failed to select with fzf");

    // No item selected on non-zero exit code
    if !output.status.success() {
        return None;
    }

    // Get selected item, assert validity
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let stdout = stdout.strip_suffix('\n').unwrap_or(stdout);
    assert!(items.contains(&stdout));

    Some(stdout.into())
}
