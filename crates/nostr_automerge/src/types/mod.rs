macro_rules! semantic_id {
    ($name:ident, $docs:literal) => {
        #[doc = $docs]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(crate::types::fixed_32::Fixed32);

        impl $name {
            /// Constructs the semantic identifier from its exact 32 bytes.
            #[must_use]
            pub const fn from_bytes(bytes: [u8; 32]) -> Self {
                Self(crate::types::fixed_32::Fixed32::new(bytes))
            }

            /// Returns the exact identifier bytes used for canonical ordering.
            #[must_use]
            pub const fn as_bytes(&self) -> &[u8; 32] {
                self.0.as_bytes()
            }

            /// Encodes the canonical lowercase hexadecimal boundary form.
            #[must_use]
            pub fn to_hex(self) -> String {
                crate::wire::hex::encode_fixed_32(self.0)
            }
        }

        impl core::str::FromStr for $name {
            type Err = crate::HexError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                crate::wire::hex::decode_fixed_32(value).map(Self)
            }
        }
    };
}

mod actor_id;
mod change_hash;
mod digest;
mod document_coordinate;
mod document_id;
mod event_id;
pub(crate) mod fixed_32;
pub(crate) mod public_key;
mod signature;

pub use actor_id::ActorId;
pub use change_hash::{ChangeHash, ChunkHash, SnapshotHash};
pub use digest::{DispositionsDigest, HistoryDigest};
pub use document_coordinate::{CoordinateError, DocumentCoordinate};
pub use document_id::DocumentId;
pub use event_id::EventId;
pub use public_key::{AccountPublicKey, ControllerPublicKey, DevicePublicKey};
pub use signature::Nip01Signature;
