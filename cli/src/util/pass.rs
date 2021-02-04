use prs_lib::types::Plaintext;

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
    assert!(
        len as usize >= PASSWORD_CHAR_SETS.len(),
        "password length shorter than character set count",
    );

    // Obtain cryptografically secure random source, build char dictionary
    let mut rng = rand::thread_rng();
    let chars = PASSWORD_CHAR_SETS.join("");

    loop {
        // Sample random list of char set indices
        let indices = rand::seq::index::sample(&mut rng, chars.len(), len as usize);

        // Ensure we have at least one char index in all sets, retry otherwise
        if !PASSWORD_CHAR_SETS
            .iter()
            .scan(0, |index, char_set| {
                *index += char_set.len();
                Some(*index - char_set.len()..*index)
            })
            .all(|range| indices.iter().any(|i| range.contains(&i)))
        {
            continue;
        }

        // Build password string
        return indices
            .into_iter()
            .map(|i| chars.chars().nth(i).unwrap())
            .collect::<String>()
            .into();
    }
}
