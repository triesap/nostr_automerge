//! Pure in-memory authoring; persistence, signing, and publication stay caller-owned.

use std::collections::BTreeSet;

use nostr_automerge::authoring::{
    ActorState, AuthoringDocument, ControlDraft, ControlGrant, ControlRole, Operation,
    UnsignedEventDraft,
};
use nostr_automerge::{
    ActorId, ControllerPublicKey, DevicePublicKey, DocumentCoordinate, DocumentId,
    ProtocolRevision, RawEventBytes, VerifiedNip01Event,
};
use secp256k1::{Keypair, Secp256k1, SecretKey};

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let secret = SecretKey::from_byte_array([1; 32])?;
    let keys = Keypair::from_secret_key(&Secp256k1::new(), &secret);
    let (x_only, _) = keys.x_only_public_key();
    let device = DevicePublicKey::from_bytes(x_only.serialize());
    let coordinate = DocumentCoordinate::new(
        ControllerPublicKey::from_bytes(x_only.serialize()),
        DocumentId::from_bytes([2; 32]),
    );
    let actor = ActorId::derive(coordinate, device);

    let control = ControlDraft::new(
        0,
        BTreeSet::new(),
        vec![ControlGrant {
            account: None,
            device,
            roles: BTreeSet::from([ControlRole::Write]),
        }],
        None,
        None,
    )
    .map_err(|error| format!("control draft: {error:?}"))?;
    let prepared = UnsignedEventDraft::new(
        1,
        1_625,
        vec![vec!["a".to_owned(), coordinate.to_address()]],
        control.content().to_owned(),
    )
    .map_err(|error| format!("event draft: {error:?}"))?
    .prepare(device)
    .map_err(|error| format!("event preimage: {error:?}"))?;

    // This signing block represents caller infrastructure. Production code
    // should hand the event ID to its own key service and durable outbox.
    let signature =
        Secp256k1::new().sign_schnorr_no_aux_rand(prepared.event_id().as_bytes(), &keys);
    let raw_json = serde_json::to_vec(&serde_json::json!({
        "id": prepared.event_id().to_hex(),
        "pubkey": prepared.public_key().to_hex(),
        "created_at": prepared.created_at(),
        "kind": prepared.kind(),
        "tags": prepared.tags(),
        "content": prepared.content(),
        "sig": signature.to_string(),
    }))?;
    let raw = RawEventBytes::new(&raw_json, ProtocolRevision::draft_v1())?;
    let _reingested = VerifiedNip01Event::verify(raw)?;

    let mut document = AuthoringDocument::empty(ActorState::initial(actor, BTreeSet::new()))
        .map_err(|error| format!("authoring document: {error:?}"))?;
    let authored = document
        .author_change(&[Operation::PutString {
            key: "title".to_owned(),
            value: "Offline draft".to_owned(),
        }])
        .map_err(|error| format!("author change: {error:?}"))?;
    assert_eq!(document.actor_state(), authored.new_state());
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run()
}

#[cfg(test)]
mod tests {
    #[test]
    fn add_pure_authoring_examples() {
        assert!(super::run().is_ok());
    }
}
