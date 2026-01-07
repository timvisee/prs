//! Crypto GPG protocol.

/// GnuPG Long key ID length
const LONG_KEY_ID: usize = 16;

/// Represents a GPG key.
#[derive(Clone)]
pub struct Key {
    /// Full fingerprint.
    pub fingerprint: String,

    /// Displayable user ID strings.
    pub user_ids: Vec<String>,
}

impl Key {
    /// Key fingerprint.
    pub fn fingerprint(&self, short: bool) -> String {
        let fp = if short {
            &self.fingerprint[self.fingerprint.len() - LONG_KEY_ID..]
        } else {
            &self.fingerprint
        };

        crate::crypto::util::normalize_fingerprint(fp)
    }

    /// Key displayable user data.
    pub fn display_user(&self) -> String {
        self.user_ids.join("; ")
    }

    /// Transform into generic key.
    pub fn into_key(self) -> crate::crypto::Key {
        crate::crypto::Key::Gpg(self)
    }
}

impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.fingerprint.trim().to_uppercase() == other.fingerprint.trim().to_uppercase()
    }
}
