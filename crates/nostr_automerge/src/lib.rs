//! Deterministic, storage-independent validation and replay for the sealed
//! draft-v1 Automerge documents-over-Nostr protocol.
//!
//! The crate is a batch reference engine. It does not perform networking,
//! persistence, signing, key custody, or application-schema validation.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

#[allow(dead_code)]
mod automerge_adapter;
#[allow(dead_code)]
mod carrier;
#[allow(dead_code)]
mod conformance;
#[allow(dead_code)]
mod control;
mod crypto;
mod diagnostic;
mod disposition;
mod error;
#[allow(dead_code)]
mod evidence;
#[allow(dead_code)]
mod graph;
mod integrity;
mod limits;
mod profile;
#[allow(dead_code)]
mod reference;
#[allow(dead_code)]
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
    Nip01Signature, SnapshotHash,
};
pub use wire::error::WireDiagnostic;
pub use wire::nip01::verified::{Nip01VerificationError, VerifiedNip01Event};
pub use wire::raw_event::{RawEventBytes, RawEventError};
pub use work_budget::{BudgetExhausted, CancellationCheck, NeverCancelled, WorkBudget};

/// The package version of this implementation.
pub const IMPLEMENTATION_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Exercises only the pre-parser Automerge framing gate for fuzzing.
#[doc(hidden)]
pub fn qualification_probe_automerge_framing(input: &[u8]) {
    let _ = automerge_adapter::framing::validate_change_frame(input, ProtocolRevision::draft_v1());
}

/// Exercises framed Automerge semantic decoding for fuzzing.
#[doc(hidden)]
pub fn qualification_probe_automerge_decode(input: &[u8]) {
    let _ = automerge_adapter::decode::decode_change(input, ProtocolRevision::draft_v1());
}

/// Exercises canonical non-compressing Automerge re-encoding for fuzzing.
#[doc(hidden)]
pub fn qualification_probe_automerge_reencode(input: &[u8]) {
    let _ = automerge_adapter::encode::qualify_canonical_reencoding(
        input,
        ProtocolRevision::draft_v1(),
    );
}

#[cfg(test)]
mod tests {
    use super::IMPLEMENTATION_VERSION;

    #[test]
    fn package_uses_approved_alpha_version() {
        assert_eq!(IMPLEMENTATION_VERSION, "0.1.0-alpha.0");
    }
}
