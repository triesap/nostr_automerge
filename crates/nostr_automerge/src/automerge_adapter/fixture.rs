use automerge::{ActorId, Automerge, ROOT, TextEncoding, transaction::Transactable};
use sha2::{Digest, Sha256};

use crate::ProtocolRevision;

use super::framing::validate_change_frame;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FixtureError {
    Operation,
    MissingChange,
}

pub(crate) fn generate_change(actor_bytes: [u8; 32]) -> Option<Vec<u8>> {
    generate_change_inner(actor_bytes).ok()
}

fn generate_change_inner(actor_bytes: [u8; 32]) -> Result<Vec<u8>, FixtureError> {
    let actor = ActorId::from(actor_bytes);
    let mut document = Automerge::new_with_encoding(TextEncoding::Utf16CodeUnit);
    document.set_actor(actor);
    {
        let mut transaction = document.transaction();
        transaction
            .put(ROOT, "key", "value")
            .map_err(|_| FixtureError::Operation)?;
        transaction.commit();
    }
    let changes = document.get_changes(&[]);
    match changes.as_slice() {
        [change] => Ok(change.raw_bytes().to_vec()),
        _ => Err(FixtureError::MissingChange),
    }
}

fn decode_hex(value: &str) -> Option<Vec<u8>> {
    let bytes = value.trim().as_bytes();
    if !bytes.len().is_multiple_of(2) {
        return None;
    }
    bytes
        .chunks_exact(2)
        .map(|pair| {
            let text = core::str::from_utf8(pair).ok()?;
            u8::from_str_radix(text, 16).ok()
        })
        .collect()
}

#[test]
fn generate_one_canonical_uncompressed_change() {
    let generated = generate_change_inner([0x42; 32]);
    let committed = decode_hex(include_str!(
        "../../../../fixtures/v1_draft/automerge_changes/basic/change.hex"
    ));
    assert_eq!(generated.as_ref().ok(), committed.as_ref());

    let change = committed
        .as_deref()
        .and_then(|bytes| automerge::Change::try_from(bytes).ok());
    assert!(change.is_some());
    let change = match change {
        Some(change) => change,
        None => return,
    };
    assert_eq!(change.actor_id(), &ActorId::from([0x42; 32]));
    assert_eq!(change.seq(), 1);
    assert_eq!(change.start_op().get(), 1);
    assert_eq!(change.timestamp(), 0);
    assert_eq!(change.message(), None);
    assert!(change.extra_bytes().is_empty());
    assert_eq!(change.len(), 1);
    assert_eq!(
        change.hash().to_string(),
        "692468c4bba75b52de637c5ea3917ae1a54cf77286670282b7eff070db4531c5"
    );

    let bytes = match committed.as_deref() {
        Some(bytes) => bytes,
        None => return,
    };
    let framed = validate_change_frame(bytes, ProtocolRevision::draft_v1());
    assert_eq!(
        framed.map(|value| value.change_hash.to_hex()),
        Ok(change.hash().to_string())
    );
    assert_eq!(
        format!("{:x}", Sha256::digest(bytes)),
        "5fd4d9f24f784442abb8a90c550aeaf32a74b44fbd24e9566e348856e7a29ac2"
    );

    let mut applied = Automerge::new_with_encoding(TextEncoding::Utf16CodeUnit);
    assert!(applied.apply_changes([change]).is_ok());
    assert_eq!(applied.get_heads().len(), 1);

    let metadata: Result<serde_json::Value, _> = serde_json::from_str(include_str!(
        "../../../../fixtures/v1_draft/automerge_changes/basic/metadata.json"
    ));
    assert!(metadata.is_ok());
    assert_eq!(
        metadata
            .as_ref()
            .ok()
            .and_then(|value| value.get("sha256"))
            .and_then(serde_json::Value::as_str),
        Some("5fd4d9f24f784442abb8a90c550aeaf32a74b44fbd24e9566e348856e7a29ac2")
    );
}
