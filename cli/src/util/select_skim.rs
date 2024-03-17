use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;

use prs_lib::{Key, Secret};
use skim::{
    prelude::{SkimItemReceiver, SkimItemSender, SkimOptionsBuilder},
    AnsiString, DisplayContext, Skim, SkimItem,
};

/// Show an interactive selection view for the given list of `items`.
/// The selected item is returned.  If no item is selected, `None` is returned instead.
fn skim_select(items: SkimItemReceiver, prompt: &str) -> Option<String> {
    let prompt = format!("{prompt}: ");
    let options = SkimOptionsBuilder::default()
        .prompt(Some(&prompt))
        .multi(false)
        // Disabled becayse of: https://github.com/lotabout/skim/issues/494
        // .height(Some("50%"))
        .build()
        .unwrap();

    // Run skim, get output, abort on close
    let output = Skim::run_with(&options, Some(items))?;
    if output.is_abort {
        return None;
    }

    // Get the first selected, and return
    output
        .selected_items
        .first()
        .map(|i| i.output().to_string())
}

/// Wrapped store secret item for skim.
pub struct SkimSecret(Secret);

impl From<Secret> for SkimSecret {
    fn from(secret: Secret) -> Self {
        Self(secret)
    }
}

impl SkimItem for SkimSecret {
    fn display(&self, _: DisplayContext) -> AnsiString {
        self.0.name.clone().into()
    }

    fn text(&self) -> Cow<str> {
        (&self.0.name).into()
    }

    fn output(&self) -> Cow<str> {
        self.0.path.to_string_lossy()
    }
}

/// Select secret.
pub fn select_secret(secrets: &[Secret]) -> Option<&Secret> {
    // Return if theres just one to choose
    if secrets.len() == 1 {
        return secrets.first();
    }

    // Let user select secret
    let items = skim_secret_items(secrets);
    let selected = skim_select(items, "Select secret")?;

    // Pick selected item from secrets list
    let path: PathBuf = selected.into();
    Some(secrets.iter().find(|e| e.path == path).unwrap())
}

/// Select key.
pub fn select_key<'a>(keys: &'a [Key], prompt: Option<&'a str>) -> Option<&'a Key> {
    // Let user select secret
    let items = skim_key_items(keys);
    let selected = skim_select(items, prompt.unwrap_or("Select key"))?;

    // Pick selected item from keys list
    Some(
        keys.iter()
            .find(|e| e.fingerprint(false) == selected)
            .unwrap(),
    )
}

/// Generate skim `SkimSecret` items from given secrets.
fn skim_secret_items(secrets: &[Secret]) -> SkimItemReceiver {
    skim_items(
        secrets
            .iter()
            .cloned()
            .map(|e| e.into())
            .collect::<Vec<SkimSecret>>(),
    )
}

/// Generate skim `SkimSecret` items from given secrets.
fn skim_key_items(keys: &[Key]) -> SkimItemReceiver {
    skim_items(
        keys.iter()
            .cloned()
            .map(|e| e.into())
            .collect::<Vec<SkimKey>>(),
    )
}

/// Create `SkimItemReceiver` from given array.
fn skim_items<I: SkimItem>(items: Vec<I>) -> SkimItemReceiver {
    let (tx_item, rx_item): (SkimItemSender, SkimItemReceiver) =
        skim::prelude::bounded(items.len());

    items.into_iter().for_each(|g| {
        let _ = tx_item.send(Arc::new(g));
    });

    rx_item
}

/// Wrapped store key item for skim.
pub struct SkimKey(Key);

impl From<Key> for SkimKey {
    fn from(key: Key) -> Self {
        Self(key)
    }
}

impl SkimItem for SkimKey {
    fn display(&self, _: DisplayContext) -> AnsiString {
        format!("{}", self.0).into()
    }

    fn text(&self) -> Cow<str> {
        format!("{}", self.0).into()
    }

    fn output(&self) -> Cow<str> {
        self.0.fingerprint(false).into()
    }
}
