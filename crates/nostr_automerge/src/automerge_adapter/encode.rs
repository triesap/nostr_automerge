use automerge::Change;

use crate::ProtocolRevision;

use super::decode::{DecodeError, decode_change};
use super::framing::{FramingError, validate_change_frame};
use super::types::DecodedChange;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ReencodeError {
    Framing(FramingError),
    Upstream,
    TooManyOperations,
    TooManyDependencies,
    ActorLength,
    PredecessorOrder,
    Semantics(DecodeError),
    NonCanonical,
}

pub(crate) fn qualify_canonical_reencoding(
    raw: &[u8],
    revision: ProtocolRevision,
) -> Result<DecodedChange, ReencodeError> {
    let validated = validate_change_frame(raw, revision).map_err(ReencodeError::Framing)?;
    let change = Change::try_from(validated.raw).map_err(|_| ReencodeError::Upstream)?;
    let limits = revision.limits();
    let operation_limit = limits
        .change_operations
        .try_usize()
        .map_err(|_| ReencodeError::TooManyOperations)?;
    let dependency_limit = limits
        .change_dependencies
        .try_usize()
        .map_err(|_| ReencodeError::TooManyDependencies)?;
    if change.len() > operation_limit {
        return Err(ReencodeError::TooManyOperations);
    }
    if change.deps().len() > dependency_limit {
        return Err(ReencodeError::TooManyDependencies);
    }
    if change.actors().any(|actor| actor.to_bytes().len() != 32) {
        return Err(ReencodeError::ActorLength);
    }

    let expanded = change.decode();
    if expanded.operations.iter().any(|operation| {
        operation
            .pred
            .iter()
            .zip(operation.pred.iter().skip(1))
            .any(|(left, right)| left > right)
    }) {
        return Err(ReencodeError::PredecessorOrder);
    }
    let reencoded = Change::from(expanded);
    require_identical(reencoded.raw_bytes(), raw)?;
    decode_change(raw, revision).map_err(ReencodeError::Semantics)
}

fn require_identical(reencoded: &[u8], raw: &[u8]) -> Result<(), ReencodeError> {
    if reencoded == raw {
        Ok(())
    } else {
        Err(ReencodeError::NonCanonical)
    }
}

#[cfg(test)]
mod tests {
    use automerge::{
        ActorId, Automerge, ROOT, TextEncoding,
        transaction::{CommitOptions, Transactable},
    };

    use super::{ReencodeError, qualify_canonical_reencoding, require_identical};
    use crate::ProtocolRevision;

    fn fixture() -> Vec<u8> {
        include_str!("../../../../fixtures/v1_draft/automerge_changes/basic/change.hex")
            .trim()
            .as_bytes()
            .chunks_exact(2)
            .filter_map(|pair| {
                core::str::from_utf8(pair)
                    .ok()
                    .and_then(|text| u8::from_str_radix(text, 16).ok())
            })
            .collect()
    }

    #[test]
    fn qualify_canonical_uncompressed_re_encoding() {
        let basic = fixture();
        assert!(qualify_canonical_reencoding(&basic, ProtocolRevision::draft_v1()).is_ok());

        let mut document = Automerge::new_with_encoding(TextEncoding::Utf16CodeUnit);
        document.set_actor(ActorId::from([1_u8; 32]));
        {
            let mut transaction = document.transaction();
            assert!(transaction.put(ROOT, "key", "first").is_ok());
            transaction.commit();
        }
        let first_head = document.get_heads();
        document.set_actor(ActorId::from([2_u8; 32]));
        {
            let mut transaction = document.transaction();
            assert!(transaction.put(ROOT, "key", "second").is_ok());
            transaction.commit();
        }
        let dependent = document.get_changes(&first_head);
        assert_eq!(dependent.len(), 1);
        assert!(
            qualify_canonical_reencoding(dependent[0].raw_bytes(), ProtocolRevision::draft_v1())
                .is_ok()
        );

        document.set_actor(ActorId::from([3_u8; 32]));
        document.empty_commit(CommitOptions::default());
        let empty = document.get_changes(&document.get_heads()[..0]);
        let empty = empty.iter().find(|change| change.is_empty());
        assert!(empty.is_some());
        assert!(empty.is_some_and(|change| {
            qualify_canonical_reencoding(change.raw_bytes(), ProtocolRevision::draft_v1()).is_ok()
        }));

        let mut unequal = basic.clone();
        unequal.push(0);
        assert_eq!(
            require_identical(&basic, &unequal),
            Err(ReencodeError::NonCanonical)
        );
    }
}
