use prs_lib::{store::FindSecret, Key, Secret, Store};

/// Find and select a secret in the given store.
///
/// If no exact secret is found, the user will be able to choose.
///
/// `None` is returned if no secret was found or selected.
pub fn store_select_secret(store: &Store, query: Option<String>) -> Option<Secret> {
    // TODO: do not use interactive selection with --no-interact mode
    #[allow(unreachable_code)]
    match store.find(query) {
        FindSecret::Exact(secret) => Some(secret),
        FindSecret::Many(secrets) => {
            // Do not show selection dialog if no secret is selected
            if secrets.is_empty() {
                return None;
            }

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
pub fn select_key(keys: &[Key]) -> Option<&Key> {
    // TODO: do not use interactive selection with --no-interact mode
    #[cfg(all(feature = "select-skim", unix))]
    {
        return super::select_skim::select_key(keys);
    }
    #[cfg(feature = "select-skim-bin")]
    {
        return super::select_skim_bin::select_key(keys);
    }
    #[cfg(feature = "select-fzf-bin")]
    {
        return super::select_fzf_bin::select_key(keys);
    }
    super::select_basic::select_key(keys)
}
