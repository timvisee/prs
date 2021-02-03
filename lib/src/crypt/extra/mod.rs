pub mod backend;
pub mod proto;
pub mod recipients;

/// Crypto protocol.
#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Proto {
    /// GPG crypto.
    #[cfg(feature = "_crypto-gpg")]
    Gpg,
}

/// Represents a key.
#[derive(Clone, PartialEq)]
#[non_exhaustive]
pub enum Key {
    /// An GPG key.
    #[cfg(feature = "_crypto-gpg")]
    Gpg(proto::gpg::Key),
}

impl Key {
    /// Get key protocol type.
    pub fn proto(&self) -> Proto {
        match self {
            #[cfg(feature = "_crypto-gpg")]
            Key::Gpg(_) => Proto::Gpg,
        }
    }

    /// Key fingerprint.
    pub fn fingerprint(&self, short: bool) -> String {
        match self {
            #[cfg(feature = "_crypto-gpg")]
            Key::Gpg(key) => key.fingerprint(short),
        }
    }

    /// Key displayable user data.
    pub fn display_user(&self) -> String {
        match self {
            #[cfg(feature = "_crypto-gpg")]
            Key::Gpg(key) => key.display_user(),
        }
    }
}
