use core::str::FromStr;

use crate::HexError;
use crate::types::fixed_32::Fixed32;
use crate::wire::hex::{decode_fixed_32, encode_fixed_32};

/// The SHA-256 identifier of a verified or retained NIP-01 event.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EventId(Fixed32);

impl EventId {
    /// Constructs an event identifier from its exact digest bytes.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(Fixed32::new(bytes))
    }

    /// Returns the exact digest bytes used for canonical ordering.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        self.0.as_bytes()
    }

    /// Encodes the canonical 64-byte lowercase hexadecimal form.
    #[must_use]
    pub fn to_hex(self) -> String {
        encode_fixed_32(self.0)
    }
}

impl FromStr for EventId {
    type Err = HexError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        decode_fixed_32(value).map(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::EventId;

    #[test]
    fn order_and_text_are_canonical() {
        let low = EventId::from_bytes([0; 32]);
        let high = EventId::from_bytes([1; 32]);
        assert!(low < high);
        assert_eq!(high.to_hex(), "01".repeat(32));
    }
}
