use core::str::FromStr;

use crate::HexError;
use crate::types::fixed_32::Fixed32;
use crate::wire::hex::{decode_fixed_32, encode_fixed_32};

macro_rules! public_key {
    ($name:ident, $docs:literal) => {
        #[doc = $docs]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(Fixed32);

        impl $name {
            /// Constructs a semantically typed key from its 32-byte x-coordinate.
            #[must_use]
            pub const fn from_bytes(bytes: [u8; 32]) -> Self {
                Self(Fixed32::new(bytes))
            }

            /// Returns the exact bytes used by signatures and canonical ordering.
            #[must_use]
            pub const fn as_bytes(&self) -> &[u8; 32] {
                self.0.as_bytes()
            }

            /// Encodes the canonical lowercase hexadecimal boundary form.
            #[must_use]
            pub fn to_hex(self) -> String {
                encode_fixed_32(self.0)
            }
        }

        impl FromStr for $name {
            type Err = HexError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                decode_fixed_32(value).map(Self)
            }
        }
    };
}

public_key!(
    ControllerPublicKey,
    "The controller key fixed by a document coordinate."
);
public_key!(
    DevicePublicKey,
    "A signing key assigned to one editing installation."
);
public_key!(
    AccountPublicKey,
    "An optional immutable account mapping in a device grant."
);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) struct VerifiedPublicKey(Fixed32);

impl From<ControllerPublicKey> for VerifiedPublicKey {
    fn from(value: ControllerPublicKey) -> Self {
        Self(value.0)
    }
}

impl From<DevicePublicKey> for VerifiedPublicKey {
    fn from(value: DevicePublicKey) -> Self {
        Self(value.0)
    }
}

impl VerifiedPublicKey {
    pub(crate) fn parse(value: &str) -> Result<Self, HexError> {
        decode_fixed_32(value).map(Self)
    }

    #[allow(dead_code)]
    pub(crate) const fn as_bytes(&self) -> &[u8; 32] {
        self.0.as_bytes()
    }

    pub(crate) fn to_hex(self) -> String {
        encode_fixed_32(self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::{AccountPublicKey, ControllerPublicKey, DevicePublicKey, VerifiedPublicKey};

    #[test]
    fn semantic_keys_share_only_private_verified_conversion() {
        let bytes = [7; 32];
        assert_eq!(ControllerPublicKey::from_bytes(bytes).as_bytes(), &bytes);
        assert_eq!(DevicePublicKey::from_bytes(bytes).to_hex(), "07".repeat(32));
        assert_eq!(AccountPublicKey::from_bytes(bytes).as_bytes(), &bytes);
        let verified = VerifiedPublicKey::from(DevicePublicKey::from_bytes(bytes));
        assert_eq!(verified.as_bytes(), &bytes);
    }
}
