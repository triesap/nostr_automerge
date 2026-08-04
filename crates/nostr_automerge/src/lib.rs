//! Deterministic, storage-independent validation and replay for the sealed
//! draft-v1 Automerge documents-over-Nostr protocol.
//!
//! The crate is a batch reference engine. It does not perform networking,
//! persistence, signing, key custody, or application-schema validation.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod error;
mod types;
mod wire;

pub use error::HexError;
pub use types::{AccountPublicKey, ControllerPublicKey, DevicePublicKey, EventId};

/// The package version of this implementation.
pub const IMPLEMENTATION_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::IMPLEMENTATION_VERSION;

    #[test]
    fn package_uses_approved_alpha_version() {
        assert_eq!(IMPLEMENTATION_VERSION, "0.1.0-alpha.0");
    }
}
