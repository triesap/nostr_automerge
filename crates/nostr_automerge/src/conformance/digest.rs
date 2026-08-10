pub(crate) use super::dispositions_digest::{
    DispositionItem, DispositionNamespace, dispositions_digest,
};
pub(crate) use super::history_digest::history_digest;

use crate::{
    ChangeHash, DispositionRecord, DispositionsDigest, DocumentCoordinate, EventId, HistoryDigest,
    ProtocolRevision,
};

/// Failure to encode a canonical protocol digest input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum DigestError {
    /// An input count does not fit the canonical wire width.
    Count,
    /// An input collection is duplicated, unordered, or otherwise noncanonical.
    NonCanonical,
}

/// Encodes the normative history digest from canonical typed collections.
pub fn canonical_history_digest(
    revision: ProtocolRevision,
    coordinate: DocumentCoordinate,
    canonical_controls: &[EventId],
    accepted_changes: &[ChangeHash],
    heads: &[ChangeHash],
) -> Result<HistoryDigest, DigestError> {
    super::history_digest::history_digest(
        revision,
        coordinate,
        canonical_controls,
        accepted_changes,
        heads,
    )
}

/// Encodes the normative dispositions digest from canonical namespaced records.
pub fn canonical_dispositions_digest(
    revision: ProtocolRevision,
    coordinate: DocumentCoordinate,
    records: &[DispositionRecord],
) -> Result<DispositionsDigest, DigestError> {
    let items = super::dispositions_digest::disposition_items(records)?;
    super::dispositions_digest::dispositions_digest(revision, coordinate, &items)
}
