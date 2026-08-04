use sha2::{Digest, Sha256};

use crate::{DispositionsDigest, DocumentCoordinate, ProtocolDisposition, ProtocolRevision};

pub(crate) use super::history_digest::history_digest;

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
    encoder.update(MANIFEST_KIND.to_be_bytes());
    encoder.update(coordinate.controller().as_bytes());
    encoder.update(coordinate.document_id().as_bytes());
    encoder.update(item_count.to_be_bytes());
    for item in items {
        encoder.update([item.namespace as u8]);
        encoder.update(item.identifier);
        encoder.update([item.disposition.code()]);
    }
    Ok(DispositionsDigest::from_bytes(encoder.finalize().into()))
}
