use super::CheckpointDescriptor;
use crate::{CancellationCheck, ChangeHash, WorkBudget};
use std::collections::BTreeSet;

/// A strictly loaded checkpoint whose declared heads match exactly.
pub struct VerifiedSnapshot {
    pub(crate) loaded: crate::automerge_adapter::checkpoint::LoadedCheckpoint,
    heads: BTreeSet<ChangeHash>,
}
impl VerifiedSnapshot {
    /// Returns exact loaded semantic heads.
    #[must_use]
    pub fn heads(&self) -> &BTreeSet<ChangeHash> {
        &self.heads
    }
    /// Recomputes and verifies all embedded count and change-set commitments.
    pub fn verify_commitments(
        &self,
        descriptor: &CheckpointDescriptor,
        budget: &mut WorkBudget,
    ) -> Result<(), VerifyError> {
        let changes = self
            .loaded
            .document
            .embedded_changes()
            .map_err(|_| VerifyError::Commitments)?;
        budget
            .charge_checkpoint_items(changes.len() as u64)
            .map_err(|_| VerifyError::Budget)?;
        let change_count = changes.len() as u64;
        let total_ops = changes
            .iter()
            .try_fold(0_u64, |sum, c| sum.checked_add(c.operations))
            .ok_or(VerifyError::Commitments)?;
        let dependency_edges = changes
            .iter()
            .try_fold(0_u64, |sum, c| sum.checked_add(c.dependencies.len() as u64))
            .ok_or(VerifyError::Commitments)?;
        if !within_checkpoint_limits(change_count, total_ops, dependency_edges) {
            return Err(VerifyError::Commitments);
        }
        let hashes = changes.iter().map(|c| c.hash).collect::<Vec<_>>();
        if change_count != descriptor.change_count
            || total_ops != descriptor.total_ops
            || dependency_edges != descriptor.dependency_edges
            || change_set_hash(hashes)? != descriptor.change_set_hash
        {
            return Err(VerifyError::Commitments);
        }
        Ok(())
    }
    /// Requires the embedded change set to equal the complete ancestor closure of heads.
    pub fn verify_exact_closure(&self) -> Result<(), VerifyError> {
        let changes = self
            .loaded
            .document
            .embedded_changes()
            .map_err(|_| VerifyError::Closure)?;
        let map = changes
            .iter()
            .map(|change| (change.hash, change.dependencies.clone()))
            .collect::<std::collections::BTreeMap<_, _>>();
        exact_closure(&map, &self.heads)
    }

    pub(crate) fn verify_exact_closure_metered(
        &self,
        budget: &mut WorkBudget,
    ) -> Result<(), VerifyError> {
        let changes = self
            .loaded
            .document
            .embedded_changes()
            .map_err(|_| VerifyError::Closure)?;
        let work = changes
            .iter()
            .try_fold(0_u64, |total, change| {
                total
                    .checked_add(1)?
                    .checked_add(u64::try_from(change.dependencies.len()).unwrap_or(u64::MAX))
            })
            .ok_or(VerifyError::Budget)?;
        budget
            .charge_checkpoint_items(work)
            .map_err(|_| VerifyError::Budget)?;
        let map = changes
            .iter()
            .map(|change| (change.hash, change.dependencies.clone()))
            .collect::<std::collections::BTreeMap<_, _>>();
        exact_closure(&map, &self.heads)
    }
}

fn change_set_hash(mut hashes: Vec<ChangeHash>) -> Result<[u8; 32], VerifyError> {
    use sha2::{Digest, Sha256};

    hashes.sort();
    let count = u64::try_from(hashes.len()).map_err(|_| VerifyError::Commitments)?;
    let mut digest = Sha256::new();
    digest.update(b"nostr-crdt/automerge/change-set/v1");
    digest.update([0]);
    digest.update(count.to_be_bytes());
    for hash in hashes {
        digest.update(hash.as_bytes());
    }
    Ok(digest.finalize().into())
}

fn within_checkpoint_limits(change_count: u64, operations: u64, dependency_edges: u64) -> bool {
    let limits = crate::ProtocolRevision::draft_v1().limits();
    change_count <= limits.checkpoint_changes.get()
        && operations <= limits.checkpoint_operations.get()
        && dependency_edges <= limits.checkpoint_dependency_edges.get()
}

fn exact_closure(
    map: &std::collections::BTreeMap<ChangeHash, Vec<ChangeHash>>,
    heads: &BTreeSet<ChangeHash>,
) -> Result<(), VerifyError> {
    let mut reachable = BTreeSet::new();
    let mut stack = heads.iter().copied().collect::<Vec<_>>();
    while let Some(hash) = stack.pop() {
        if !reachable.insert(hash) {
            continue;
        }
        let deps = map.get(&hash).ok_or(VerifyError::Closure)?;
        for dep in deps {
            if *dep == hash {
                return Err(VerifyError::Closure);
            }
            stack.push(*dep);
        }
    }
    if reachable == map.keys().copied().collect() {
        Ok(())
    } else {
        Err(VerifyError::Closure)
    }
}
/// Why snapshot semantic commitments differed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerifyError {
    /// Caller-selected checkpoint work budget was exhausted.
    Budget,
    /// Caller requested cooperative cancellation.
    Cancelled,
    /// Hardened load or local work policy failed.
    Load,
    /// Loaded heads differed from the signed descriptor.
    Heads,
    /// Embedded commitments differed.
    Commitments,
    /// Embedded graph was not the exact reachable closure.
    Closure,
}

/// Loads a complete snapshot and requires byte-for-byte sorted head equality.
pub fn verify_snapshot_heads<C: CancellationCheck>(
    bytes: &[u8],
    descriptor: &CheckpointDescriptor,
    budget: &mut WorkBudget,
    cancellation: &C,
) -> Result<VerifiedSnapshot, VerifyError> {
    let loaded = crate::automerge_adapter::checkpoint::load(bytes, budget, cancellation).map_err(
        |error| match error {
            crate::automerge_adapter::checkpoint::CheckpointLoadError::Budget => {
                VerifyError::Budget
            }
            crate::automerge_adapter::checkpoint::CheckpointLoadError::Cancelled => {
                VerifyError::Cancelled
            }
            crate::automerge_adapter::checkpoint::CheckpointLoadError::Invalid => VerifyError::Load,
        },
    )?;
    let heads = loaded
        .document
        .semantic_heads()
        .map_err(|_| VerifyError::Load)?;
    if heads != descriptor.heads {
        return Err(VerifyError::Heads);
    }
    Ok(VerifiedSnapshot { loaded, heads })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ActorId, NeverCancelled, SnapshotHash,
        authoring::{ActorState, AuthoringDocument, Operation},
    };
    use sha2::{Digest, Sha256};
    #[test]
    fn commit_empty_sorted_change_set_hash() {
        assert_eq!(
            change_set_hash(Vec::new()).ok(),
            Some([
                0xcc, 0xe1, 0x46, 0xbe, 0x40, 0x7e, 0xe7, 0xaa, 0xe2, 0xe3, 0xfd, 0x5e, 0x4b, 0x12,
                0x49, 0xb5, 0x00, 0x05, 0xb5, 0xb9, 0xa6, 0xe4, 0xe0, 0x0a, 0x2c, 0x5f, 0x65, 0xc8,
                0x52, 0xfa, 0x42, 0x6c,
            ])
        );
    }
    #[test]
    fn verify_declared_checkpoint_heads() {
        let mut doc = AuthoringDocument::empty(ActorState::initial(
            ActorId::from_bytes([1; 32]),
            BTreeSet::new(),
        ))
        .ok();
        assert!(doc.is_some());
        let authored = doc.as_mut().and_then(|d| {
            d.author_change(&[Operation::PutString {
                key: "k".into(),
                value: "v".into(),
            }])
            .ok()
        });
        assert!(authored.is_some());
        let bytes = doc.map(|d| d.accepted_state_bytes()).unwrap_or_default();
        let heads = BTreeSet::from([authored
            .map(|a| a.change_hash())
            .unwrap_or(ChangeHash::from_bytes([0; 32]))]);
        let descriptor = CheckpointDescriptor {
            snapshot_hash: SnapshotHash::from_bytes(Sha256::digest(&bytes).into()),
            heads: heads.clone(),
            raw_size: bytes.len() as u64,
            chunk_size: bytes.len() as u32,
            chunk_count: 1,
            chunk_root: [0; 32],
            change_count: 1,
            change_set_hash: [0; 32],
            dependency_edges: 0,
            total_ops: 1,
        };
        assert!(
            verify_snapshot_heads(
                &bytes,
                &descriptor,
                &mut WorkBudget::new(u64::MAX, u64::MAX),
                &NeverCancelled
            )
            .is_ok()
        );
        let mut wrong = descriptor;
        wrong.heads = BTreeSet::from([ChangeHash::from_bytes([9; 32])]);
        assert!(matches!(
            verify_snapshot_heads(
                &bytes,
                &wrong,
                &mut WorkBudget::new(u64::MAX, u64::MAX),
                &NeverCancelled
            ),
            Err(VerifyError::Heads)
        ));
    }
    #[test]
    fn enumerate_embedded_changes_and_counts() {
        let mut doc = AuthoringDocument::empty(ActorState::initial(
            ActorId::from_bytes([2; 32]),
            BTreeSet::new(),
        ))
        .ok();
        assert!(doc.is_some());
        let authored = doc.as_mut().and_then(|d| {
            d.author_change(&[Operation::PutString {
                key: "k".into(),
                value: "v".into(),
            }])
            .ok()
        });
        let bytes = doc.map(|d| d.accepted_state_bytes()).unwrap_or_default();
        let hash = authored
            .map(|a| a.change_hash())
            .unwrap_or(ChangeHash::from_bytes([0; 32]));
        let mut digest = Sha256::new();
        digest.update(b"nostr-crdt/automerge/change-set/v1");
        digest.update([0]);
        digest.update(1_u64.to_be_bytes());
        digest.update(hash.as_bytes());
        let descriptor = CheckpointDescriptor {
            snapshot_hash: SnapshotHash::from_bytes(Sha256::digest(&bytes).into()),
            heads: BTreeSet::from([hash]),
            raw_size: bytes.len() as u64,
            chunk_size: bytes.len() as u32,
            chunk_count: 1,
            chunk_root: [0; 32],
            change_count: 1,
            change_set_hash: digest.finalize().into(),
            dependency_edges: 0,
            total_ops: 1,
        };
        let loaded = verify_snapshot_heads(
            &bytes,
            &descriptor,
            &mut WorkBudget::new(u64::MAX, u64::MAX),
            &NeverCancelled,
        )
        .ok();
        assert!(loaded.is_some());
        assert_eq!(
            loaded.map(
                |v| v.verify_commitments(&descriptor, &mut WorkBudget::new(u64::MAX, u64::MAX))
            ),
            Some(Ok(()))
        );
    }
    #[test]
    fn verify_exact_reachable_ancestor_closure() {
        use std::collections::BTreeMap;
        let a = ChangeHash::from_bytes([1; 32]);
        let b = ChangeHash::from_bytes([2; 32]);
        let extra = ChangeHash::from_bytes([3; 32]);
        let valid = BTreeMap::from([(a, vec![]), (b, vec![a])]);
        assert_eq!(super::exact_closure(&valid, &BTreeSet::from([b])), Ok(()));
        let disconnected = BTreeMap::from([(a, vec![]), (b, vec![a]), (extra, vec![])]);
        assert_eq!(
            super::exact_closure(&disconnected, &BTreeSet::from([b])),
            Err(VerifyError::Closure)
        );
        let missing = BTreeMap::from([(b, vec![a])]);
        assert_eq!(
            super::exact_closure(&missing, &BTreeSet::from([b])),
            Err(VerifyError::Closure)
        );
        let cycle = BTreeMap::from([(a, vec![a])]);
        assert_eq!(
            super::exact_closure(&cycle, &BTreeSet::from([a])),
            Err(VerifyError::Closure)
        );
    }
    #[test]
    fn checkpoint_graph_limits_remain_checkpoint_specific() {
        let limits = crate::ProtocolRevision::draft_v1().limits();
        assert!(super::within_checkpoint_limits(
            limits.checkpoint_changes.get(),
            limits.checkpoint_operations.get(),
            limits.checkpoint_dependency_edges.get(),
        ));
        assert!(!super::within_checkpoint_limits(
            limits.checkpoint_changes.get() + 1,
            limits.checkpoint_operations.get(),
            limits.checkpoint_dependency_edges.get(),
        ));
        assert!(!super::within_checkpoint_limits(
            limits.checkpoint_changes.get(),
            limits.checkpoint_operations.get(),
            limits.checkpoint_dependency_edges.get() + 1,
        ));
    }
}
