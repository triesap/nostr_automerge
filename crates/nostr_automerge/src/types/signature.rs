use core::fmt;
use core::str::FromStr;

use crate::HexError;
use crate::wire::hex::{decode_bytes, encode_bytes};

/// An exact 64-byte NIP-01 BIP-340 signature.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Nip01Signature([u8; 64]);

impl Nip01Signature {
    /// Constructs a signature from exact bytes without cryptographic verification.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 64]) -> Self {
        Self(bytes)
    }

    /// Returns the exact signature bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }

    /// Returns canonical 128-byte lowercase hexadecimal.
    #[must_use]
    pub fn to_hex(self) -> String {
        encode_bytes(&self.0)
    }
}

impl FromStr for Nip01Signature {
    type Err = HexError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        decode_bytes(value).map(Self)
    }
}

impl fmt::Debug for Nip01Signature {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Nip01Signature([REDACTED])")
    }
}

#[cfg(test)]
mod tests {
    use super::Nip01Signature;
    use crate::HexError;
    use core::str::FromStr;

    #[test]
    fn signature_codec_is_exact_and_redacted() {
        let text = "ab".repeat(64);
        assert_eq!(
            Nip01Signature::from_str(&text).map(Nip01Signature::to_hex),
            Ok(text)
        );
        assert_eq!(
            Nip01Signature::from_str(&"AB".repeat(64)),
            Err(HexError::InvalidDigit)
        );
        assert_eq!(
            format!("{:?}", Nip01Signature::from_bytes([1; 64])),
            "Nip01Signature([REDACTED])"
        );
    }
}
