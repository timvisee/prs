use prs_lib::{store::FindSecret, Key, Secret, Store};

/// Find and select a secret in the given store.
///
/// If no exact secret is found, the user will be able to choose.
///
/// `None` is returned if no secret was found or selected.
pub fn store_select_secret(store: &Store, query: Option<String>) -> Option<Secret> {
    // TODO: do not use interactive selection with --no-interact mode
    match store.find(query) {
        FindSecret::Exact(secret) => Some(secret),
        FindSecret::Many(secrets) => {
            #[cfg(unix)]
            {
                super::select_skim::select_secret(&secrets).cloned()
            }
            #[cfg(not(unix))]
            {
                super::select_fzf::select_secret(&secrets).cloned()
            }
        }
    }
}

/// Select key.
pub fn select_key(keys: &[Key]) -> Option<&Key> {
    // TODO: do not use interactive selection with --no-interact mode
    #[cfg(unix)]
    {
        super::select_skim::select_key(keys)
    }
    #[cfg(not(unix))]
    {
        super::select_fzf::select_key(keys)
    }
}
