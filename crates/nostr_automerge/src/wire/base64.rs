use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;

use crate::ByteLimit;

pub(crate) fn decode_padded(input: &str, maximum: ByteLimit) -> Result<Vec<u8>, Base64Error> {
    let maximum = maximum.try_usize().map_err(|_| Base64Error::TooLarge)?;
    let encoded_maximum = maximum
        .checked_add(2)
        .and_then(|value| value.checked_div(3))
        .and_then(|value| value.checked_mul(4))
        .ok_or(Base64Error::TooLarge)?;
    if input.len() > encoded_maximum || !input.len().is_multiple_of(4) || !input.is_ascii() {
        return Err(Base64Error::NonCanonical);
    }
    let decoded = STANDARD
        .decode(input)
        .map_err(|_| Base64Error::NonCanonical)?;
    if decoded.len() > maximum {
        return Err(Base64Error::TooLarge);
    }
    if STANDARD.encode(&decoded) != input {
        return Err(Base64Error::NonCanonical);
    }
    Ok(decoded)
}

pub(crate) fn encode_padded(input: &[u8]) -> String {
    STANDARD.encode(input)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Base64Error {
    TooLarge,
    NonCanonical,
}

#[cfg(test)]
mod tests {
    use super::{Base64Error, decode_padded, encode_padded};
    use crate::ProtocolRevision;

    #[test]
    fn accepts_only_standard_padded_canonical_form() {
        let limit = ProtocolRevision::draft_v1().limits().change_bytes;
        assert_eq!(decode_padded("AAE=", limit), Ok(vec![0, 1]));
        assert_eq!(encode_padded(&[0, 1]), "AAE=");
        for invalid in ["AAE", "AAE=\n", "AAE_", "AB=="] {
            assert_eq!(
                decode_padded(invalid, limit),
                Err(Base64Error::NonCanonical)
            );
        }
    }
}
