//! Deterministic, storage-independent validation and replay for the sealed
//! draft-v1 Automerge documents-over-Nostr protocol.
//!
//! The crate is a batch reference engine. It does not perform networking,
//! persistence, signing, key custody, or application-schema validation.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod diagnostic;
mod disposition;
mod error;
mod integrity;
mod limits;
mod profile;
mod report;
mod types;
mod wire;
mod work_budget;

pub use diagnostic::DiagnosticCode;
pub use disposition::{Completion, ProtocolDisposition};
pub use error::HexError;
pub use integrity::{
    AlertError, CanonicalControlReorganizationAlert, CheckpointMismatchAlert,
    ControllerEquivocationAlert, DeviceEquivocationAlert, IntegrityAlert,
    PotentialClonedDeviceKeyAlert,
};
pub use limits::{ByteLimit, ItemLimit, LimitConversionError, ProtocolLimits};
pub use profile::ProtocolRevision;
pub use types::{
    AccountPublicKey, ActorId, ChangeHash, ChunkHash, ControllerPublicKey, CoordinateError,
    DevicePublicKey, DispositionsDigest, DocumentCoordinate, DocumentId, EventId, HistoryDigest,
    SnapshotHash,
};
pub use wire::raw_event::{RawEventBytes, RawEventError};
pub use work_budget::{BudgetExhausted, CancellationCheck, NeverCancelled, WorkBudget};

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
