use prs_lib::{Key, Secret, Store, store::FindSecret};

use crate::{
    cmd::matcher::MainMatcher,
    util::error::{ErrorHintsBuilder, quit_error_msg},
};

/// Find and select a secret in the given store.
///
/// If no exact secret is found, the user will be able to choose.
///
/// `None` is returned if no secret was found or selected.
pub fn store_select_secret(
    store: &Store,
    query: Option<String>,
    matcher_main: &MainMatcher,
) -> Option<Secret> {
    let has_query = query.as_ref().is_some_and(|q| !q.trim().is_empty());

    #[allow(unreachable_code)]
    match store.find(query) {
        FindSecret::Exact(secret) => Some(secret),
        FindSecret::Many(mut secrets) => {
            // Do not show selection dialog if no secret is selected
            if secrets.is_empty() {
                return None;
            }

            // Return if theres just one to choose
            if secrets.len() == 1 {
                return secrets.pop();
            }

            // Cannot choose out of many without interaction, error instead
            if matcher_main.no_interact() {
                quit_error_msg(
                    if has_query {
                        format!("query matched {} secrets", secrets.len())
                    } else {
                        format!("no query specified, store has {} secrets", secrets.len())
                    },
                    ErrorHintsBuilder::from_matcher(matcher_main)
                        .add_info(if has_query {
                            "change query to match exactly one secret"
                        } else {
                            "add query to match exactly one secret"
                        })
                        .add_info(format!(
                            "or remove '{}' ('{}') to use interactive selection",
                            crate::util::style::highlight("--no-interact"),
                            crate::util::style::highlight("-I"),
                        ))
                        .verbose(false)
                        .help(true)
                        .build()
                        .unwrap(),
                );
            }

            // When updating features, also update warning in build.rs
            #[cfg(all(feature = "select-skim", unix))]
            {
                return super::select_skim::select_secret(&secrets).cloned();
            }
            #[cfg(feature = "select-skim-bin")]
            {
                return super::select_skim_bin::select_secret(&secrets).cloned();
            }
            #[cfg(feature = "select-fzf-bin")]
            {
                return super::select_fzf_bin::select_secret(&secrets).cloned();
            }
            super::select_basic::select_secret(&secrets).cloned()
        }
    }
}

/// Select key.
#[allow(unreachable_code)]
pub fn select_key<'a>(keys: &'a [Key], prompt: Option<&'a str>) -> Option<&'a Key> {
    // TODO: do not use interactive selection with --no-interact mode

    // When updating features, also update warning in build.rs
    #[cfg(all(feature = "select-skim", unix))]
    {
        return super::select_skim::select_key(keys, prompt);
    }
    #[cfg(feature = "select-skim-bin")]
    {
        return super::select_skim_bin::select_key(keys, prompt);
    }
    #[cfg(feature = "select-fzf-bin")]
    {
        return super::select_fzf_bin::select_key(keys, prompt);
    }
    super::select_basic::select_key(keys, prompt)
}
