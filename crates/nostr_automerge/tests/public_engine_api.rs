//! Downstream-only compile and behavior checks for the public engine surface.

mod support;

use std::collections::BTreeSet;

use base64::Engine as _;
use nostr_automerge::authoring::{ActorState, AuthoringDocument, Operation, UnsignedEventDraft};
use nostr_automerge::{
    ActorId, Completion, CorpusBuilder, DocumentCoordinate, EvidenceCorpus, EvidenceStatus,
    IngestOutcome, NeverCancelled, ProtocolRevision, ReferenceEvaluator, VerifiedNip01Event,
    WorkBudget,
};
use support::test_signer::TestSigner;

#[test]
fn build_immutable_evidence_corpus_through_public_api() {
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
    let corpus: EvidenceCorpus = builder.finish();
    assert_eq!(corpus.event_count(), 1);
    assert_eq!(corpus.invalid_count(), 1);
    assert_eq!(corpus.duplicate_count(), 1);
    assert!(!corpus.is_empty());
    assert_eq!(
        corpus
            .records()
            .map(|record| record.status())
            .collect::<Vec<_>>(),
        vec![
            EvidenceStatus::Irrelevant,
            EvidenceStatus::Invalid,
            EvidenceStatus::Duplicate,
        ]
    );
    assert!(!include_str!("../src/lib.rs").contains("IndexValidity"));
    assert_eq!(corpus.control_ids().count(), 0);
    assert_eq!(corpus.change_hashes().count(), 0);
}

#[test]
fn reference_evaluator_api_is_sealed_and_repository_owned() {
    let evaluator = ReferenceEvaluator::new(ProtocolRevision::draft_v1());
    assert_eq!(evaluator.revision(), ProtocolRevision::draft_v1());
    assert!(std::any::type_name::<ReferenceEvaluator>().starts_with("nostr_automerge::"));
}

#[test]
#[allow(clippy::expect_used)]
fn signed_events_reach_materialized_state_through_public_engine() {
    let controller = TestSigner::from_byte(20);
    let device = TestSigner::from_byte(21);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "42".repeat(32)
    )
    .parse()
    .expect("fixed coordinate");
    let actor = ActorId::derive(coordinate, device.public_key());
    let mut document = AuthoringDocument::empty(ActorState::initial(actor, BTreeSet::new()))
        .expect("empty authoring document");
    let change = document
        .author_change(&[Operation::PutString {
            key: "title".to_owned(),
            value: "trusted".to_owned(),
        }])
        .expect("canonical authored change");
    let control_content = format!(
        r#"{{"base_heads":[],"format":"automerge-change-v1","members":[{{"account":null,"pubkey":"{}","roles":["checkpoint","write"]}}],"policy":"controller-acl-v1","predecessor":null,"seq":0,"successor":null,"text_encoding":"utf16","v":1}}"#,
        device.public_key().to_hex()
    );
    let control = controller.sign(
        &UnsignedEventDraft::new(
            1,
            1_625,
            vec![vec!["a".to_owned(), coordinate.to_address()]],
            control_content,
        )
        .expect("control draft")
        .prepare(controller.public_key())
        .expect("control preimage"),
    );
    let control_id = VerifiedNip01Event::verify(control.clone())
        .expect("signed control")
        .event_id();
    let change_event = device.sign(
        &UnsignedEventDraft::new(
            2,
            1_624,
            vec![
                vec!["a".to_owned(), coordinate.to_address()],
                vec!["e".to_owned(), control_id.to_hex()],
                vec!["x".to_owned(), change.change_hash().to_hex()],
            ],
            base64::engine::general_purpose::STANDARD.encode(change.raw()),
        )
        .expect("change draft")
        .prepare(device.public_key())
        .expect("change preimage"),
    );

    let mut builder = CorpusBuilder::new();
    assert!(matches!(
        builder.ingest(change_event),
        IngestOutcome::Accepted { .. }
    ));
    assert!(matches!(
        builder.ingest(control),
        IngestOutcome::Accepted { .. }
    ));
    let corpus = builder.finish();
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate(
        &corpus,
        coordinate,
        &mut WorkBudget::new(1_000_000, 1_000),
        &NeverCancelled,
    );

    assert_eq!(report.completion(), Completion::Complete);
    assert_eq!(report.canonical_controls(), [control_id]);
    assert_eq!(report.accepted_changes(), [change.change_hash()]);
    assert_eq!(report.heads(), [change.change_hash()]);
    assert!(report.document().is_some_and(|view| !view.is_empty()));
}

#[test]
#[allow(clippy::expect_used)]
fn signed_manifest_selection_validates_latest_without_fallback_or_authority() {
    let signer = TestSigner::from_byte(3);
    let document_id = "33".repeat(32);
    let control = "11".repeat(32);
    let content = format!(
        r#"{{"application":null,"checkpoint":null,"control":"{control}","description":null,"format":"automerge-change-v1","name":null,"relays":["wss://relay.example"],"status":"active","successor":null,"text_encoding":"utf16","v":1}}"#
    );
    let sign = |created_at: u64, tags: Vec<Vec<String>>, content: String| {
        let prepared = UnsignedEventDraft::new(created_at, 31_624, tags, content)
            .expect("valid draft")
            .prepare(signer.public_key())
            .expect("canonical preimage");
        signer.sign(&prepared)
    };

    let mut builder = CorpusBuilder::new();
    assert!(matches!(
        builder.ingest(sign(
            1,
            vec![vec!["d".to_owned(), document_id.clone()]],
            content.clone()
        )),
        IngestOutcome::Accepted { .. }
    ));
    assert!(matches!(
        builder.ingest(sign(
            0,
            vec![vec!["d".to_owned(), document_id.clone()]],
            content.replace("active", "paused")
        )),
        IngestOutcome::InvalidCarrier { diagnostic, .. }
            if diagnostic.as_str() == "manifest.semantics"
    ));
    assert!(matches!(
        builder.ingest(sign(0, vec![], content.clone())),
        IngestOutcome::InvalidCarrier { diagnostic, .. }
            if diagnostic.as_str() == "tag.required"
    ));
    assert!(matches!(
        builder.ingest(sign(
            0,
            vec![vec!["d".to_owned(), document_id.clone()]],
            content.replacen("{\"application\":null,\"checkpoint\":null", "{\"checkpoint\":null,\"application\":null", 1)
        )),
        IngestOutcome::InvalidCarrier { diagnostic, .. }
            if diagnostic.as_str() == "jcs.noncanonical"
    ));
    assert!(matches!(
        builder.ingest(sign(
            0,
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

    let coordinate = hints[0].coordinate();
    let mut invalid_latest = CorpusBuilder::new();
    assert!(matches!(
        invalid_latest.ingest(sign(
            1,
            vec![vec!["d".to_owned(), coordinate.document_id().to_hex()]],
            content.clone()
        )),
        IngestOutcome::Accepted { .. }
    ));
    let invalid = sign(
        2,
        vec![vec!["d".to_owned(), coordinate.document_id().to_hex()]],
        content.replace("active", "paused"),
    );
    let invalid_id = VerifiedNip01Event::verify(invalid.clone())
        .expect("signed invalid latest")
        .event_id();
    assert!(matches!(
        invalid_latest.ingest(invalid),
        IngestOutcome::InvalidCarrier { .. }
    ));
    let invalid_latest = invalid_latest.finish();
    assert!(matches!(
        invalid_latest.selected_manifest(coordinate),
        nostr_automerge::ManifestAvailability::Unavailable { event_id, diagnostic }
            if event_id == invalid_id && diagnostic.as_str() == "manifest.semantics"
    ));
    assert_eq!(invalid_latest.manifest_hints().count(), 0);

    let tie_left = sign(
        3,
        vec![vec!["d".to_owned(), coordinate.document_id().to_hex()]],
        content.clone(),
    );
    let tie_right = sign(
        3,
        vec![vec!["d".to_owned(), coordinate.document_id().to_hex()]],
        content.replace(&control, &"22".repeat(32)),
    );
    let tie_left_id = VerifiedNip01Event::verify(tie_left.clone())
        .expect("signed tie left")
        .event_id();
    let tie_right_id = VerifiedNip01Event::verify(tie_right.clone())
        .expect("signed tie right")
        .event_id();
    let expected = tie_left_id.min(tie_right_id);
    let mut tied = CorpusBuilder::new();
    assert!(matches!(
        tied.ingest(tie_right),
        IngestOutcome::Accepted { .. }
    ));
    assert!(matches!(
        tied.ingest(tie_left),
        IngestOutcome::Accepted { .. }
    ));
    assert!(matches!(
        tied.finish().selected_manifest(coordinate),
        nostr_automerge::ManifestAvailability::Available(hints)
            if hints.event_id() == expected
    ));
}

#[test]
#[allow(clippy::expect_used)]
fn pending_controls_converge_after_signed_parent_delivery() {
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

    let genesis = sign(1, vec![a(&coordinate)], content(0));
    let genesis_id = VerifiedNip01Event::verify(genesis.clone())
        .expect("signed genesis")
        .event_id();
    let child = sign(
        2,
        vec![a(&coordinate), vec!["e".to_owned(), genesis_id.to_hex()]],
        content(1),
    );
    let child_id = VerifiedNip01Event::verify(child.clone())
        .expect("signed child")
        .event_id();

    let mut pending = CorpusBuilder::new();
    assert!(matches!(
        pending.ingest(child.clone()),
        IngestOutcome::Accepted { .. }
    ));
    assert_eq!(
        pending.finish().pending_control_ids().collect::<Vec<_>>(),
        vec![child_id]
    );

    let mut builder = CorpusBuilder::new();
    assert!(matches!(
        builder.ingest(child),
        IngestOutcome::Accepted { .. }
    ));
    assert!(matches!(
        builder.ingest(genesis),
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
    assert_eq!(builder.finish().pending_control_ids().count(), 0);
}

#[test]
#[allow(clippy::expect_used)]
fn signed_change_ingest_requires_canonical_actor_hash_control_and_bytes() {
    let controller = TestSigner::from_byte(8);
    let device = TestSigner::from_byte(9);
    let wrong_device = TestSigner::from_byte(10);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "22".repeat(32)
    )
    .parse()
    .expect("fixed coordinate");
    let actor = ActorId::derive(coordinate, device.public_key());
    let mut document = AuthoringDocument::empty(ActorState::initial(actor, BTreeSet::new()))
        .expect("empty authoring document");
    let change = document
        .author_change(&[Operation::PutString {
            key: "key".to_owned(),
            value: "value".to_owned(),
        }])
        .expect("canonical authored change");
    let content = base64::engine::general_purpose::STANDARD.encode(change.raw());
    let tags = vec![
        vec!["a".to_owned(), coordinate.to_address()],
        vec!["e".to_owned(), "44".repeat(32)],
        vec!["x".to_owned(), change.change_hash().to_hex()],
    ];
    let sign = |signer: &TestSigner, created_at: u64, tags: Vec<Vec<String>>, content: String| {
        let prepared = UnsignedEventDraft::new(created_at, 1_624, tags, content)
            .expect("valid draft")
            .prepare(signer.public_key())
            .expect("canonical preimage");
        signer.sign(&prepared)
    };

    let mut builder = CorpusBuilder::new();
    assert!(matches!(
        builder.ingest(sign(&device, 1, tags.clone(), content.clone())),
        IngestOutcome::Accepted { .. }
    ));
    let mut wrong_hash = tags.clone();
    wrong_hash[2][1] = "00".repeat(32);
    assert!(matches!(
        builder.ingest(sign(&device, 2, wrong_hash, content.clone())),
        IngestOutcome::InvalidCarrier { diagnostic, .. }
            if diagnostic.as_str() == "change.hash"
    ));
    assert!(matches!(
        builder.ingest(sign(&wrong_device, 3, tags.clone(), content.clone())),
        IngestOutcome::InvalidCarrier { diagnostic, .. }
            if diagnostic.as_str() == "change.actor"
    ));
    let mut wrong_control = tags.clone();
    wrong_control[1][1] = "not-an-event-id".to_owned();
    assert!(matches!(
        builder.ingest(sign(&device, 4, wrong_control, content.clone())),
        IngestOutcome::InvalidCarrier { diagnostic, .. }
            if diagnostic.as_str() == "tag.required"
    ));
    let mut wrong_coordinate = tags.clone();
    wrong_coordinate[0][1] = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "23".repeat(32)
    );
    assert!(matches!(
        builder.ingest(sign(&device, 5, wrong_coordinate, content.clone())),
        IngestOutcome::InvalidCarrier { diagnostic, .. }
            if diagnostic.as_str() == "change.actor"
    ));
    assert!(matches!(
        builder.ingest(sign(&device, 6, tags.clone(), "not-base64".to_owned())),
        IngestOutcome::InvalidCarrier { diagnostic, .. }
            if diagnostic.as_str() == "base64.noncanonical"
    ));
    let forbidden =
        include_str!("../../../fixtures/v1_draft/automerge_framing/compressed_change_chunk.hex")
            .trim()
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| {
                u8::from_str_radix(std::str::from_utf8(pair).expect("hex pair"), 16)
                    .expect("fixture hex")
            })
            .collect::<Vec<_>>();
    assert!(matches!(
        builder.ingest(sign(
            &device,
            7,
            tags,
            base64::engine::general_purpose::STANDARD.encode(forbidden)
        )),
        IngestOutcome::InvalidCarrier { diagnostic, .. }
            if diagnostic.as_str() == "automerge.chunk_type"
    ));
}
