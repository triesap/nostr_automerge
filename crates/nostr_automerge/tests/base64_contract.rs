//! Executable signed-Event coverage for the canonical base64 contract.

mod support;

use std::collections::BTreeSet;

use base64::Engine as _;
use nostr_automerge::authoring::{ActorState, AuthoringDocument, Operation, UnsignedEventDraft};
use nostr_automerge::{ActorId, CorpusBuilder, DocumentCoordinate, IngestOutcome};
use support::test_signer::TestSigner;

#[test]
#[allow(clippy::expect_used)]
fn signed_change_events_reject_every_noncanonical_base64_class() {
    let controller = TestSigner::from_byte(71);
    let device = TestSigner::from_byte(72);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "b6".repeat(32)
    )
    .parse()
    .expect("fixed coordinate");
    let actor = ActorId::derive(coordinate, device.public_key());
    let mut document = AuthoringDocument::empty(ActorState::initial(actor, BTreeSet::new()))
        .expect("empty authoring document");
    let change = document
        .author_change(&[Operation::PutString {
            key: "base64".to_owned(),
            value: "canonical".to_owned(),
        }])
        .expect("canonical authored change");
    let canonical_content = base64::engine::general_purpose::STANDARD.encode(change.raw());
    let tags = vec![
        vec!["a".to_owned(), coordinate.to_address()],
        vec!["e".to_owned(), "44".repeat(32)],
        vec!["x".to_owned(), change.change_hash().to_hex()],
    ];
    let sign = |created_at: u64, content: String| {
        let prepared = UnsignedEventDraft::new(created_at, 1_624, tags.clone(), content)
            .expect("valid draft")
            .prepare(device.public_key())
            .expect("canonical preimage");
        device.sign(&prepared)
    };

    let mut builder = CorpusBuilder::new();
    assert!(matches!(
        builder.ingest(sign(1, canonical_content)),
        IngestOutcome::Accepted { .. }
    ));
    for (offset, invalid_content) in ["AAE", "AAE=\n", "AAE_", "AB==", "not-base64", "éé"]
        .into_iter()
        .enumerate()
    {
        assert!(matches!(
            builder.ingest(sign(
                2 + u64::try_from(offset).expect("bounded vector index"),
                invalid_content.to_owned()
            )),
            IngestOutcome::InvalidCarrier { diagnostic, .. }
                if diagnostic.as_str() == "base64.noncanonical"
        ));
    }
}
