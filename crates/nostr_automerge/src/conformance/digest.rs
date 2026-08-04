use sha2::{Digest, Sha256};

use crate::{
    ChangeHash, DispositionsDigest, DocumentCoordinate, EventId, HistoryDigest,
    ProtocolDisposition, ProtocolRevision,
};

const HISTORY_DOMAIN: &[u8] = b"nostr-crdt/automerge/history/v1\0";
const DISPOSITIONS_DOMAIN: &[u8] = b"nostr-crdt/automerge/dispositions/v1\0";
const MANIFEST_KIND: u32 = 31_624;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DigestError {
    Count,
    NonCanonical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub(crate) enum DispositionNamespace {
    ControlEvent = 1,
    ChangeHash = 2,
    Event = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct DispositionItem {
    pub(crate) namespace: DispositionNamespace,
    pub(crate) identifier: [u8; 32],
    pub(crate) disposition: ProtocolDisposition,
}

pub(crate) fn history_digest(
    revision: ProtocolRevision,
    coordinate: DocumentCoordinate,
    controls: &[EventId],
    accepted: &[ChangeHash],
    heads: &[ChangeHash],
) -> Result<HistoryDigest, DigestError> {
    canonical(accepted)?;
    canonical(heads)?;
    let revision = revision.identifier().as_bytes();
    let revision_len = u16::try_from(revision.len()).map_err(|_| DigestError::Count)?;
    let control_count = u32::try_from(controls.len()).map_err(|_| DigestError::Count)?;
    let accepted_count = u64::try_from(accepted.len()).map_err(|_| DigestError::Count)?;
    let head_count = u32::try_from(heads.len()).map_err(|_| DigestError::Count)?;
    let mut encoder = Sha256::new();
    encoder.update(HISTORY_DOMAIN);
    encoder.update(revision_len.to_be_bytes());
    encoder.update(revision);
    encode_coordinate(&mut encoder, coordinate);
    encoder.update(control_count.to_be_bytes());
    controls.iter().for_each(|id| encoder.update(id.as_bytes()));
    encoder.update(accepted_count.to_be_bytes());
    accepted
        .iter()
        .for_each(|hash| encoder.update(hash.as_bytes()));
    encoder.update(head_count.to_be_bytes());
    heads
        .iter()
        .for_each(|hash| encoder.update(hash.as_bytes()));
    Ok(HistoryDigest::from_bytes(encoder.finalize().into()))
}

pub(crate) fn dispositions_digest(
    revision: ProtocolRevision,
    coordinate: DocumentCoordinate,
    items: &[DispositionItem],
) -> Result<DispositionsDigest, DigestError> {
    if !items.windows(2).all(|pair| {
        (pair[0].namespace, pair[0].identifier) < (pair[1].namespace, pair[1].identifier)
    }) {
        return Err(DigestError::NonCanonical);
    }
    let revision = revision.identifier().as_bytes();
    let revision_len = u16::try_from(revision.len()).map_err(|_| DigestError::Count)?;
    let item_count = u64::try_from(items.len()).map_err(|_| DigestError::Count)?;
    let mut encoder = Sha256::new();
    encoder.update(DISPOSITIONS_DOMAIN);
    encoder.update(revision_len.to_be_bytes());
    encoder.update(revision);
    encode_coordinate(&mut encoder, coordinate);
    encoder.update(item_count.to_be_bytes());
    for item in items {
        encoder.update([item.namespace as u8]);
        encoder.update(item.identifier);
        encoder.update([item.disposition.code()]);
    }
    Ok(DispositionsDigest::from_bytes(encoder.finalize().into()))
}

fn encode_coordinate(encoder: &mut Sha256, coordinate: DocumentCoordinate) {
    encoder.update(MANIFEST_KIND.to_be_bytes());
    encoder.update(coordinate.controller().as_bytes());
    encoder.update(coordinate.document_id().as_bytes());
}

fn canonical<T: Ord>(items: &[T]) -> Result<(), DigestError> {
    items
        .windows(2)
        .all(|pair| pair[0] < pair[1])
        .then_some(())
        .ok_or(DigestError::NonCanonical)
}
