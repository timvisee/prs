use super::Key;

/// Format fingerprint in consistent format.
///
/// Trims and uppercases.
pub fn format_fingerprint<S: AsRef<str>>(fingerprint: S) -> String {
    fingerprint.as_ref().trim().to_uppercase()
}

/// Check whether two fingerprints match.
pub fn fingerprints_equal<S: AsRef<str>, T: AsRef<str>>(a: S, b: T) -> bool {
    !a.as_ref().trim().is_empty()
        && a.as_ref().trim().to_uppercase() == b.as_ref().trim().to_uppercase()
}

/// Check whether a list of keys contains the given fingerprint.
pub fn keys_contain_fingerprint<S: AsRef<str>>(keys: &[Key], fingerprint: S) -> bool {
    keys.iter()
        .any(|key| fingerprints_equal(key.fingerprint(false), fingerprint.as_ref()))
}

///// Check whether the given recipients contain any key that we have a secret key in our keychain
///// for.
//pub fn contains_own_secret_key(recipients: &Recipients) -> Result<bool> {
//    let secrets = all(true)?;
//    Ok(recipients
//        .keys()
//        .iter()
//        .any(|k| secrets.has_fingerprint(&k.fingerprint(false))))
//}

///// Filter list of fingerprints.
/////
///// Keep list of unimported fingerprints.
//pub fn filter_imported_fingerprints(fingerprints: Vec<String>) -> Result<Vec<String>> {
//    let mut context = crypto_old::context()?;
//    Ok(fingerprints
//        .into_iter()
//        .filter(|fp| context.get_key(fp).is_err())
//        .collect())
//}
