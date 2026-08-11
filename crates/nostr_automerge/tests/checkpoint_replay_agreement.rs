//! Checkpoint verification cannot redefine full-replay semantic commitments.

use std::collections::BTreeSet;

use automerge::{Automerge, Change, TextEncoding};
use nostr_automerge::authoring::{ActorState, AuthoringDocument, Operation};
use nostr_automerge::checkpoint::{
    CheckpointDescriptor, VerifyError, verify_full_history, verify_snapshot_heads,
};
use nostr_automerge::{ActorId, ChangeHash, NeverCancelled, SnapshotHash, WorkBudget};
use sha2::{Digest, Sha256};

#[test]
#[allow(clippy::expect_used)]
fn concurrent_checkpoint_matches_full_replay() {
    let (left_raw, left_hash) = independent_change(1, "left", "a");
    let (right_raw, right_hash) = independent_change(2, "right", "b");
    let mut replay = Automerge::new_with_encoding(TextEncoding::Utf16CodeUnit);
    replay
        .apply_changes([
            Change::try_from(left_raw.as_slice()).expect("left change"),
            Change::try_from(right_raw.as_slice()).expect("right change"),
        ])
        .expect("concurrent replay");
    let bytes = replay.save_nocompress();
    let accepted = BTreeSet::from([left_hash, right_hash]);
    let checkpoint = verified_checkpoint(&bytes, accepted.clone(), 2, 0, 2);
    assert_eq!(checkpoint.heads(), &accepted);
    assert_eq!(
        verify_full_history(&checkpoint, &accepted, &accepted),
        Ok(())
    );
}

#[test]
#[allow(clippy::expect_used)]
fn revoked_and_equivocated_candidates_do_not_enter_checkpoint_history() {
    let actor = ActorId::from_bytes([3; 32]);
    let mut accepted_document =
        AuthoringDocument::empty(ActorState::initial(actor, BTreeSet::new())).expect("document");
    let accepted_change = accepted_document
        .author_change(&[Operation::PutString {
            key: "accepted".into(),
            value: "base".into(),
        }])
        .expect("accepted change");
    let accepted_hash = accepted_change.change_hash();
    let accepted_bytes = accepted_document.accepted_state_bytes();
    let next_state = accepted_change.new_state().clone();

    let mut revoked_branch = AuthoringDocument::from_accepted(&accepted_bytes, next_state.clone())
        .expect("revoked base");
    let revoked = revoked_branch
        .author_change(&[Operation::PutString {
            key: "revoked".into(),
            value: "excluded".into(),
        }])
        .expect("revoked candidate");

    let mut equivocated_left =
        AuthoringDocument::from_accepted(&accepted_bytes, next_state.clone()).expect("left base");
    let left = equivocated_left
        .author_change(&[Operation::PutString {
            key: "fork".into(),
            value: "left".into(),
        }])
        .expect("left candidate");
    let mut equivocated_right =
        AuthoringDocument::from_accepted(&accepted_bytes, next_state).expect("right base");
    let right = equivocated_right
        .author_change(&[Operation::PutString {
            key: "fork".into(),
            value: "right".into(),
        }])
        .expect("right candidate");
    assert_ne!(left.change_hash(), right.change_hash());

    let accepted = BTreeSet::from([accepted_hash]);
    let carriers = BTreeSet::from([
        accepted_hash,
        revoked.change_hash(),
        left.change_hash(),
        right.change_hash(),
    ]);
    let checkpoint = verified_checkpoint(&accepted_bytes, accepted.clone(), 1, 0, 1);
    assert_eq!(checkpoint.heads(), &accepted);
    assert_eq!(
        verify_full_history(&checkpoint, &carriers, &accepted),
        Ok(())
    );
    assert!(!checkpoint.heads().contains(&revoked.change_hash()));
    assert!(!checkpoint.heads().contains(&left.change_hash()));
    assert!(!checkpoint.heads().contains(&right.change_hash()));
}

#[test]
#[allow(clippy::expect_used)]
fn checkpoint_closure_refusals() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/v1_draft/checkpoints/negative_closure.json"
    ))
    .expect("closure refusal fixture");
    assert_eq!(fixture["cases"].as_array().map(Vec::len), Some(1));
    let (left_raw, left_hash) = independent_change(7, "left", "retained");
    let (right_raw, right_hash) = independent_change(8, "right", "disconnected");
    let mut replay = Automerge::new_with_encoding(TextEncoding::Utf16CodeUnit);
    replay
        .apply_changes([
            Change::try_from(left_raw.as_slice()).expect("left change"),
            Change::try_from(right_raw.as_slice()).expect("right change"),
        ])
        .expect("concurrent replay");
    let bytes = replay.save_nocompress();
    assert_eq!(
        replay
            .get_heads()
            .into_iter()
            .map(|hash| ChangeHash::from_bytes(hash.0))
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([left_hash, right_hash])
    );
    let descriptor = CheckpointDescriptor {
        snapshot_hash: SnapshotHash::from_bytes(Sha256::digest(&bytes).into()),
        heads: BTreeSet::from([left_hash]),
        raw_size: bytes.len() as u64,
        chunk_size: bytes.len() as u32,
        chunk_count: 1,
        chunk_root: [0; 32],
        change_count: 2,
        change_set_hash: [0; 32],
        dependency_edges: 0,
        total_ops: 2,
    };
    assert!(matches!(
        verify_snapshot_heads(
            &bytes,
            &descriptor,
            &mut WorkBudget::new(u64::MAX, u64::MAX),
            &NeverCancelled,
        ),
        Err(VerifyError::Closure)
    ));
}

#[allow(clippy::expect_used)]
fn independent_change(actor: u8, key: &str, value: &str) -> (Vec<u8>, ChangeHash) {
    let mut document = AuthoringDocument::empty(ActorState::initial(
        ActorId::from_bytes([actor; 32]),
        BTreeSet::new(),
    ))
    .expect("document");
    let change = document
        .author_change(&[Operation::PutString {
            key: key.into(),
            value: value.into(),
        }])
        .expect("change");
    (change.raw().to_vec(), change.change_hash())
}

#[allow(clippy::expect_used)]
fn verified_checkpoint(
    bytes: &[u8],
    heads: BTreeSet<ChangeHash>,
    change_count: u64,
    dependency_edges: u64,
    total_ops: u64,
) -> nostr_automerge::checkpoint::VerifiedSnapshot {
    let mut set_hash = Sha256::new();
    set_hash.update(b"nostr-crdt/automerge/change-set/v1");
    set_hash.update([0]);
    set_hash.update(change_count.to_be_bytes());
    for hash in &heads {
        set_hash.update(hash.as_bytes());
    }
    let descriptor = CheckpointDescriptor {
        snapshot_hash: SnapshotHash::from_bytes(Sha256::digest(bytes).into()),
        heads,
        raw_size: bytes.len() as u64,
        chunk_size: bytes.len() as u32,
        chunk_count: 1,
        chunk_root: [0; 32],
        change_count,
        change_set_hash: set_hash.finalize().into(),
        dependency_edges,
        total_ops,
    };
    let checkpoint = verify_snapshot_heads(
        bytes,
        &descriptor,
        &mut WorkBudget::new(u64::MAX, u64::MAX),
        &NeverCancelled,
    )
    .expect("checkpoint");
    checkpoint
        .verify_commitments(&descriptor, &mut WorkBudget::new(u64::MAX, u64::MAX))
        .expect("commitments");
    checkpoint.verify_exact_closure().expect("closure");
    checkpoint
}
