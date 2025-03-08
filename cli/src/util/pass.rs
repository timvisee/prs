use rand::Rng;

use prs_lib::Plaintext;

use crate::util::error;

/// Character sets to use for password generation.
///
/// When generating a password, characters are sampled from all these lists. Password generation is
/// retried if it doesn't contain at least one character from all the lists.
const PASSWORD_CHAR_SETS: [&str; 4] = [
    "abcdefghijklmnopqrstuvwxyz",
    "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
    "0123456789",
    "!@#$%&*+-=/[]<>(),.;|",
];

/// Generate secure random password.
///
/// This generates a cryptografically secure random password string.
/// Password entropy (defining its strength) is dependent on the given `len`. Don't use short
/// lengths.
///
/// The characters used in the password being generated is defined in `PASSWORD_CHAR_SETS`. A
/// password always includes at least one character from each set.
///
/// The returned password is embedded in `Plaintext` for security reasons.
///
/// # Panics
///
/// Panics if `len` is shorter than the number of sets in `PASSWORD_CHAR_SETS`.
pub fn generate_password(len: u16) -> Plaintext {
    // Show warning if length if too short to cover all sets
    let too_short = (len as usize) < PASSWORD_CHAR_SETS.len();
    if too_short {
        error::print_warning(format!(
            "password length too short to use all character sets (should be at least {})",
            PASSWORD_CHAR_SETS.len()
        ));
    }

    // Obtain secure random source, build char dictionary
    let mut rng = rand::rng();
    let chars = PASSWORD_CHAR_SETS.join("");

    // Build password until we have an accepted one
    let mut pass = String::with_capacity(len as usize);
    loop {
        for _ in 0..len {
            let c = rng.random_range(0..chars.len());
            let c = chars.chars().nth(c).unwrap();
            pass.push(c);
        }

        // Ensure password covers all sets
        if too_short
            || PASSWORD_CHAR_SETS
                .iter()
                .all(|set| set.chars().any(|c| pass.contains(c)))
        {
            return pass.into();
        }

        pass.truncate(0);
    }
}
