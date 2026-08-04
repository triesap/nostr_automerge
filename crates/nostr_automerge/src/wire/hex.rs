use crate::error::HexError;
use crate::types::fixed_32::{FIXED_32_LENGTH, Fixed32};

const ENCODED_LENGTH: usize = FIXED_32_LENGTH * 2;
const DIGITS: &[u8; 16] = b"0123456789abcdef";

pub(crate) fn decode_fixed_32(value: &str) -> Result<Fixed32, HexError> {
    let input = value.as_bytes();
    if input.len() != ENCODED_LENGTH {
        return Err(HexError::InvalidLength);
    }
    let mut output = [0_u8; FIXED_32_LENGTH];
    for (index, pair) in input.chunks_exact(2).enumerate() {
        output[index] = (nibble(pair[0])? << 4) | nibble(pair[1])?;
    }
    Ok(Fixed32::new(output))
}

pub(crate) fn encode_fixed_32(value: Fixed32) -> String {
    let mut output = [0_u8; ENCODED_LENGTH];
    for (index, byte) in value.as_bytes().iter().copied().enumerate() {
        output[index * 2] = DIGITS[usize::from(byte >> 4)];
        output[index * 2 + 1] = DIGITS[usize::from(byte & 0x0f)];
    }
    output.into_iter().map(char::from).collect()
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
