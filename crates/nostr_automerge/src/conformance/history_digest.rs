use sha2::{Digest, Sha256};

use super::digest::DigestError;
use crate::{ChangeHash, DocumentCoordinate, EventId, HistoryDigest, ProtocolRevision};

const DOMAIN: &[u8] = b"nostr-crdt/automerge/history/v1\0";
const MANIFEST_KIND: u32 = 31_624;

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
    encoder.update(DOMAIN);
    encoder.update(revision_len.to_be_bytes());
    encoder.update(revision);
    encoder.update(MANIFEST_KIND.to_be_bytes());
    encoder.update(coordinate.controller().as_bytes());
    encoder.update(coordinate.document_id().as_bytes());
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

fn canonical<T: Ord>(items: &[T]) -> Result<(), DigestError> {
    items
        .windows(2)
        .all(|pair| pair[0] < pair[1])
        .then_some(())
        .ok_or(DigestError::NonCanonical)
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use super::history_digest;
    use crate::conformance::digest::DigestError;
    use crate::{ChangeHash, DocumentCoordinate, EventId, ProtocolRevision};

    #[test]
    fn implement_normative_history_digest_encoder() {
        let coordinate =
            DocumentCoordinate::from_str(&format!("31624:{}:{}", "11".repeat(32), "22".repeat(32)));
        assert!(coordinate.is_ok());
        let Ok(coordinate) = coordinate else { return };
        let controls = [
            EventId::from_bytes([0xaa; 32]),
            EventId::from_bytes([0xbb; 32]),
        ];
        let accepted = [
            ChangeHash::from_bytes([0xcc; 32]),
            ChangeHash::from_bytes([0xdd; 32]),
        ];
        assert_eq!(
            history_digest(
                ProtocolRevision::draft_v1(),
                coordinate,
                &controls,
                &accepted,
                &accepted[1..]
            )
            .map(|digest| digest.to_hex()),
            Ok("796bd40b8e9912a14b0b464133c80d5fafd552c2caa870cf3b7eaa9af0bcdb2e".to_owned())
        );
        assert_eq!(
            history_digest(
                ProtocolRevision::draft_v1(),
                coordinate,
                &controls,
                &[accepted[1], accepted[0]],
                &accepted[1..]
            ),
            Err(DigestError::NonCanonical)
        );
        assert_eq!(
            history_digest(
                ProtocolRevision::draft_v1(),
                coordinate,
                &controls,
                &accepted,
                &[accepted[1], accepted[1]]
            ),
            Err(DigestError::NonCanonical)
        );
    }
}
