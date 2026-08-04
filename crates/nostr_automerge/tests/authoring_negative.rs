//! Stable authoring refusal and canonicalization conformance cases.

use std::collections::BTreeSet;

use nostr_automerge::authoring::{
    ActorState, AuthoringDocument, AuthoringError, CommitMetadata, CommitMetadataError,
    ControlDraft, ControlDraftError, ControlGrant, ControlRole, FanInError, FanInPlan, Operation,
};
use nostr_automerge::{ActorId, ChangeHash, DevicePublicKey, ProtocolRevision};

#[test]
#[allow(clippy::expect_used)]
fn add_authoring_negative_conformance_fixtures() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/v1_draft/authoring/refusals.json"
    ))
    .expect("committed fixture JSON");
    let diagnostics = fixture["cases"]
        .as_array()
        .expect("fixture cases")
        .iter()
        .map(|case| case["diagnostic"].as_str().expect("diagnostic"))
        .collect::<Vec<_>>();
    assert_eq!(diagnostics.len(), 8);
    assert_eq!(
        CommitMetadata::validate(1, None, &[]),
        Err(CommitMetadataError)
    );

    let state = ActorState::initial(ActorId::from_bytes([1; 32]), BTreeSet::new());
    let mut document = AuthoringDocument::empty(state.clone()).expect("empty authoring document");
    let before = document.accepted_state_bytes();
    let count = ProtocolRevision::draft_v1()
        .limits()
        .change_operations
        .get()
        .saturating_add(1);
    let operations = (0..count)
        .map(|index| Operation::PutString {
            key: format!("k{index}"),
            value: "v".to_owned(),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        document.author_change(&operations),
        Err(AuthoringError::Limit)
    );
    assert_eq!(document.actor_state(), &state);
    assert_eq!(document.accepted_state_bytes(), before);

    let dependencies = (0_u16..257)
        .map(|index| {
            let mut bytes = [0_u8; 32];
            bytes[30..].copy_from_slice(&index.to_be_bytes());
            ChangeHash::from_bytes(bytes)
        })
        .collect();
    assert_eq!(
        FanInPlan::new(&dependencies),
        Err(FanInError::DependencyLimit)
    );

    let invalid_control = ControlDraft::new(
        0,
        BTreeSet::new(),
        vec![ControlGrant {
            account: None,
            device: DevicePublicKey::from_bytes([2; 32]),
            roles: BTreeSet::from([ControlRole::Write]),
        }],
        None,
        Some(nostr_automerge::DocumentCoordinate::new(
            nostr_automerge::ControllerPublicKey::from_bytes([3; 32]),
            nostr_automerge::DocumentId::from_bytes([4; 32]),
        )),
    );
    assert_eq!(invalid_control, Err(ControlDraftError::InvalidTransition));
}
