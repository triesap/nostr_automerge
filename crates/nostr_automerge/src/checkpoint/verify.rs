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
}
/// Why snapshot semantic commitments differed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerifyError {
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
    let loaded = crate::automerge_adapter::checkpoint::load(bytes, budget, cancellation)
        .map_err(|_| VerifyError::Load)?;
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
}
