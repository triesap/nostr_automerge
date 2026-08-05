//! Downstream-only compile and behavior checks for the public engine surface.

mod support;

use nostr_automerge::authoring::UnsignedEventDraft;
use nostr_automerge::{CorpusBuilder, IngestOutcome};
use support::test_signer::TestSigner;

#[test]
fn build_immutable_ingress_corpus_through_public_api() {
    let valid = include_bytes!("../../../fixtures/v1_draft/nip01/valid_event.json");

    let mut builder = CorpusBuilder::new();
    assert!(matches!(
        builder.ingest_bytes(valid),
        IngestOutcome::Irrelevant { .. }
    ));
    assert!(matches!(
        builder.ingest_bytes(valid),
        IngestOutcome::Duplicate { .. }
    ));
    assert!(matches!(
        builder.ingest_bytes(b"{}"),
        IngestOutcome::Invalid { diagnostic }
            if diagnostic.as_str() == "nip01.shape"
    ));
    assert!(matches!(
        builder.ingest_bytes(&[0xff]),
        IngestOutcome::Invalid { diagnostic }
            if diagnostic.as_str() == "raw.invalid_utf8"
    ));
    let oversized = vec![b' '; 262_145];
    assert!(matches!(
        builder.ingest_bytes(&oversized),
        IngestOutcome::Invalid { diagnostic }
            if diagnostic.as_str() == "raw.too_large"
    ));
    let corpus = builder.finish();
    assert_eq!(corpus.event_count(), 1);
    assert_eq!(corpus.invalid_count(), 1);
    assert_eq!(corpus.duplicate_count(), 1);
    assert!(!corpus.is_empty());
}

#[test]
#[allow(clippy::expect_used)]
fn signed_manifest_carrier_validation_exposes_only_advisory_hints() {
    let signer = TestSigner::from_byte(3);
    let document_id = "33".repeat(32);
    let control = "11".repeat(32);
    let content = format!(
        r#"{{"application":null,"checkpoint":null,"control":"{control}","description":null,"format":"automerge-change-v1","name":null,"relays":["wss://relay.example"],"status":"active","successor":null,"text_encoding":"utf16","v":1}}"#
    );
    let sign = |tags: Vec<Vec<String>>, content: String| {
        let prepared = UnsignedEventDraft::new(1, 31_624, tags, content)
            .expect("valid draft")
            .prepare(signer.public_key())
            .expect("canonical preimage");
        signer.sign(&prepared)
    };

    let mut builder = CorpusBuilder::new();
    assert!(matches!(
        builder.ingest(sign(
            vec![vec!["d".to_owned(), document_id.clone()]],
            content.clone()
        )),
        IngestOutcome::Accepted { .. }
    ));
    assert!(matches!(
        builder.ingest(sign(
            vec![vec!["d".to_owned(), document_id.clone()]],
            content.replace("active", "paused")
        )),
        IngestOutcome::InvalidCarrier { diagnostic, .. }
            if diagnostic.as_str() == "manifest.semantics"
    ));
    assert!(matches!(
        builder.ingest(sign(vec![], content.clone())),
        IngestOutcome::InvalidCarrier { diagnostic, .. }
            if diagnostic.as_str() == "tag.required"
    ));
    assert!(matches!(
        builder.ingest(sign(
            vec![vec!["d".to_owned(), document_id.clone()]],
            content.replacen("{\"application\":null,\"checkpoint\":null", "{\"checkpoint\":null,\"application\":null", 1)
        )),
        IngestOutcome::InvalidCarrier { diagnostic, .. }
            if diagnostic.as_str() == "jcs.noncanonical"
    ));
    assert!(matches!(
        builder.ingest(sign(
            vec![vec!["d".to_owned(), document_id]],
            content.replace("\"v\":1", "\"v\":2")
        )),
        IngestOutcome::UnsupportedRevision { diagnostic, .. }
            if diagnostic.as_str() == "carrier.revision"
    ));

    let corpus = builder.finish();
    let hints = corpus.manifest_hints().collect::<Vec<_>>();
    assert_eq!(hints.len(), 1);
    assert_eq!(hints[0].control().to_hex(), control);
    assert_eq!(hints[0].relays(), ["wss://relay.example"]);
    assert!(hints[0].checkpoint().is_none());
    assert_eq!(
        hints[0].coordinate().controller().as_bytes(),
        signer.public_key().as_bytes()
    );
}

#[test]
#[allow(clippy::expect_used)]
fn signed_control_carrier_tags_bind_controller_coordinate_and_parent() {
    let controller = TestSigner::from_byte(5);
    let other_controller = TestSigner::from_byte(6);
    let device = TestSigner::from_byte(7);
    let document_id = "22".repeat(32);
    let coordinate = format!("31624:{}:{document_id}", controller.public_key().to_hex());
    let other_coordinate = format!(
        "31624:{}:{document_id}",
        other_controller.public_key().to_hex()
    );
    let content = |sequence: u64| {
        format!(
            r#"{{"base_heads":[],"format":"automerge-change-v1","members":[{{"account":null,"pubkey":"{}","roles":["checkpoint","write"]}}],"policy":"controller-acl-v1","predecessor":null,"seq":{sequence},"successor":null,"text_encoding":"utf16","v":1}}"#,
            device.public_key().to_hex()
        )
    };
    let sign = |created_at: u64, tags: Vec<Vec<String>>, content: String| {
        let prepared = UnsignedEventDraft::new(created_at, 1_625, tags, content)
            .expect("valid draft")
            .prepare(controller.public_key())
            .expect("canonical preimage");
        controller.sign(&prepared)
    };
    let a = |value: &str| vec!["a".to_owned(), value.to_owned()];
    let e = || vec!["e".to_owned(), "44".repeat(32)];

    let mut builder = CorpusBuilder::new();
    assert!(matches!(
        builder.ingest(sign(1, vec![a(&coordinate)], content(0))),
        IngestOutcome::Accepted { .. }
    ));
    assert!(matches!(
        builder.ingest(sign(2, vec![a(&coordinate), e()], content(1))),
        IngestOutcome::Accepted { .. }
    ));
    for (created_at, tags, sequence, expected) in [
        (3, vec![], 0, "tag.required"),
        (4, vec![a(&coordinate), a(&coordinate)], 0, "tag.required"),
        (
            5,
            vec![a(&coordinate), vec!["p".to_owned(), "extra".to_owned()]],
            0,
            "tag.forbidden",
        ),
        (6, vec![a(&coordinate), e()], 0, "tag.forbidden"),
        (7, vec![a(&coordinate)], 1, "tag.required"),
    ] {
        assert!(matches!(
            builder.ingest(sign(created_at, tags, content(sequence))),
            IngestOutcome::InvalidCarrier { diagnostic, .. }
                if diagnostic.as_str() == expected
        ));
    }
    assert!(matches!(
        builder.ingest(sign(8, vec![a(&other_coordinate)], content(0))),
        IngestOutcome::InvalidCarrier { diagnostic, .. }
            if diagnostic.as_str() == "carrier.coordinate"
    ));
    assert!(matches!(
        builder.ingest(sign(
            9,
            vec![a(&coordinate)],
            content(0).replacen(
                "{\"base_heads\":[],\"format\":\"automerge-change-v1\"",
                "{\"format\":\"automerge-change-v1\",\"base_heads\":[]",
                1
            )
        )),
        IngestOutcome::InvalidCarrier { diagnostic, .. }
            if diagnostic.as_str() == "jcs.noncanonical"
    ));
    assert!(matches!(
        builder.ingest(sign(
            10,
            vec![a(&coordinate)],
            content(0).replace("\"v\":1", "\"v\":2")
        )),
        IngestOutcome::UnsupportedRevision { .. }
    ));
}
