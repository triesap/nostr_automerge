//! Downstream-only compile and behavior checks for the public engine surface.

mod support;

use std::cell::Cell;
use std::collections::BTreeSet;

use base64::Engine as _;
use nostr_automerge::authoring::{ActorState, AuthoringDocument, Operation, UnsignedEventDraft};
use nostr_automerge::{
    ActorId, ChangeHash, Completion, CorpusBuilder, DocumentCoordinate, EvaluationFailure, EventId,
    EvidenceCorpus, EvidenceStatus, IngestOutcome, NeverCancelled, ProtocolDisposition,
    ProtocolRevision, RawEventBytes, ReferenceEvaluator, VerifiedNip01Event, WorkBudget,
    WorkCounter,
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
    let public_root = include_str!("../src/lib.rs");
    assert!(!public_root.contains("BatchControl"));
    assert!(!public_root.contains("BatchChange"));
    assert!(!public_root.contains("OpaqueDocumentView"));
    assert!(!public_root.contains("IndexValidity"));
}

#[test]
#[allow(clippy::expect_used)]
fn signed_events_reach_materialized_state_through_public_engine() {
    let scenario = signed_engine_scenario();
    let mut builder = CorpusBuilder::new();
    assert!(matches!(
        builder.ingest(scenario.change.clone()),
        IngestOutcome::Accepted { .. }
    ));
    assert!(matches!(
        builder.ingest(scenario.control.clone()),
        IngestOutcome::Accepted { .. }
    ));
    let corpus = builder.finish();
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate(
        &corpus,
        scenario.coordinate,
        &mut WorkBudget::new(1_000_000, 1_000),
        &NeverCancelled,
    );

    assert_eq!(report.completion(), Completion::Complete);
    assert_eq!(report.canonical_controls(), [scenario.control_id]);
    assert_eq!(report.accepted_changes(), [scenario.change_hash]);
    assert_eq!(report.heads(), [scenario.change_hash]);
    assert!(report.document().is_some_and(|view| !view.is_empty()));
}

#[test]
#[allow(clippy::expect_used)]
fn signed_empty_terminal_genesis_materializes_empty_state() {
    let controller = TestSigner::from_byte(22);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "24".repeat(32)
    )
    .parse()
    .expect("fixed coordinate");
    let content = r#"{"base_heads":[],"format":"automerge-change-v1","members":[],"policy":"controller-acl-v1","predecessor":null,"seq":0,"successor":null,"text_encoding":"utf16","v":1}"#;
    let control = controller.sign(
        &UnsignedEventDraft::new(
            1,
            1_625,
            vec![vec!["a".to_owned(), coordinate.to_address()]],
            content.to_owned(),
        )
        .expect("control draft")
        .prepare(controller.public_key())
        .expect("control preimage"),
    );
    let control_id = VerifiedNip01Event::verify(control.clone())
        .expect("signed control")
        .event_id();
    let mut builder = CorpusBuilder::new();
    assert!(matches!(
        builder.ingest(control),
        IngestOutcome::Accepted { event_id } if event_id == control_id
    ));
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate(
        &builder.finish(),
        coordinate,
        &mut WorkBudget::new(1_000_000, 1_000),
        &NeverCancelled,
    );
    assert_eq!(report.completion(), Completion::Complete);
    assert_eq!(report.canonical_controls(), [control_id]);
    assert!(report.accepted_changes().is_empty());
    assert!(report.heads().is_empty());
    assert!(
        report
            .document()
            .is_some_and(|document| !document.is_empty())
    );
}

#[test]
#[allow(clippy::expect_used)]
fn signed_terminal_genesis_rejects_children() {
    let controller = TestSigner::from_byte(23);
    let device = TestSigner::from_byte(24);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "25".repeat(32)
    )
    .parse()
    .expect("fixed coordinate");
    let content = |sequence: u64| {
        format!(
            r#"{{"base_heads":[],"format":"automerge-change-v1","members":[],"policy":"controller-acl-v1","predecessor":null,"seq":{sequence},"successor":null,"text_encoding":"utf16","v":1}}"#
        )
    };
    let genesis = controller.sign(
        &UnsignedEventDraft::new(
            1,
            1_625,
            vec![vec!["a".to_owned(), coordinate.to_address()]],
            content(0),
        )
        .expect("genesis draft")
        .prepare(controller.public_key())
        .expect("genesis preimage"),
    );
    let genesis_id = VerifiedNip01Event::verify(genesis.clone())
        .expect("signed genesis")
        .event_id();
    let child = controller.sign(
        &UnsignedEventDraft::new(
            2,
            1_625,
            vec![
                vec!["a".to_owned(), coordinate.to_address()],
                vec!["e".to_owned(), genesis_id.to_hex()],
            ],
            content(1),
        )
        .expect("child draft")
        .prepare(controller.public_key())
        .expect("child preimage"),
    );
    let child_id = VerifiedNip01Event::verify(child.clone())
        .expect("signed child")
        .event_id();
    let actor = ActorId::derive(coordinate, device.public_key());
    let mut document = AuthoringDocument::empty(ActorState::initial(actor, BTreeSet::new()))
        .expect("empty authoring document");
    let authored = document
        .author_change(&[Operation::PutString {
            key: "forbidden".to_owned(),
            value: "extension".to_owned(),
        }])
        .expect("canonical authored change");
    let change_hash = authored.change_hash();
    let change = device.sign(
        &UnsignedEventDraft::new(
            3,
            1_624,
            vec![
                vec!["a".to_owned(), coordinate.to_address()],
                vec!["e".to_owned(), genesis_id.to_hex()],
                vec!["x".to_owned(), change_hash.to_hex()],
            ],
            base64::engine::general_purpose::STANDARD.encode(authored.raw()),
        )
        .expect("change draft")
        .prepare(device.public_key())
        .expect("change preimage"),
    );
    let mut builder = CorpusBuilder::new();
    assert!(matches!(
        builder.ingest(child),
        IngestOutcome::Accepted { event_id } if event_id == child_id
    ));
    assert!(matches!(
        builder.ingest(genesis),
        IngestOutcome::Accepted { event_id } if event_id == genesis_id
    ));
    assert!(matches!(
        builder.ingest(change),
        IngestOutcome::Accepted { .. }
    ));
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate(
        &builder.finish(),
        coordinate,
        &mut WorkBudget::new(1_000_000, 1_000),
        &NeverCancelled,
    );
    assert_eq!(report.completion(), Completion::Complete);
    assert_eq!(report.canonical_controls(), [genesis_id]);
    assert!(!report.canonical_controls().contains(&child_id));
    assert!(report.accepted_changes().is_empty());
    assert!(report.heads().is_empty());
    assert_eq!(
        report.dispositions(),
        [(change_hash, ProtocolDisposition::Excluded)]
    );
}

#[test]
fn duplicate_delayed_and_invalid_evidence_converges() {
    let scenario = signed_engine_scenario();
    let evaluate = |events: &[RawEventBytes]| {
        let mut builder = CorpusBuilder::new();
        for event in events {
            let _ = builder.ingest(event.clone());
        }
        ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate(
            &builder.finish(),
            scenario.coordinate,
            &mut WorkBudget::new(1_000_000, 1_000),
            &NeverCancelled,
        )
    };
    let ordered = evaluate(&[scenario.control.clone(), scenario.change.clone()]);
    let delayed = evaluate(&[scenario.change.clone(), scenario.control.clone()]);
    let duplicate = evaluate(&[
        scenario.change.clone(),
        scenario.change.clone(),
        scenario.control.clone(),
        scenario.control.clone(),
    ]);
    let mut invalid_first = CorpusBuilder::new();
    assert!(matches!(
        invalid_first.ingest_bytes(b"{}"),
        IngestOutcome::Invalid { .. }
    ));
    assert!(matches!(
        invalid_first.ingest(scenario.change.clone()),
        IngestOutcome::Accepted { .. }
    ));
    assert!(matches!(
        invalid_first.ingest(scenario.control.clone()),
        IngestOutcome::Accepted { .. }
    ));
    let invalid_first = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate(
        &invalid_first.finish(),
        scenario.coordinate,
        &mut WorkBudget::new(1_000_000, 1_000),
        &NeverCancelled,
    );

    for report in [&delayed, &duplicate, &invalid_first] {
        assert_eq!(report.canonical_controls(), ordered.canonical_controls());
        assert_eq!(report.dispositions(), ordered.dispositions());
        assert_eq!(report.accepted_changes(), ordered.accepted_changes());
        assert_eq!(report.heads(), ordered.heads());
        assert_eq!(report.history_digest(), ordered.history_digest());
        assert_eq!(report.dispositions_digest(), ordered.dispositions_digest());
        assert_eq!(
            report.document().map(|view| view.byte_len()),
            ordered.document().map(|view| view.byte_len())
        );
    }
}

#[test]
fn event_and_carrier_work_exhaustion_precedes_state() {
    let scenario = signed_engine_scenario();
    let mut builder = CorpusBuilder::new();
    assert!(matches!(
        builder.ingest(scenario.change),
        IngestOutcome::Accepted { .. }
    ));
    assert!(matches!(
        builder.ingest(scenario.control),
        IngestOutcome::Accepted { .. }
    ));
    let mut budget = WorkBudget::new(1_000_000, 3);
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate(
        &builder.finish(),
        scenario.coordinate,
        &mut budget,
        &NeverCancelled,
    );

    assert_eq!(report.completion(), Completion::BudgetExhausted);
    assert_eq!(report.failure(), Some(EvaluationFailure::BudgetExhausted));
    assert!(report.canonical_controls().is_empty());
    assert!(report.accepted_changes().is_empty());
    assert_eq!(budget.consumed().get(WorkCounter::Event), 2);
    assert_eq!(budget.consumed().get(WorkCounter::Carrier), 0);
}

#[test]
fn cancellation_before_control_evaluation_fabricates_no_state() {
    let scenario = signed_engine_scenario();
    let mut builder = CorpusBuilder::new();
    assert!(matches!(
        builder.ingest(scenario.change),
        IngestOutcome::Accepted { .. }
    ));
    assert!(matches!(
        builder.ingest(scenario.control),
        IngestOutcome::Accepted { .. }
    ));
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate(
        &builder.finish(),
        scenario.coordinate,
        &mut WorkBudget::new(1_000_000, 1_000),
        &|| true,
    );

    assert_eq!(report.completion(), Completion::Cancelled);
    assert_eq!(report.failure(), Some(EvaluationFailure::Cancelled));
    assert!(report.canonical_controls().is_empty());
    assert!(report.dispositions().is_empty());
    assert!(report.accepted_changes().is_empty());
    assert!(report.document().is_none());
}

#[test]
fn control_selection_and_transition_have_distinct_charges() {
    let scenario = signed_engine_scenario();
    let mut builder = CorpusBuilder::new();
    assert!(matches!(
        builder.ingest(scenario.change),
        IngestOutcome::Accepted { .. }
    ));
    assert!(matches!(
        builder.ingest(scenario.control),
        IngestOutcome::Accepted { .. }
    ));
    let mut budget = WorkBudget::new(1_000_000, 5);
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate(
        &builder.finish(),
        scenario.coordinate,
        &mut budget,
        &NeverCancelled,
    );

    assert_eq!(report.completion(), Completion::BudgetExhausted);
    assert_eq!(report.canonical_controls(), [scenario.control_id]);
    assert!(report.dispositions().is_empty());
    assert!(report.accepted_changes().is_empty());
    assert_eq!(budget.consumed().get(WorkCounter::Control), 1);
}

#[test]
fn automerge_decode_work_is_bounded_before_state() {
    let scenario = signed_engine_scenario();
    let mut builder = CorpusBuilder::new();
    assert!(matches!(
        builder.ingest(scenario.change),
        IngestOutcome::Accepted { .. }
    ));
    assert!(matches!(
        builder.ingest(scenario.control),
        IngestOutcome::Accepted { .. }
    ));
    let corpus = builder.finish();
    let evaluator = ReferenceEvaluator::new(ProtocolRevision::draft_v1());
    let mut measured = WorkBudget::new(1_000_000, 1_000);
    let measured_report =
        evaluator.evaluate(&corpus, scenario.coordinate, &mut measured, &NeverCancelled);
    assert_eq!(measured_report.completion(), Completion::Complete);
    let decode_bytes = measured.consumed().get(WorkCounter::DecodeByte);
    assert!(decode_bytes > 0);

    let mut exhausted = WorkBudget::new(decode_bytes - 1, 1_000);
    let report = evaluator.evaluate(
        &corpus,
        scenario.coordinate,
        &mut exhausted,
        &NeverCancelled,
    );
    assert_eq!(report.completion(), Completion::BudgetExhausted);
    assert!(report.canonical_controls().is_empty());
    assert!(report.accepted_changes().is_empty());
    assert_eq!(exhausted.consumed().get(WorkCounter::DecodeByte), 0);
}

#[test]
fn automerge_application_and_materialization_are_charged() {
    let scenario = signed_engine_scenario();
    let mut builder = CorpusBuilder::new();
    assert!(matches!(
        builder.ingest(scenario.change),
        IngestOutcome::Accepted { .. }
    ));
    assert!(matches!(
        builder.ingest(scenario.control),
        IngestOutcome::Accepted { .. }
    ));
    let corpus = builder.finish();
    let evaluator = ReferenceEvaluator::new(ProtocolRevision::draft_v1());
    let mut measured = WorkBudget::new(1_000_000, 1_000);
    let measured_report =
        evaluator.evaluate(&corpus, scenario.coordinate, &mut measured, &NeverCancelled);
    assert_eq!(measured_report.completion(), Completion::Complete);
    assert_eq!(measured.consumed().get(WorkCounter::ApplyChange), 2);
    let consumed_items = 1_000 - measured.remaining().1;

    let mut exhausted = WorkBudget::new(1_000_000, consumed_items - 1);
    let report = evaluator.evaluate(
        &corpus,
        scenario.coordinate,
        &mut exhausted,
        &NeverCancelled,
    );
    assert_eq!(report.completion(), Completion::BudgetExhausted);
    assert_eq!(report.accepted_changes(), [scenario.change_hash]);
    assert!(report.document().is_none());
    assert_eq!(exhausted.consumed().get(WorkCounter::ApplyChange), 1);
}

#[test]
fn accepted_empty_history_has_a_real_document_view() {
    let scenario = signed_engine_scenario();
    let mut builder = CorpusBuilder::new();
    assert!(matches!(
        builder.ingest(scenario.control),
        IngestOutcome::Accepted { .. }
    ));
    let mut budget = WorkBudget::new(1_000_000, 1_000);
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate(
        &builder.finish(),
        scenario.coordinate,
        &mut budget,
        &NeverCancelled,
    );

    assert_eq!(report.completion(), Completion::Complete);
    assert_eq!(report.canonical_controls(), [scenario.control_id]);
    assert!(report.accepted_changes().is_empty());
    assert!(report.heads().is_empty());
    assert!(report.document().is_some_and(|view| view.byte_len() > 0));
    assert_eq!(budget.consumed().get(WorkCounter::ApplyChange), 1);
}

#[test]
fn every_work_counter_has_exact_before_and_after_boundaries() {
    for counter in [
        WorkCounter::Event,
        WorkCounter::Carrier,
        WorkCounter::Control,
        WorkCounter::GraphNode,
        WorkCounter::GraphEdge,
        WorkCounter::DecodeByte,
        WorkCounter::ApplyChange,
        WorkCounter::CheckpointByte,
        WorkCounter::Assertion,
    ] {
        let byte_counter = matches!(
            counter,
            WorkCounter::DecodeByte | WorkCounter::CheckpointByte
        );
        let mut budget = if byte_counter {
            WorkBudget::new(2, 0)
        } else {
            WorkBudget::new(0, 2)
        };
        assert!(budget.charge(counter, 2).is_ok(), "{}", counter.as_str());
        let before_failure = budget;
        assert!(matches!(
            budget.charge(counter, 1),
            Err(error) if error.counter() == counter
        ));
        assert_eq!(budget, before_failure);
        assert_eq!(budget.consumed().get(counter), 2);
    }
}

#[test]
fn cancellation_is_safe_at_every_evaluator_boundary() {
    let scenario = signed_engine_scenario();
    let mut builder = CorpusBuilder::new();
    assert!(matches!(
        builder.ingest(scenario.change),
        IngestOutcome::Accepted { .. }
    ));
    assert!(matches!(
        builder.ingest(scenario.control),
        IngestOutcome::Accepted { .. }
    ));
    let corpus = builder.finish();
    let evaluator = ReferenceEvaluator::new(ProtocolRevision::draft_v1());
    let calls = Cell::new(0_usize);
    let complete = evaluator.evaluate(
        &corpus,
        scenario.coordinate,
        &mut WorkBudget::new(1_000_000, 1_000),
        &|| {
            calls.set(calls.get() + 1);
            false
        },
    );
    assert_eq!(complete.completion(), Completion::Complete);
    let boundary_count = calls.get();
    assert!(boundary_count > 0);

    for cancel_at in 0..boundary_count {
        let calls = Cell::new(0_usize);
        let report = evaluator.evaluate(
            &corpus,
            scenario.coordinate,
            &mut WorkBudget::new(1_000_000, 1_000),
            &|| {
                let boundary = calls.get();
                calls.set(boundary + 1);
                boundary == cancel_at
            },
        );
        assert_eq!(report.completion(), Completion::Cancelled, "{cancel_at}");
        assert_eq!(report.failure(), Some(EvaluationFailure::Cancelled));
        assert!(report.document().is_none());
        assert!(
            report
                .heads()
                .iter()
                .all(|head| report.accepted_changes().contains(head))
        );
    }
}

struct SignedEngineScenario {
    coordinate: DocumentCoordinate,
    control: RawEventBytes,
    change: RawEventBytes,
    control_id: EventId,
    change_hash: ChangeHash,
}

#[allow(clippy::expect_used)]
fn signed_engine_scenario() -> SignedEngineScenario {
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

    SignedEngineScenario {
        coordinate,
        control,
        change: change_event,
        control_id,
        change_hash: change.change_hash(),
    }
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
        builder.ingest(child.clone()),
        IngestOutcome::Accepted { .. }
    ));
    assert!(matches!(
        builder.ingest(genesis.clone()),
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
    let corpus = builder.finish();
    assert_eq!(corpus.pending_control_ids().count(), 0);
    let coordinate: DocumentCoordinate = coordinate.parse().expect("fixed coordinate");
    let evaluator = ReferenceEvaluator::new(ProtocolRevision::draft_v1());
    let report = evaluator.evaluate(
        &corpus,
        coordinate,
        &mut WorkBudget::new(1_000_000, 1_000),
        &NeverCancelled,
    );
    let mut ordered = CorpusBuilder::new();
    assert!(matches!(
        ordered.ingest(genesis),
        IngestOutcome::Accepted { .. }
    ));
    assert!(matches!(
        ordered.ingest(child),
        IngestOutcome::Accepted { .. }
    ));
    let ordered = evaluator.evaluate(
        &ordered.finish(),
        coordinate,
        &mut WorkBudget::new(1_000_000, 1_000),
        &NeverCancelled,
    );
    assert_eq!(report.completion(), Completion::Complete);
    assert_eq!(report.canonical_controls(), [genesis_id, child_id]);
    assert_eq!(report.canonical_controls(), ordered.canonical_controls());
    assert_eq!(report.dispositions(), ordered.dispositions());
    assert_eq!(report.accepted_changes(), ordered.accepted_changes());
    assert_eq!(report.heads(), ordered.heads());
    assert_eq!(report.history_digest(), ordered.history_digest());
    assert_eq!(report.dispositions_digest(), ordered.dispositions_digest());
    assert_eq!(
        report.document().map(|document| document.byte_len()),
        ordered.document().map(|document| document.byte_len())
    );
}

#[test]
#[allow(clippy::expect_used)]
fn signed_child_cannot_discard_retained_writer_contributions() {
    let controller = TestSigner::from_byte(30);
    let retained = TestSigner::from_byte(31);
    let removed = TestSigner::from_byte(32);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "50".repeat(32)
    )
    .parse()
    .expect("fixed coordinate");
    let mut members = [retained.public_key(), removed.public_key()]
        .into_iter()
        .map(|key| {
            format!(
                r#"{{"account":null,"pubkey":"{}","roles":["checkpoint","write"]}}"#,
                key.to_hex()
            )
        })
        .collect::<Vec<_>>();
    members.sort();
    let parent_content = format!(
        r#"{{"base_heads":[],"format":"automerge-change-v1","members":[{}],"policy":"controller-acl-v1","predecessor":null,"seq":0,"successor":null,"text_encoding":"utf16","v":1}}"#,
        members.join(",")
    );
    let child_content = format!(
        r#"{{"base_heads":[],"format":"automerge-change-v1","members":[{{"account":null,"pubkey":"{}","roles":["checkpoint","write"]}}],"policy":"controller-acl-v1","predecessor":null,"seq":1,"successor":null,"text_encoding":"utf16","v":1}}"#,
        retained.public_key().to_hex()
    );
    let parent = controller.sign(
        &UnsignedEventDraft::new(
            1,
            1_625,
            vec![vec!["a".to_owned(), coordinate.to_address()]],
            parent_content,
        )
        .expect("parent draft")
        .prepare(controller.public_key())
        .expect("parent preimage"),
    );
    let parent_id = VerifiedNip01Event::verify(parent.clone())
        .expect("signed parent")
        .event_id();
    let actor = ActorId::derive(coordinate, retained.public_key());
    let mut document = AuthoringDocument::empty(ActorState::initial(actor, BTreeSet::new()))
        .expect("empty authoring document");
    let authored = document
        .author_change(&[Operation::PutString {
            key: "retained".to_owned(),
            value: "required".to_owned(),
        }])
        .expect("canonical authored change");
    let change_hash = authored.change_hash();
    let change = retained.sign(
        &UnsignedEventDraft::new(
            2,
            1_624,
            vec![
                vec!["a".to_owned(), coordinate.to_address()],
                vec!["e".to_owned(), parent_id.to_hex()],
                vec!["x".to_owned(), change_hash.to_hex()],
            ],
            base64::engine::general_purpose::STANDARD.encode(authored.raw()),
        )
        .expect("change draft")
        .prepare(retained.public_key())
        .expect("change preimage"),
    );
    let child = controller.sign(
        &UnsignedEventDraft::new(
            3,
            1_625,
            vec![
                vec!["a".to_owned(), coordinate.to_address()],
                vec!["e".to_owned(), parent_id.to_hex()],
            ],
            child_content,
        )
        .expect("child draft")
        .prepare(controller.public_key())
        .expect("child preimage"),
    );
    let child_id = VerifiedNip01Event::verify(child.clone())
        .expect("signed child")
        .event_id();
    let mut builder = CorpusBuilder::new();
    for event in [child, change, parent] {
        assert!(matches!(
            builder.ingest(event),
            IngestOutcome::Accepted { .. }
        ));
    }
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate(
        &builder.finish(),
        coordinate,
        &mut WorkBudget::new(1_000_000, 1_000),
        &NeverCancelled,
    );
    assert_eq!(report.completion(), Completion::Complete);
    assert_eq!(report.canonical_controls(), [parent_id]);
    assert!(!report.canonical_controls().contains(&child_id));
    assert_eq!(report.accepted_changes(), [change_hash]);
    assert_eq!(report.heads(), [change_hash]);
    assert!(report.document().is_some_and(|view| !view.is_empty()));
}

#[test]
#[allow(clippy::expect_used)]
fn late_lower_control_id_reorganizes_and_replays_signed_state() {
    let controller = TestSigner::from_byte(33);
    let device = TestSigner::from_byte(34);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "51".repeat(32)
    )
    .parse()
    .expect("fixed coordinate");
    let content = |sequence: u64| {
        format!(
            r#"{{"base_heads":[],"format":"automerge-change-v1","members":[{{"account":null,"pubkey":"{}","roles":["checkpoint","write"]}}],"policy":"controller-acl-v1","predecessor":null,"seq":{sequence},"successor":null,"text_encoding":"utf16","v":1}}"#,
            device.public_key().to_hex()
        )
    };
    let sign_control = |created_at: u64, parent: Option<EventId>| {
        let mut tags = vec![vec!["a".to_owned(), coordinate.to_address()]];
        if let Some(parent) = parent {
            tags.push(vec!["e".to_owned(), parent.to_hex()]);
        }
        controller.sign(
            &UnsignedEventDraft::new(
                created_at,
                1_625,
                tags,
                content(u64::from(parent.is_some())),
            )
            .expect("control draft")
            .prepare(controller.public_key())
            .expect("control preimage"),
        )
    };
    let parent = sign_control(1, None);
    let parent_id = VerifiedNip01Event::verify(parent.clone())
        .expect("signed parent")
        .event_id();
    let first = sign_control(2, Some(parent_id));
    let second = sign_control(3, Some(parent_id));
    let first_id = VerifiedNip01Event::verify(first.clone())
        .expect("signed first child")
        .event_id();
    let second_id = VerifiedNip01Event::verify(second.clone())
        .expect("signed second child")
        .event_id();
    let ((lower, lower_id), (higher, higher_id)) = if first_id < second_id {
        ((first, first_id), (second, second_id))
    } else {
        ((second, second_id), (first, first_id))
    };
    let sign_change = |control_id: EventId, value: &str, created_at: u64| {
        let actor = ActorId::derive(coordinate, device.public_key());
        let mut document = AuthoringDocument::empty(ActorState::initial(actor, BTreeSet::new()))
            .expect("empty authoring document");
        let authored = document
            .author_change(&[Operation::PutString {
                key: "winner".to_owned(),
                value: value.to_owned(),
            }])
            .expect("canonical authored change");
        let hash = authored.change_hash();
        let event = device.sign(
            &UnsignedEventDraft::new(
                created_at,
                1_624,
                vec![
                    vec!["a".to_owned(), coordinate.to_address()],
                    vec!["e".to_owned(), control_id.to_hex()],
                    vec!["x".to_owned(), hash.to_hex()],
                ],
                base64::engine::general_purpose::STANDARD.encode(authored.raw()),
            )
            .expect("change draft")
            .prepare(device.public_key())
            .expect("change preimage"),
        );
        (event, hash)
    };
    let (lower_change, lower_hash) = sign_change(lower_id, "lower", 4);
    let (higher_change, higher_hash) = sign_change(higher_id, "higher", 5);
    let evaluate = |events: &[RawEventBytes]| {
        let mut builder = CorpusBuilder::new();
        for event in events {
            assert!(matches!(
                builder.ingest(event.clone()),
                IngestOutcome::Accepted { .. }
            ));
        }
        ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate(
            &builder.finish(),
            coordinate,
            &mut WorkBudget::new(1_000_000, 1_000),
            &NeverCancelled,
        )
    };
    let before = evaluate(&[parent.clone(), higher.clone(), higher_change.clone()]);
    assert_eq!(before.canonical_controls(), [parent_id, higher_id]);
    assert_eq!(before.accepted_changes(), [higher_hash]);
    assert_eq!(before.heads(), [higher_hash]);
    let after = evaluate(&[parent, higher, higher_change, lower, lower_change]);
    assert_eq!(after.completion(), Completion::Complete);
    assert_eq!(after.canonical_controls(), [parent_id, lower_id]);
    assert_eq!(after.accepted_changes(), [lower_hash]);
    assert_eq!(after.heads(), [lower_hash]);
    assert!(
        after
            .dispositions()
            .contains(&(higher_hash, ProtocolDisposition::Excluded))
    );
    assert!(after.integrity_alerts().iter().any(|alert| matches!(
        alert,
        nostr_automerge::IntegrityAlert::ControllerEquivocation { .. }
    )));
    assert!(after.document().is_some_and(|view| !view.is_empty()));
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
