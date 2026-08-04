use crate::error::HexError;
use crate::types::fixed_32::Fixed32;

const DIGITS: &[u8; 16] = b"0123456789abcdef";

pub(crate) fn decode_fixed_32(value: &str) -> Result<Fixed32, HexError> {
    decode_bytes(value).map(Fixed32::new)
}

pub(crate) fn decode_bytes<const N: usize>(value: &str) -> Result<[u8; N], HexError> {
    let input = value.as_bytes();
    if input.len() != N.checked_mul(2).ok_or(HexError::InvalidLength)? {
        return Err(HexError::InvalidLength);
    }
    let mut output = [0_u8; N];
    for (index, pair) in input.chunks_exact(2).enumerate() {
        output[index] = (nibble(pair[0])? << 4) | nibble(pair[1])?;
    }
    Ok(output)
}

pub(crate) fn encode_fixed_32(value: Fixed32) -> String {
    encode_bytes(value.as_bytes())
}

pub(crate) fn encode_bytes(value: &[u8]) -> String {
    let mut output = String::with_capacity(value.len().saturating_mul(2));
    for byte in value.iter().copied() {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}

const fn nibble(value: u8) -> Result<u8, HexError> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        _ => Err(HexError::InvalidDigit),
    }
}

#[cfg(test)]
mod tests {
    use super::{decode_fixed_32, encode_fixed_32};
    use crate::HexError;

    #[test]
    fn exact_lowercase_roundtrip() {
        let encoded = "0123456789abcdef".repeat(4);
        let decoded = decode_fixed_32(&encoded);
        assert_eq!(decoded.map(encode_fixed_32), Ok(encoded));
    }

    #[test]
    fn rejects_length_uppercase_and_non_ascii() {
        assert_eq!(decode_fixed_32("00"), Err(HexError::InvalidLength));
        assert_eq!(
            decode_fixed_32(&"A0".repeat(32)),
            Err(HexError::InvalidDigit)
        );
        assert_eq!(
            decode_fixed_32(&"é".repeat(32)),
            Err(HexError::InvalidDigit)
        );
    }
}
