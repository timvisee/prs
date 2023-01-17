use base64::{DecodeError, Engine};

/// Encode a string as base64 with standard encoding.
pub(crate) fn encode<T: AsRef<[u8]>>(input: T) -> String {
    base64::engine::general_purpose::STANDARD.encode(input)
}

/// Decode a string from base64 with standard decoding.
pub(crate) fn decode<T: AsRef<[u8]>>(input: T) -> Result<Vec<u8>, DecodeError> {
    base64::engine::general_purpose::STANDARD.decode(input)
}
