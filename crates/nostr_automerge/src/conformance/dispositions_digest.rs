use sha2::{Digest, Sha256};

use super::digest::DigestError;
use crate::{
    DispositionRecord, DispositionsDigest, DocumentCoordinate, ProtocolDisposition,
    ProtocolItemIdentifier, ProtocolRevision,
};

const DOMAIN: &[u8] = b"nostr-crdt/automerge/dispositions/v1\0";
const MANIFEST_KIND: u32 = 31_624;

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

pub(crate) fn disposition_items(
    records: &[DispositionRecord],
) -> Result<Vec<DispositionItem>, DigestError> {
    if !records
        .windows(2)
        .all(|pair| pair[0].identifier() < pair[1].identifier())
    {
        return Err(DigestError::NonCanonical);
    }
    Ok(records
        .iter()
        .map(|record| {
            let namespace = match record.identifier() {
                ProtocolItemIdentifier::ControlEvent(_) => DispositionNamespace::ControlEvent,
                ProtocolItemIdentifier::ChangeHash(_) => DispositionNamespace::ChangeHash,
                ProtocolItemIdentifier::Event(_) => DispositionNamespace::Event,
            };
            DispositionItem {
                namespace,
                identifier: *record.identifier().as_bytes(),
                disposition: record.disposition(),
            }
        })
        .collect())
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
    encoder.update(DOMAIN);
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

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use super::{DispositionItem, DispositionNamespace, disposition_items, dispositions_digest};
    use crate::conformance::digest::DigestError;
    use crate::{Completion, DocumentCoordinate, ProtocolDisposition, ProtocolRevision};

    #[test]
    fn implement_dispositions_digest_encoder() {
        let coordinate =
            DocumentCoordinate::from_str(&format!("31624:{}:{}", "11".repeat(32), "22".repeat(32)));
        assert!(coordinate.is_ok());
        let Ok(coordinate) = coordinate else { return };
        let items = [
            DispositionItem {
                namespace: DispositionNamespace::ControlEvent,
                identifier: [0xaa; 32],
                disposition: ProtocolDisposition::Accepted,
            },
            DispositionItem {
                namespace: DispositionNamespace::ChangeHash,
                identifier: [0xbb; 32],
                disposition: ProtocolDisposition::Excluded,
            },
            DispositionItem {
                namespace: DispositionNamespace::Event,
                identifier: [0xcc; 32],
                disposition: ProtocolDisposition::Invalid,
            },
        ];
        let digest = dispositions_digest(ProtocolRevision::draft_v1(), coordinate, &items);
        assert_eq!(
            digest.map(|value| value.to_hex()),
            Ok("ae39260c28bb68255ccd83b5f602187e48dc78c4a92df5264d17b5e8c827d080".to_owned())
        );
        let completions = [
            Completion::Complete,
            Completion::BudgetExhausted,
            Completion::Cancelled,
        ];
        let digests = completions
            .map(|_| dispositions_digest(ProtocolRevision::draft_v1(), coordinate, &items));
        assert!(digests.windows(2).all(|pair| pair[0] == pair[1]));
        assert_eq!(
            dispositions_digest(
                ProtocolRevision::draft_v1(),
                coordinate,
                &[items[0], items[0]]
            ),
            Err(DigestError::NonCanonical)
        );
    }

    #[test]
    fn canonical_records_generate_every_namespaced_digest_item() {
        let records = [
            crate::DispositionRecord::new(
                crate::ProtocolItemIdentifier::control_event(crate::EventId::from_bytes([1; 32])),
                ProtocolDisposition::Accepted,
                crate::DiagnosticCode::lookup("control.order"),
            ),
            crate::DispositionRecord::new(
                crate::ProtocolItemIdentifier::from(crate::ChangeHash::from_bytes([2; 32])),
                ProtocolDisposition::Excluded,
                None,
            ),
            crate::DispositionRecord::new(
                crate::ProtocolItemIdentifier::event(crate::EventId::from_bytes([3; 32])),
                ProtocolDisposition::Invalid,
                crate::DiagnosticCode::lookup("manifest.semantics"),
            ),
        ];
        let items = disposition_items(&records);
        assert!(items.is_ok());
        let Ok(items) = items else { return };
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].namespace, DispositionNamespace::ControlEvent);
        assert_eq!(items[1].namespace, DispositionNamespace::ChangeHash);
        assert_eq!(items[2].namespace, DispositionNamespace::Event);
        assert_eq!(items[0].identifier, [1; 32]);
        assert_eq!(items[0].disposition, ProtocolDisposition::Accepted);
        assert_eq!(
            disposition_items(&[records[0], records[0]]),
            Err(DigestError::NonCanonical)
        );
        assert_eq!(
            disposition_items(&[records[1], records[0]]),
            Err(DigestError::NonCanonical)
        );
    }
}
