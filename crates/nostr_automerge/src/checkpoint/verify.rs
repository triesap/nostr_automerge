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
        self.verify_commitments_metered(descriptor, budget, &crate::NeverCancelled)
    }

    pub(crate) fn verify_commitments_metered(
        &self,
        descriptor: &CheckpointDescriptor,
        budget: &mut WorkBudget,
        cancellation: &impl CancellationCheck,
    ) -> Result<(), VerifyError> {
        charge_checkpoint_item(budget, cancellation)?;
        let changes = self
            .loaded
            .document
            .embedded_changes()
            .map_err(|_| VerifyError::Commitments)?;
        let mut change_count = 0_u64;
        let mut total_ops = 0_u64;
        let mut dependency_edges = 0_u64;
        let mut hashes = BTreeSet::new();
        for change in &changes {
            charge_checkpoint_item(budget, cancellation)?;
            change_count = change_count
                .checked_add(1)
                .ok_or(VerifyError::Commitments)?;
            for _ in 0..change.operations {
                charge_checkpoint_item(budget, cancellation)?;
                total_ops = total_ops.checked_add(1).ok_or(VerifyError::Commitments)?;
            }
            for _ in &change.dependencies {
                charge_checkpoint_item(budget, cancellation)?;
                dependency_edges = dependency_edges
                    .checked_add(1)
                    .ok_or(VerifyError::Commitments)?;
            }
            if !hashes.insert(change.hash) {
                return Err(VerifyError::Commitments);
            }
        }
        if !within_checkpoint_limits(change_count, total_ops, dependency_edges) {
            return Err(VerifyError::Commitments);
        }
        if change_count != descriptor.change_count
            || total_ops != descriptor.total_ops
            || dependency_edges != descriptor.dependency_edges
            || change_set_hash_metered(&hashes, budget, cancellation)? != descriptor.change_set_hash
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
        cancellation: &impl CancellationCheck,
    ) -> Result<(), VerifyError> {
        charge_checkpoint_item(budget, cancellation)?;
        let changes = self
            .loaded
            .document
            .embedded_changes()
            .map_err(|_| VerifyError::Closure)?;
        let map = metered_dependency_map(&changes, budget, cancellation)?;
        exact_closure_metered(&map, &self.heads, budget, cancellation)
    }
}

#[cfg(test)]
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

fn change_set_hash_metered(
    hashes: &BTreeSet<ChangeHash>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<[u8; 32], VerifyError> {
    use sha2::{Digest, Sha256};

    let count = u64::try_from(hashes.len()).map_err(|_| VerifyError::Commitments)?;
    let mut digest = Sha256::new();
    digest.update(b"nostr-crdt/automerge/change-set/v1");
    digest.update([0]);
    digest.update(count.to_be_bytes());
    for hash in hashes {
        charge_checkpoint_item(budget, cancellation)?;
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

fn metered_dependency_map(
    changes: &[crate::automerge_adapter::document::EmbeddedChange],
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<std::collections::BTreeMap<ChangeHash, Vec<ChangeHash>>, VerifyError> {
    let mut map = std::collections::BTreeMap::new();
    for change in changes {
        charge_checkpoint_item(budget, cancellation)?;
        let mut dependencies = Vec::new();
        for dependency in &change.dependencies {
            charge_checkpoint_item(budget, cancellation)?;
            dependencies.push(*dependency);
        }
        if map.insert(change.hash, dependencies).is_some() {
            return Err(VerifyError::Closure);
        }
    }
    Ok(map)
}

fn exact_closure_metered(
    map: &std::collections::BTreeMap<ChangeHash, Vec<ChangeHash>>,
    heads: &BTreeSet<ChangeHash>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<(), VerifyError> {
    let mut reachable = BTreeSet::new();
    let mut stack = Vec::new();
    for head in heads {
        charge_checkpoint_item(budget, cancellation)?;
        stack.push(*head);
    }
    while let Some(hash) = stack.pop() {
        charge_checkpoint_item(budget, cancellation)?;
        if !reachable.insert(hash) {
            continue;
        }
        let deps = map.get(&hash).ok_or(VerifyError::Closure)?;
        for dep in deps {
            charge_checkpoint_item(budget, cancellation)?;
            if *dep == hash {
                return Err(VerifyError::Closure);
            }
            stack.push(*dep);
        }
    }
    if reachable.len() != map.len() {
        return Err(VerifyError::Closure);
    }
    for hash in map.keys() {
        charge_checkpoint_item(budget, cancellation)?;
        if !reachable.contains(hash) {
            return Err(VerifyError::Closure);
        }
    }
    Ok(())
}

fn charge_checkpoint_item(
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<(), VerifyError> {
    if cancellation.is_cancelled() {
        return Err(VerifyError::Cancelled);
    }
    budget
        .charge_checkpoint_items(1)
        .map_err(|_| VerifyError::Budget)
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
    charge_checkpoint_item(budget, cancellation)?;
    let heads = loaded
        .document
        .semantic_heads()
        .map_err(|_| VerifyError::Load)?;
    let heads_match = if heads.len() != descriptor.heads.len() {
        false
    } else {
        let mut matches = true;
        for (actual, expected) in heads.iter().zip(&descriptor.heads) {
            charge_checkpoint_item(budget, cancellation)?;
            matches &= actual == expected;
        }
        matches
    };
    if !heads_match {
        charge_checkpoint_item(budget, cancellation)?;
        let changes = loaded
            .document
            .embedded_changes()
            .map_err(|_| VerifyError::Load)?;
        let map = metered_dependency_map(&changes, budget, cancellation)?;
        let mut declared_heads_present = true;
        for head in &descriptor.heads {
            charge_checkpoint_item(budget, cancellation)?;
            declared_heads_present &= map.contains_key(head);
        }
        if declared_heads_present {
            match exact_closure_metered(&map, &descriptor.heads, budget, cancellation) {
                Ok(()) => {}
                Err(VerifyError::Budget) => return Err(VerifyError::Budget),
                Err(VerifyError::Cancelled) => return Err(VerifyError::Cancelled),
                Err(_) => return Err(VerifyError::Closure),
            }
        }
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
