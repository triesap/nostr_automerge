//! Checkpoint verification cannot redefine full-replay semantic commitments.

use nostr_automerge::authoring::{ActorState, AuthoringDocument, Operation};
use nostr_automerge::checkpoint::{CheckpointDescriptor, verify_snapshot_heads};
use nostr_automerge::{ActorId, ChangeHash, NeverCancelled, SnapshotHash, WorkBudget};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

#[test]
#[allow(clippy::expect_used)]
fn prove_checkpoint_and_full_replay_agreement() {
    for (index, name) in ["basic", "concurrent", "revocation", "equivocation"]
        .into_iter()
        .enumerate()
    {
        let mut document = AuthoringDocument::empty(ActorState::initial(
            ActorId::from_bytes([index as u8 + 1; 32]),
            BTreeSet::new(),
        ))
        .expect("document");
        let change = document
            .author_change(&[Operation::PutString {
                key: "scenario".into(),
                value: name.into(),
            }])
            .expect("change");
        let bytes = document.accepted_state_bytes();
        let hash = change.change_hash();
        let mut set_hash = Sha256::new();
        set_hash.update(b"nostr-crdt/automerge/change-set/v1");
        set_hash.update([0]);
        set_hash.update(1_u64.to_be_bytes());
        set_hash.update(hash.as_bytes());
        let descriptor = CheckpointDescriptor {
            snapshot_hash: SnapshotHash::from_bytes(Sha256::digest(&bytes).into()),
            heads: BTreeSet::from([hash]),
            raw_size: bytes.len() as u64,
            chunk_size: bytes.len() as u32,
            chunk_count: 1,
            chunk_root: [0; 32],
            change_count: 1,
            change_set_hash: set_hash.finalize().into(),
            dependency_edges: 0,
            total_ops: 1,
        };
        let checkpoint = verify_snapshot_heads(
            &bytes,
            &descriptor,
            &mut WorkBudget::new(u64::MAX, u64::MAX),
            &NeverCancelled,
        )
        .expect("checkpoint");
        checkpoint
            .verify_commitments(&descriptor, &mut WorkBudget::new(u64::MAX, u64::MAX))
            .expect("commitments");
        checkpoint.verify_exact_closure().expect("closure");
        assert_eq!(checkpoint.heads(), &BTreeSet::<ChangeHash>::from([hash]));
    }
}
