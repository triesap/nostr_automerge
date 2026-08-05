//! Evaluate signed in-memory evidence through the public trusted engine.

use std::collections::BTreeSet;

use base64::Engine as _;
use nostr_automerge::authoring::{
    ActorState, AuthoringDocument, ControlDraft, ControlGrant, ControlRole, Operation,
    PreparedEvent, UnsignedEventDraft,
};
use nostr_automerge::{
    ActorId, Completion, ControllerPublicKey, CorpusBuilder, DevicePublicKey, DocumentCoordinate,
    DocumentId, IngestOutcome, NeverCancelled, ProtocolRevision, RawEventBytes, ReferenceEvaluator,
    VerifiedNip01Event, WorkBudget,
};
use secp256k1::{Keypair, Secp256k1, SecretKey};

fn sign(
    prepared: &PreparedEvent,
    keys: &Keypair,
) -> Result<RawEventBytes, Box<dyn std::error::Error>> {
    let signature = Secp256k1::new().sign_schnorr_no_aux_rand(prepared.event_id().as_bytes(), keys);
    let raw_json = serde_json::to_vec(&serde_json::json!({
        "id": prepared.event_id().to_hex(),
        "pubkey": prepared.public_key().to_hex(),
        "created_at": prepared.created_at(),
        "kind": prepared.kind(),
        "tags": prepared.tags(),
        "content": prepared.content(),
        "sig": signature.to_string(),
    }))?;
    Ok(RawEventBytes::new(&raw_json, ProtocolRevision::draft_v1())?)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let secp = Secp256k1::new();
    let controller_keys = Keypair::from_secret_key(&secp, &SecretKey::from_byte_array([1; 32])?);
    let device_keys = Keypair::from_secret_key(&secp, &SecretKey::from_byte_array([2; 32])?);
    let (controller_x_only, _) = controller_keys.x_only_public_key();
    let (device_x_only, _) = device_keys.x_only_public_key();
    let controller = ControllerPublicKey::from_bytes(controller_x_only.serialize());
    let controller_device = DevicePublicKey::from_bytes(controller_x_only.serialize());
    let device = DevicePublicKey::from_bytes(device_x_only.serialize());
    let coordinate = DocumentCoordinate::new(controller, DocumentId::from_bytes([3; 32]));
    let actor = ActorId::derive(coordinate, device);

    let control = ControlDraft::new(
        0,
        BTreeSet::new(),
        vec![ControlGrant {
            account: None,
            device,
            roles: BTreeSet::from([ControlRole::Checkpoint, ControlRole::Write]),
        }],
        None,
        None,
    )
    .map_err(|error| format!("control draft: {error:?}"))?;
    let prepared_control = UnsignedEventDraft::new(
        1,
        1_625,
        vec![vec!["a".to_owned(), coordinate.to_address()]],
        control.content().to_owned(),
    )
    .map_err(|error| format!("control event: {error:?}"))?
    .prepare(controller_device)
    .map_err(|error| format!("control preimage: {error:?}"))?;
    let signed_control = sign(&prepared_control, &controller_keys)?;
    let control_id = VerifiedNip01Event::verify(signed_control.clone())?.event_id();

    let mut document = AuthoringDocument::empty(ActorState::initial(actor, BTreeSet::new()))
        .map_err(|error| format!("authoring document: {error:?}"))?;
    let change = document
        .author_change(&[Operation::PutString {
            key: "title".to_owned(),
            value: "Trusted engine".to_owned(),
        }])
        .map_err(|error| format!("author change: {error:?}"))?;
    let prepared_change = UnsignedEventDraft::new(
        2,
        1_624,
        vec![
            vec!["a".to_owned(), coordinate.to_address()],
            vec!["e".to_owned(), control_id.to_hex()],
            vec!["x".to_owned(), change.change_hash().to_hex()],
        ],
        base64::engine::general_purpose::STANDARD.encode(change.raw()),
    )
    .map_err(|error| format!("change event: {error:?}"))?
    .prepare(device)
    .map_err(|error| format!("change preimage: {error:?}"))?;
    let signed_change = sign(&prepared_change, &device_keys)?;

    let mut builder = CorpusBuilder::new();
    assert!(matches!(
        builder.ingest(signed_change),
        IngestOutcome::Accepted { .. }
    ));
    assert!(matches!(
        builder.ingest(signed_control),
        IngestOutcome::Accepted { .. }
    ));
    let corpus = builder.finish();
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate(
        &corpus,
        coordinate,
        &mut WorkBudget::new(1_000_000, 10_000),
        &NeverCancelled,
    );

    assert_eq!(report.completion(), Completion::Complete);
    assert_eq!(report.accepted_changes(), [change.change_hash()]);
    assert_eq!(report.heads(), [change.change_hash()]);
    assert!(report.document().is_some());
    Ok(())
}
