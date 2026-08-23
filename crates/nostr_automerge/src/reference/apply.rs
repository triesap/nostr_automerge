use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::ChangeHash;
use crate::automerge_adapter::document::{AppliedDocument, ExactApplyError};

pub(crate) fn apply_exact_closure(
    closure: &BTreeMap<ChangeHash, Arc<[u8]>>,
    ordered: &[ChangeHash],
    candidate_hash: ChangeHash,
    candidate_raw: &[u8],
    candidate_dependencies: &BTreeSet<ChangeHash>,
) -> Result<AppliedDocument, ExactApplyError> {
    crate::automerge_adapter::document::apply_exact_closure(
        closure,
        ordered,
        candidate_hash,
        candidate_raw,
        candidate_dependencies,
    )
}

pub(crate) fn apply_exact_closure_metered(
    closure: &BTreeMap<ChangeHash, Arc<[u8]>>,
    ordered: &[ChangeHash],
    candidate_hash: ChangeHash,
    candidate_raw: &[u8],
    candidate_dependencies: &BTreeSet<ChangeHash>,
    budget: &mut crate::WorkBudget,
    cancellation: &impl crate::CancellationCheck,
) -> Result<AppliedDocument, ExactApplyError> {
    crate::automerge_adapter::document::apply_exact_closure_metered(
        closure,
        ordered,
        candidate_hash,
        candidate_raw,
        candidate_dependencies,
        budget,
        cancellation,
    )
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::{apply_exact_closure, apply_exact_closure_metered};
    use crate::automerge_adapter::document::ExactApplyError;
    use crate::automerge_adapter::encode::qualify_canonical_reencoding;
    use crate::{ChangeHash, ProtocolRevision};

    fn bytes() -> Vec<u8> {
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
    fn apply_changes_to_exact_dependency_closure() {
        let raw = bytes();
        let decoded = qualify_canonical_reencoding(&raw, ProtocolRevision::draft_v1());
        assert!(decoded.is_ok());
        let hash = match decoded {
            Ok(decoded) => ChangeHash::from_bytes(*decoded.hash.as_bytes()),
            Err(_) => return,
        };
        let applied = apply_exact_closure(&BTreeMap::new(), &[], hash, &raw, &BTreeSet::new());
        assert!(applied.is_ok());
        assert_eq!(
            applied.map(|document| document.heads),
            Ok(BTreeSet::from([hash]))
        );
        assert_eq!(
            apply_exact_closure(
                &BTreeMap::from([(hash, raw.clone().into())]),
                &[],
                hash,
                &raw,
                &BTreeSet::new(),
            ),
            Err(ExactApplyError::ClosureMismatch)
        );
        let mut malformed = raw.clone();
        malformed.truncate(4);
        assert_eq!(
            apply_exact_closure(&BTreeMap::new(), &[], hash, &malformed, &BTreeSet::new(),),
            Err(ExactApplyError::Decode)
        );

        let mut exact = crate::WorkBudget::new(raw.len() as u64, 5);
        assert!(
            apply_exact_closure_metered(
                &BTreeMap::new(),
                &[],
                hash,
                &raw,
                &BTreeSet::new(),
                &mut exact,
                &crate::NeverCancelled,
            )
            .is_ok()
        );
        assert_eq!(exact.consumed().get(crate::WorkCounter::ApplyChange), 2);
        assert_eq!(exact.consumed().get(crate::WorkCounter::GraphNode), 3);
        assert_eq!(
            exact.consumed().get(crate::WorkCounter::DecodeByte),
            raw.len() as u64
        );
        let mut cancelled = crate::WorkBudget::new(raw.len() as u64, 5);
        assert_eq!(
            apply_exact_closure_metered(
                &BTreeMap::new(),
                &[],
                hash,
                &raw,
                &BTreeSet::new(),
                &mut cancelled,
                &|| true,
            ),
            Err(ExactApplyError::Cancelled)
        );
        assert_eq!(cancelled.consumed().get(crate::WorkCounter::ApplyChange), 0);
    }
}
