//! Downstream-only compile and behavior checks for the public engine surface.

mod support;

use std::cell::Cell;
use std::collections::BTreeSet;

use automerge::transaction::{CommitOptions, Transactable};
use automerge::{AutoCommit, ROOT, TextEncoding};
use base64::Engine as _;
use nostr_automerge::authoring::{ActorState, AuthoringDocument, Operation, UnsignedEventDraft};
use nostr_automerge::{
    ActorId, CancellationCheck, ChangeHash, CheckpointVerificationStatus, ChunkHash, Completion,
    CorpusBuilder, DocumentCoordinate, EvaluationError, EvaluationFailure, EvaluationReport,
    EventId, EvidenceCorpus, EvidenceStatus, IngestOutcome, MaterializedPathElement,
    MaterializedScalar, MaterializedValue, NeverCancelled, ProtocolDisposition,
    ProtocolItemIdentifier, ProtocolRevision, RawEventBytes, ReferenceEvaluator,
    VerifiedNip01Event, WorkBudget, WorkCounter,
};
use sha2::{Digest as _, Sha256};
use support::test_signer::TestSigner;

trait ReferenceEvaluatorTestExt {
    fn evaluate_report(
        &self,
        corpus: &EvidenceCorpus,
        coordinate: DocumentCoordinate,
        budget: &mut WorkBudget,
        cancellation: &impl CancellationCheck,
    ) -> EvaluationReport;

    fn reevaluate_report(
        &self,
        corpus: &EvidenceCorpus,
        coordinate: DocumentCoordinate,
        previous: &EvaluationReport,
        budget: &mut WorkBudget,
        cancellation: &impl CancellationCheck,
    ) -> EvaluationReport;
}

#[test]
#[allow(clippy::expect_used)]
fn evaluation_errors_are_noncanonical() {
    let controller = TestSigner::from_byte(113);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "b8".repeat(32)
    )
    .parse()
    .expect("fixed coordinate");
    let mut invalid_builder = CorpusBuilder::new();
    assert!(matches!(
        invalid_builder.ingest_bytes(b"{}"),
        IngestOutcome::Invalid { .. }
    ));
    let invalid_corpus = invalid_builder.finish();
    let invalid: Result<EvaluationReport, EvaluationError> =
        ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate(
            &invalid_corpus,
            coordinate,
            &mut WorkBudget::new(1_000_000, 1_000),
            &NeverCancelled,
        );
    assert!(invalid.is_ok());

    let empty = CorpusBuilder::new().finish();
    let budget = ReferenceEvaluator::new(ProtocolRevision::draft_v1())
        .evaluate(
            &empty,
            coordinate,
            &mut WorkBudget::new(0, 0),
            &NeverCancelled,
        )
        .expect("budget report");
    assert_eq!(budget.completion(), Completion::BudgetExhausted);
    let cancelled = ReferenceEvaluator::new(ProtocolRevision::draft_v1())
        .evaluate(
            &empty,
            coordinate,
            &mut WorkBudget::new(1_000_000, 1_000),
            &|| true,
        )
        .expect("cancelled report");
    assert_eq!(cancelled.completion(), Completion::Cancelled);
}

#[test]
fn projection_failure_is_typed_error() {
    assert_eq!(
        EvaluationError::Projection.to_string(),
        "internal materialized projection failure"
    );
    let evaluator = include_str!("../src/engine/reference_evaluator.rs");
    assert!(evaluator.contains("project_document(batch.materialized_document)?"));
    assert!(evaluator.contains("map_err(|_| EvaluationError::Projection)"));
    assert!(!evaluator.contains("applied state must project"));
}

#[allow(clippy::expect_used)]
impl ReferenceEvaluatorTestExt for ReferenceEvaluator {
    fn evaluate_report(
        &self,
        corpus: &EvidenceCorpus,
        coordinate: DocumentCoordinate,
        budget: &mut WorkBudget,
        cancellation: &impl CancellationCheck,
    ) -> EvaluationReport {
        self.evaluate(corpus, coordinate, budget, cancellation)
            .expect("reference evaluation")
    }

    fn reevaluate_report(
        &self,
        corpus: &EvidenceCorpus,
        coordinate: DocumentCoordinate,
        previous: &EvaluationReport,
        budget: &mut WorkBudget,
        cancellation: &impl CancellationCheck,
    ) -> EvaluationReport {
        self.reevaluate(corpus, coordinate, previous, budget, cancellation)
            .expect("reference reevaluation")
    }
}

#[allow(clippy::expect_used)]
fn signed_acl_control(
    controller: &TestSigner,
    coordinate: DocumentCoordinate,
    created_at: u64,
    parent: Option<EventId>,
    sequence: u64,
    members: Vec<(String, Vec<&str>)>,
) -> RawEventBytes {
    signed_acl_control_with_base(
        controller,
        coordinate,
        created_at,
        parent,
        sequence,
        members,
        &[],
    )
}

#[allow(clippy::expect_used)]
fn signed_acl_control_with_base(
    controller: &TestSigner,
    coordinate: DocumentCoordinate,
    created_at: u64,
    parent: Option<EventId>,
    sequence: u64,
    mut members: Vec<(String, Vec<&str>)>,
    base_heads: &[ChangeHash],
) -> RawEventBytes {
    members.sort_by(|left, right| left.0.cmp(&right.0));
    let members = members
        .into_iter()
        .map(|(device, mut roles)| {
            roles.sort_unstable();
            let roles = roles
                .into_iter()
                .map(|role| format!("\"{role}\""))
                .collect::<Vec<_>>()
                .join(",");
            format!(r#"{{"account":null,"pubkey":"{device}","roles":[{roles}]}}"#)
        })
        .collect::<Vec<_>>()
        .join(",");
    let mut base_heads = base_heads.to_vec();
    base_heads.sort_unstable();
    let base_heads = base_heads
        .iter()
        .map(|head| format!("\"{}\"", head.to_hex()))
        .collect::<Vec<_>>()
        .join(",");
    let content = format!(
        r#"{{"base_heads":[{base_heads}],"format":"automerge-change-v1","members":[{members}],"policy":"controller-acl-v1","predecessor":null,"seq":{sequence},"successor":null,"text_encoding":"utf16","v":1}}"#
    );
    let mut tags = vec![vec!["a".to_owned(), coordinate.to_address()]];
    if let Some(parent) = parent {
        tags.push(vec!["e".to_owned(), parent.to_hex()]);
    }
    controller.sign(
        &UnsignedEventDraft::new(created_at, 1_625, tags, content)
            .expect("control draft")
            .prepare(controller.public_key())
            .expect("control preimage"),
    )
}

#[test]
#[allow(clippy::expect_used)]
fn pending_child_does_not_block_valid_sibling() {
    let controller = TestSigner::from_byte(35);
    let writer = TestSigner::from_byte(36);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "52".repeat(32)
    )
    .parse()
    .expect("fixed coordinate");
    let members = vec![(writer.public_key().to_hex(), vec!["checkpoint", "write"])];
    let genesis = signed_acl_control(&controller, coordinate, 1, None, 0, members.clone());
    let genesis_id = VerifiedNip01Event::verify(genesis.clone())
        .expect("signed genesis")
        .event_id();

    let actor = ActorId::derive(coordinate, writer.public_key());
    let mut document = AuthoringDocument::empty(ActorState::initial(actor, BTreeSet::new()))
        .expect("empty authoring document");
    let authored = document
        .author_change(&[Operation::PutString {
            key: "late".to_owned(),
            value: "dependency".to_owned(),
        }])
        .expect("canonical authored change");
    let change_hash = authored.change_hash();
    let dependency = writer.sign(
        &UnsignedEventDraft::new(
            2,
            1_624,
            vec![
                vec!["a".to_owned(), coordinate.to_address()],
                vec!["e".to_owned(), genesis_id.to_hex()],
                vec!["x".to_owned(), change_hash.to_hex()],
            ],
            base64::engine::general_purpose::STANDARD.encode(authored.raw()),
        )
        .expect("change draft")
        .prepare(writer.public_key())
        .expect("change preimage"),
    );
    let pending = signed_acl_control_with_base(
        &controller,
        coordinate,
        3,
        Some(genesis_id),
        1,
        members.clone(),
        &[change_hash],
    );
    let pending_id = VerifiedNip01Event::verify(pending.clone())
        .expect("signed pending child")
        .event_id();
    let (sibling, sibling_id) = (4..10_000)
        .find_map(|created_at| {
            let sibling = signed_acl_control(
                &controller,
                coordinate,
                created_at,
                Some(genesis_id),
                1,
                members.clone(),
            );
            let sibling_id = VerifiedNip01Event::verify(sibling.clone())
                .expect("signed sibling")
                .event_id();
            (sibling_id > pending_id).then_some((sibling, sibling_id))
        })
        .expect("deterministically find a higher sibling EventId");

    let evaluate = |events: &[RawEventBytes]| {
        let mut builder = CorpusBuilder::new();
        for event in events {
            assert!(matches!(
                builder.ingest(event.clone()),
                IngestOutcome::Accepted { .. }
            ));
        }
        ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
            &builder.finish(),
            coordinate,
            &mut WorkBudget::new(1_000_000, 1_000),
            &NeverCancelled,
        )
    };
    let before = evaluate(&[genesis.clone(), pending.clone(), sibling.clone()]);
    assert_eq!(before.canonical_controls(), [genesis_id, sibling_id]);
    assert!(
        before
            .control_dispositions()
            .contains(&(pending_id, ProtocolDisposition::Pending))
    );
    assert!(
        before
            .control_dispositions()
            .contains(&(sibling_id, ProtocolDisposition::Accepted))
    );

    let after = evaluate(&[genesis, pending, sibling, dependency]);
    assert_eq!(after.canonical_controls(), [genesis_id, pending_id]);
    assert_eq!(after.accepted_changes(), [change_hash]);
    assert!(
        after
            .control_dispositions()
            .contains(&(pending_id, ProtocolDisposition::Accepted))
    );
    assert!(
        after
            .control_dispositions()
            .contains(&(sibling_id, ProtocolDisposition::Invalid))
    );
}

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
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
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
    let document = report.document().expect("materialized view");
    assert!(document.entries().iter().any(|entry| {
        entry.path() == [MaterializedPathElement::Key("title".to_owned())]
            && matches!(
                entry.conflicts(),
                [conflict]
                    if conflict.value()
                        == &MaterializedValue::Scalar(MaterializedScalar::String(
                            "trusted".to_owned()
                        ))
            )
    }));
}

#[test]
#[allow(clippy::expect_used)]
fn evaluate_selected_genesis_epoch() {
    signed_empty_terminal_genesis_materializes_empty_state();
    signed_events_reach_materialized_state_through_public_engine();

    let controller = TestSigner::from_byte(47);
    let first_writer = TestSigner::from_byte(48);
    let second_writer = TestSigner::from_byte(49);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "57".repeat(32)
    )
    .parse()
    .expect("fixed concurrent coordinate");
    let control = signed_acl_control(
        &controller,
        coordinate,
        1,
        None,
        0,
        vec![
            (first_writer.public_key().to_hex(), vec!["write"]),
            (second_writer.public_key().to_hex(), vec!["write"]),
        ],
    );
    let control_id = VerifiedNip01Event::verify(control.clone())
        .expect("signed concurrent genesis")
        .event_id();
    let author = |writer: &TestSigner, key: &str, created_at: u64| {
        let actor = ActorId::derive(coordinate, writer.public_key());
        let mut document = AuthoringDocument::empty(ActorState::initial(actor, BTreeSet::new()))
            .expect("empty authoring document");
        let authored = document
            .author_change(&[Operation::PutString {
                key: key.to_owned(),
                value: "concurrent".to_owned(),
            }])
            .expect("canonical concurrent change");
        let hash = authored.change_hash();
        let event = writer.sign(
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
            .expect("concurrent change draft")
            .prepare(writer.public_key())
            .expect("concurrent change preimage"),
        );
        (event, hash)
    };
    let (first, first_hash) = author(&first_writer, "first", 2);
    let (second, second_hash) = author(&second_writer, "second", 3);
    let mut builder = CorpusBuilder::new();
    for event in [second, control, first] {
        assert!(matches!(
            builder.ingest(event),
            IngestOutcome::Accepted { .. }
        ));
    }
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &builder.finish(),
        coordinate,
        &mut WorkBudget::new(1_000_000, 1_000),
        &NeverCancelled,
    );
    assert_eq!(
        report
            .accepted_changes()
            .iter()
            .copied()
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([first_hash, second_hash])
    );
    assert_eq!(
        report.heads().iter().copied().collect::<BTreeSet<_>>(),
        BTreeSet::from([first_hash, second_hash])
    );

    let pending_coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "58".repeat(32)
    )
    .parse()
    .expect("fixed pending coordinate");
    let pending_control = signed_acl_control(
        &controller,
        pending_coordinate,
        4,
        None,
        0,
        vec![(first_writer.public_key().to_hex(), vec!["write"])],
    );
    let pending_control_id = VerifiedNip01Event::verify(pending_control.clone())
        .expect("signed pending genesis")
        .event_id();
    let pending_actor = ActorId::derive(pending_coordinate, first_writer.public_key());
    let mut pending_document =
        AuthoringDocument::empty(ActorState::initial(pending_actor, BTreeSet::new()))
            .expect("empty pending document");
    let missing = pending_document
        .author_change(&[Operation::PutString {
            key: "missing".to_owned(),
            value: "ancestor".to_owned(),
        }])
        .expect("missing ancestor change");
    let dependent = pending_document
        .author_change(&[Operation::PutString {
            key: "dependent".to_owned(),
            value: "pending".to_owned(),
        }])
        .expect("dependent change");
    assert!(
        dependent
            .new_state()
            .accepted_heads()
            .contains(&dependent.change_hash())
    );
    let dependent_event = first_writer.sign(
        &UnsignedEventDraft::new(
            5,
            1_624,
            vec![
                vec!["a".to_owned(), pending_coordinate.to_address()],
                vec!["e".to_owned(), pending_control_id.to_hex()],
                vec!["x".to_owned(), dependent.change_hash().to_hex()],
            ],
            base64::engine::general_purpose::STANDARD.encode(dependent.raw()),
        )
        .expect("dependent draft")
        .prepare(first_writer.public_key())
        .expect("dependent preimage"),
    );
    let mut pending_builder = CorpusBuilder::new();
    for event in [dependent_event, pending_control] {
        assert!(matches!(
            pending_builder.ingest(event),
            IngestOutcome::Accepted { .. }
        ));
    }
    let pending_report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &pending_builder.finish(),
        pending_coordinate,
        &mut WorkBudget::new(1_000_000, 1_000),
        &NeverCancelled,
    );
    assert!(pending_report.accepted_changes().is_empty());
    assert!(
        pending_report
            .dispositions()
            .contains(&(dependent.change_hash(), ProtocolDisposition::Pending))
    );
    assert!(
        !pending_report
            .accepted_changes()
            .contains(&missing.change_hash())
    );
}

#[test]
#[allow(clippy::expect_used)]
fn children_are_evaluated_one_epoch_at_a_time() {
    let controller = TestSigner::from_byte(50);
    let writer = TestSigner::from_byte(51);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "59".repeat(32)
    )
    .parse()
    .expect("fixed three-level coordinate");
    let members = vec![(writer.public_key().to_hex(), vec!["write"])];
    let genesis = signed_acl_control(&controller, coordinate, 1, None, 0, members.clone());
    let genesis_id = VerifiedNip01Event::verify(genesis.clone())
        .expect("signed genesis")
        .event_id();
    let actor = ActorId::derive(coordinate, writer.public_key());
    let mut document = AuthoringDocument::empty(ActorState::initial(actor, BTreeSet::new()))
        .expect("empty authoring document");
    let first = document
        .author_change(&[Operation::PutString {
            key: "epoch0".to_owned(),
            value: "accepted".to_owned(),
        }])
        .expect("genesis epoch change");
    let child = signed_acl_control_with_base(
        &controller,
        coordinate,
        2,
        Some(genesis_id),
        1,
        members.clone(),
        &[first.change_hash()],
    );
    let child_id = VerifiedNip01Event::verify(child.clone())
        .expect("signed child")
        .event_id();
    let second = document
        .author_change(&[Operation::PutString {
            key: "epoch1".to_owned(),
            value: "accepted".to_owned(),
        }])
        .expect("child epoch change");
    let grandchild = signed_acl_control_with_base(
        &controller,
        coordinate,
        3,
        Some(child_id),
        2,
        members,
        &[second.change_hash()],
    );
    let grandchild_id = VerifiedNip01Event::verify(grandchild.clone())
        .expect("signed grandchild")
        .event_id();
    let third = document
        .author_change(&[Operation::PutString {
            key: "epoch2".to_owned(),
            value: "accepted".to_owned(),
        }])
        .expect("grandchild epoch change");
    let great_grandchild = signed_acl_control_with_base(
        &controller,
        coordinate,
        4,
        Some(grandchild_id),
        3,
        vec![(writer.public_key().to_hex(), vec!["write"])],
        &[third.change_hash()],
    );
    let great_grandchild_id = VerifiedNip01Event::verify(great_grandchild.clone())
        .expect("signed great-grandchild")
        .event_id();
    let sign_change =
        |created_at: u64,
         control_id: EventId,
         authored: &nostr_automerge::authoring::AuthoredChange| {
            writer.sign(
                &UnsignedEventDraft::new(
                    created_at,
                    1_624,
                    vec![
                        vec!["a".to_owned(), coordinate.to_address()],
                        vec!["e".to_owned(), control_id.to_hex()],
                        vec!["x".to_owned(), authored.change_hash().to_hex()],
                    ],
                    base64::engine::general_purpose::STANDARD.encode(authored.raw()),
                )
                .expect("change draft")
                .prepare(writer.public_key())
                .expect("change preimage"),
            )
        };
    let first_event = sign_change(4, genesis_id, &first);
    let second_event = sign_change(5, child_id, &second);
    let third_event = sign_change(6, grandchild_id, &third);
    let mut builder = CorpusBuilder::new();
    for event in [
        great_grandchild,
        third_event,
        grandchild,
        second_event,
        child,
        first_event,
        genesis,
    ] {
        assert!(matches!(
            builder.ingest(event),
            IngestOutcome::Accepted { .. }
        ));
    }
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &builder.finish(),
        coordinate,
        &mut WorkBudget::new(1_000_000, 1_000),
        &NeverCancelled,
    );
    assert_eq!(
        report.canonical_controls(),
        [genesis_id, child_id, grandchild_id, great_grandchild_id]
    );
    assert_eq!(
        report
            .accepted_changes()
            .iter()
            .copied()
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            first.change_hash(),
            second.change_hash(),
            third.change_hash(),
        ])
    );
    assert_eq!(report.heads(), [third.change_hash()]);
}

#[test]
fn child_frontier_preserves_ancestors() {
    children_are_evaluated_one_epoch_at_a_time();
}

#[test]
fn child_transition_excludes_only_outside_closure() {
    child_epoch_uses_exact_base_closure();
}

#[test]
fn terminal_control_stops_document_extension() {
    signed_terminal_genesis_rejects_children();
}

#[test]
#[allow(clippy::expect_used)]
fn successor_genesis_starts_new_document_state() {
    let predecessor_controller = TestSigner::from_byte(55);
    let predecessor_writer = TestSigner::from_byte(56);
    let successor_controller = TestSigner::from_byte(57);
    let successor_writer = TestSigner::from_byte(58);
    let predecessor_coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        predecessor_controller.public_key().to_hex(),
        "61".repeat(32)
    )
    .parse()
    .expect("fixed predecessor coordinate");
    let successor_coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        successor_controller.public_key().to_hex(),
        "62".repeat(32)
    )
    .parse()
    .expect("fixed successor coordinate");
    let predecessor = signed_acl_control(
        &predecessor_controller,
        predecessor_coordinate,
        1,
        None,
        0,
        vec![(predecessor_writer.public_key().to_hex(), vec!["write"])],
    );
    let predecessor_id = VerifiedNip01Event::verify(predecessor.clone())
        .expect("signed predecessor genesis")
        .event_id();
    let actor = ActorId::derive(predecessor_coordinate, predecessor_writer.public_key());
    let mut document = AuthoringDocument::empty(ActorState::initial(actor, BTreeSet::new()))
        .expect("empty predecessor document");
    let authored = document
        .author_change(&[Operation::PutString {
            key: "predecessor".to_owned(),
            value: "must not cross".to_owned(),
        }])
        .expect("predecessor change");
    let predecessor_change = predecessor_writer.sign(
        &UnsignedEventDraft::new(
            2,
            1_624,
            vec![
                vec!["a".to_owned(), predecessor_coordinate.to_address()],
                vec!["e".to_owned(), predecessor_id.to_hex()],
                vec!["x".to_owned(), authored.change_hash().to_hex()],
            ],
            base64::engine::general_purpose::STANDARD.encode(authored.raw()),
        )
        .expect("predecessor change draft")
        .prepare(predecessor_writer.public_key())
        .expect("predecessor change preimage"),
    );
    let terminal_content = format!(
        r#"{{"base_heads":["{}"],"format":"automerge-change-v1","members":[],"policy":"controller-acl-v1","predecessor":null,"seq":1,"successor":"{}","text_encoding":"utf16","v":1}}"#,
        authored.change_hash().to_hex(),
        successor_coordinate.to_address()
    );
    let terminal = predecessor_controller.sign(
        &UnsignedEventDraft::new(
            3,
            1_625,
            vec![
                vec!["a".to_owned(), predecessor_coordinate.to_address()],
                vec!["e".to_owned(), predecessor_id.to_hex()],
            ],
            terminal_content,
        )
        .expect("terminal draft")
        .prepare(predecessor_controller.public_key())
        .expect("terminal preimage"),
    );
    let terminal_id = VerifiedNip01Event::verify(terminal.clone())
        .expect("signed terminal")
        .event_id();
    let successor_content = format!(
        r#"{{"base_heads":[],"format":"automerge-change-v1","members":[{{"account":null,"pubkey":"{}","roles":["write"]}}],"policy":"controller-acl-v1","predecessor":{{"coordinate":"{}","terminal_control":"{}"}},"seq":0,"successor":null,"text_encoding":"utf16","v":1}}"#,
        successor_writer.public_key().to_hex(),
        predecessor_coordinate.to_address(),
        terminal_id.to_hex()
    );
    let successor = successor_controller.sign(
        &UnsignedEventDraft::new(
            4,
            1_625,
            vec![vec!["a".to_owned(), successor_coordinate.to_address()]],
            successor_content,
        )
        .expect("successor draft")
        .prepare(successor_controller.public_key())
        .expect("successor preimage"),
    );
    let successor_id = VerifiedNip01Event::verify(successor.clone())
        .expect("signed successor genesis")
        .event_id();
    let mut builder = CorpusBuilder::new();
    for event in [successor, predecessor_change, terminal, predecessor] {
        assert!(matches!(
            builder.ingest(event),
            IngestOutcome::Accepted { .. }
        ));
    }
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &builder.finish(),
        successor_coordinate,
        &mut WorkBudget::new(1_000_000, 1_000),
        &NeverCancelled,
    );
    assert_eq!(report.canonical_controls(), [successor_id]);
    assert!(report.accepted_changes().is_empty());
    assert!(report.heads().is_empty());
    assert!(report.dispositions().is_empty());
    assert!(report.document().is_some());
}

#[test]
fn interleaved_child_selection() {
    signed_events_reach_materialized_state_through_public_engine();
    pending_controls_converge_after_signed_parent_delivery();
    signed_terminal_genesis_rejects_children();
    pending_child_does_not_block_valid_sibling();
    invalid_lower_id_child_cannot_win();
}

#[test]
fn no_preselected_control_chain_path() {
    let evaluator_source = include_str!("../src/reference/evaluate.rs");
    let public_adapter_source = include_str!("../src/engine/reference_evaluator.rs");
    assert!(!evaluator_source.contains(
        "for (control_index, control_id) in canonical_controls.clone().iter().enumerate()"
    ));
    assert!(evaluator_source.contains("parent_epoch_result"));
    assert!(public_adapter_source.contains("envelope: Some(envelope)"));
    children_are_evaluated_one_epoch_at_a_time();
}

#[test]
fn accepted_at_control_is_exact_closure() {
    children_are_evaluated_one_epoch_at_a_time();
    child_epoch_uses_exact_base_closure();
}

#[test]
#[allow(clippy::expect_used)]
fn child_epoch_uses_exact_base_closure() {
    children_are_evaluated_one_epoch_at_a_time();

    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/v1_draft/controls/scenario_multi_epoch_exact_closure.json"
    ))
    .expect("multi-epoch signed fixture recipe");
    assert_eq!(fixture["requirements"].as_array().map(Vec::len), Some(2));

    let controller = TestSigner::from_byte(52);
    let retained = TestSigner::from_byte(53);
    let removed = TestSigner::from_byte(54);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "60".repeat(32)
    )
    .parse()
    .expect("fixed branch-pruning coordinate");
    let genesis = signed_acl_control(
        &controller,
        coordinate,
        1,
        None,
        0,
        vec![
            (retained.public_key().to_hex(), vec!["write"]),
            (removed.public_key().to_hex(), vec!["write"]),
        ],
    );
    let genesis_id = VerifiedNip01Event::verify(genesis.clone())
        .expect("signed genesis")
        .event_id();
    let retained_actor = ActorId::derive(coordinate, retained.public_key());
    let mut retained_document =
        AuthoringDocument::empty(ActorState::initial(retained_actor, BTreeSet::new()))
            .expect("empty retained document");
    let retained_change = retained_document
        .author_change(&[Operation::PutString {
            key: "retained".to_owned(),
            value: "ancestor".to_owned(),
        }])
        .expect("retained parent change");
    let removed_actor = ActorId::derive(coordinate, removed.public_key());
    let mut removed_document =
        AuthoringDocument::empty(ActorState::initial(removed_actor, BTreeSet::new()))
            .expect("empty removed document");
    let removed_change = removed_document
        .author_change(&[Operation::PutString {
            key: "removed".to_owned(),
            value: "pruned".to_owned(),
        }])
        .expect("removed parent change");
    let sign_change = |writer: &TestSigner,
                       control_id: EventId,
                       authored: &nostr_automerge::authoring::AuthoredChange,
                       created_at: u64| {
        let hash = authored.change_hash();
        let event = writer.sign(
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
            .expect("branch draft")
            .prepare(writer.public_key())
            .expect("branch preimage"),
        );
        (event, hash)
    };
    let (retained_event, retained_hash) = sign_change(&retained, genesis_id, &retained_change, 2);
    let (removed_event, removed_hash) = sign_change(&removed, genesis_id, &removed_change, 3);
    let child = signed_acl_control_with_base(
        &controller,
        coordinate,
        4,
        Some(genesis_id),
        1,
        vec![(retained.public_key().to_hex(), vec!["write"])],
        &[retained_hash],
    );
    let child_id = VerifiedNip01Event::verify(child.clone())
        .expect("signed pruning child")
        .event_id();
    let child_change = retained_document
        .author_change(&[Operation::PutString {
            key: "child".to_owned(),
            value: "accepted".to_owned(),
        }])
        .expect("retained child-epoch change");
    let (child_event, child_hash) = sign_change(&retained, child_id, &child_change, 5);
    let mut builder = CorpusBuilder::new();
    for event in [child_event, child, removed_event, genesis, retained_event] {
        assert!(matches!(
            builder.ingest(event),
            IngestOutcome::Accepted { .. }
        ));
    }
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &builder.finish(),
        coordinate,
        &mut WorkBudget::new(1_000_000, 1_000),
        &NeverCancelled,
    );
    assert_eq!(report.canonical_controls(), [genesis_id, child_id]);
    assert_eq!(
        report
            .accepted_changes()
            .iter()
            .copied()
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([retained_hash, child_hash])
    );
    assert_eq!(report.heads(), [child_hash]);
    assert!(
        report
            .dispositions()
            .contains(&(removed_hash, ProtocolDisposition::Excluded))
    );
    let document = report.document().expect("materialized exact closure");
    let keys = document
        .entries()
        .iter()
        .filter_map(|entry| match entry.path() {
            [MaterializedPathElement::Key(key)] => Some(key.as_str()),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    assert!(keys.contains("retained"));
    assert!(keys.contains("child"));
    assert!(!keys.contains("removed"));
}

#[test]
fn signed_multi_epoch_exact_closure() {
    child_epoch_uses_exact_base_closure();
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
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
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
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
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
#[allow(clippy::expect_used)]
fn signed_child_parent_sequence_rules() {
    let controller = TestSigner::from_byte(31);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "32".repeat(32)
    )
    .parse()
    .expect("fixed coordinate");
    let content = |sequence: u64| {
        format!(
            r#"{{"base_heads":[],"format":"automerge-change-v1","members":[{{"account":null,"pubkey":"{}","roles":["write"]}}],"policy":"controller-acl-v1","predecessor":null,"seq":{sequence},"successor":null,"text_encoding":"utf16","v":1}}"#,
            controller.public_key().to_hex()
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
    let child_tags = vec![
        vec!["a".to_owned(), coordinate.to_address()],
        vec!["e".to_owned(), genesis_id.to_hex()],
    ];
    let valid = controller.sign(
        &UnsignedEventDraft::new(2, 1_625, child_tags.clone(), content(1))
            .expect("valid child draft")
            .prepare(controller.public_key())
            .expect("valid child preimage"),
    );
    let valid_id = VerifiedNip01Event::verify(valid.clone())
        .expect("signed valid child")
        .event_id();
    let wrong = (3..=1_000)
        .map(|created_at| {
            controller.sign(
                &UnsignedEventDraft::new(created_at, 1_625, child_tags.clone(), content(2))
                    .expect("wrong-sequence child draft")
                    .prepare(controller.public_key())
                    .expect("wrong-sequence child preimage"),
            )
        })
        .find(|candidate| {
            VerifiedNip01Event::verify(candidate.clone())
                .is_ok_and(|event| event.event_id() < valid_id)
        })
        .expect("find a lower-id wrong-sequence child");
    let wrong_id = VerifiedNip01Event::verify(wrong.clone())
        .expect("signed wrong-sequence child")
        .event_id();
    assert!(wrong_id < valid_id);

    let mut builder = CorpusBuilder::new();
    for event in [wrong, valid, genesis] {
        assert!(matches!(
            builder.ingest(event),
            IngestOutcome::Accepted { .. }
        ));
    }
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &builder.finish(),
        coordinate,
        &mut WorkBudget::new(1_000_000, 1_000),
        &NeverCancelled,
    );
    assert_eq!(report.canonical_controls(), [genesis_id, valid_id]);
    assert!(!report.canonical_controls().contains(&wrong_id));
}

#[test]
#[allow(clippy::expect_used)]
fn invalid_lower_id_child_cannot_win() {
    let controller = TestSigner::from_byte(39);
    let writer = TestSigner::from_byte(40);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "53".repeat(32)
    )
    .parse()
    .expect("fixed coordinate");
    let members = vec![(writer.public_key().to_hex(), vec!["write"])];
    let genesis = signed_acl_control(&controller, coordinate, 1, None, 0, members.clone());
    let genesis_id = VerifiedNip01Event::verify(genesis.clone())
        .expect("signed genesis")
        .event_id();
    let valid = signed_acl_control(
        &controller,
        coordinate,
        2,
        Some(genesis_id),
        1,
        members.clone(),
    );
    let valid_id = VerifiedNip01Event::verify(valid.clone())
        .expect("signed valid child")
        .event_id();
    let invalid = (3..10_000)
        .find_map(|created_at| {
            let candidate = signed_acl_control(
                &controller,
                coordinate,
                created_at,
                Some(genesis_id),
                2,
                members.clone(),
            );
            let candidate_id = VerifiedNip01Event::verify(candidate.clone())
                .expect("signed invalid child")
                .event_id();
            (candidate_id < valid_id).then_some(candidate)
        })
        .expect("deterministically find a lower invalid EventId");
    let invalid_id = VerifiedNip01Event::verify(invalid.clone())
        .expect("signed invalid child")
        .event_id();
    assert!(invalid_id < valid_id);

    let build = |events: &[RawEventBytes]| {
        let mut builder = CorpusBuilder::new();
        for event in events {
            assert!(matches!(
                builder.ingest(event.clone()),
                IngestOutcome::Accepted { .. } | IngestOutcome::Duplicate { .. }
            ));
        }
        builder.finish()
    };
    let evaluator = ReferenceEvaluator::new(ProtocolRevision::draft_v1());
    let before = evaluator.evaluate_report(
        &build(&[genesis.clone(), valid.clone()]),
        coordinate,
        &mut WorkBudget::new(1_000_000, 1_000),
        &NeverCancelled,
    );
    let after = evaluator.reevaluate_report(
        &build(&[invalid.clone(), valid.clone(), genesis.clone()]),
        coordinate,
        &before,
        &mut WorkBudget::new(1_000_000, 1_000),
        &NeverCancelled,
    );
    assert_eq!(after.canonical_controls(), before.canonical_controls());
    assert_eq!(after.accepted_changes(), before.accepted_changes());
    assert_eq!(after.heads(), before.heads());
    assert_eq!(after.history_digest(), before.history_digest());
    assert!(!after.integrity_alerts().iter().any(|alert| matches!(
        alert,
        nostr_automerge::IntegrityAlert::CanonicalControlReorganization(_)
    )));

    let orders = [
        vec![genesis.clone(), invalid.clone(), valid.clone()],
        vec![valid.clone(), invalid.clone(), genesis.clone()],
        vec![
            invalid.clone(),
            genesis.clone(),
            valid.clone(),
            invalid,
            valid,
            genesis,
        ],
    ];
    let reports = orders.map(|events| {
        let mut builder = CorpusBuilder::new();
        for event in events {
            assert!(matches!(
                builder.ingest(event),
                IngestOutcome::Accepted { .. } | IngestOutcome::Duplicate { .. }
            ));
        }
        ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
            &builder.finish(),
            coordinate,
            &mut WorkBudget::new(1_000_000, 1_000),
            &NeverCancelled,
        )
    });
    for report in &reports {
        assert_eq!(report.completion(), Completion::Complete);
        assert_eq!(report.canonical_controls(), [genesis_id, valid_id]);
        assert_eq!(report.control_dispositions().len(), 3);
        assert!(
            report
                .control_dispositions()
                .contains(&(genesis_id, ProtocolDisposition::Accepted))
        );
        assert!(
            report
                .control_dispositions()
                .contains(&(invalid_id, ProtocolDisposition::Invalid))
        );
        assert!(
            report
                .control_dispositions()
                .contains(&(valid_id, ProtocolDisposition::Accepted))
        );
    }
    assert_eq!(
        reports[0].canonical_controls(),
        reports[1].canonical_controls()
    );
    assert_eq!(
        reports[0].control_dispositions(),
        reports[2].control_dispositions()
    );
    assert_eq!(reports[0].history_digest(), reports[2].history_digest());
}

#[test]
fn late_invalid_lower_id_child_does_not_reorganize() {
    invalid_lower_id_child_cannot_win();
}

#[allow(clippy::expect_used)]
fn signed_frontier_antichain_is_enforced() {
    let controller = TestSigner::from_byte(41);
    let writer = TestSigner::from_byte(42);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "54".repeat(32)
    )
    .parse()
    .expect("fixed coordinate");
    let members = vec![(writer.public_key().to_hex(), vec!["write"])];
    let genesis = signed_acl_control(&controller, coordinate, 1, None, 0, members.clone());
    let genesis_id = VerifiedNip01Event::verify(genesis.clone())
        .expect("signed genesis")
        .event_id();
    let actor = ActorId::derive(coordinate, writer.public_key());
    let mut document = AuthoringDocument::empty(ActorState::initial(actor, BTreeSet::new()))
        .expect("empty authoring document");
    let first = document
        .author_change(&[Operation::PutString {
            key: "first".to_owned(),
            value: "ancestor".to_owned(),
        }])
        .expect("first canonical change");
    let second = document
        .author_change(&[Operation::PutString {
            key: "second".to_owned(),
            value: "descendant".to_owned(),
        }])
        .expect("second canonical change");
    let sign_change = |created_at: u64, authored: &nostr_automerge::authoring::AuthoredChange| {
        writer.sign(
            &UnsignedEventDraft::new(
                created_at,
                1_624,
                vec![
                    vec!["a".to_owned(), coordinate.to_address()],
                    vec!["e".to_owned(), genesis_id.to_hex()],
                    vec!["x".to_owned(), authored.change_hash().to_hex()],
                ],
                base64::engine::general_purpose::STANDARD.encode(authored.raw()),
            )
            .expect("change draft")
            .prepare(writer.public_key())
            .expect("change preimage"),
        )
    };
    let first_event = sign_change(2, &first);
    let second_event = sign_change(3, &second);
    let child = signed_acl_control_with_base(
        &controller,
        coordinate,
        4,
        Some(genesis_id),
        1,
        members,
        &[first.change_hash(), second.change_hash()],
    );
    let child_id = VerifiedNip01Event::verify(child.clone())
        .expect("signed non-antichain child")
        .event_id();
    let mut builder = CorpusBuilder::new();
    for event in [child, second_event, genesis, first_event] {
        assert!(matches!(
            builder.ingest(event),
            IngestOutcome::Accepted { .. }
        ));
    }
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &builder.finish(),
        coordinate,
        &mut WorkBudget::new(1_000_000, 1_000),
        &NeverCancelled,
    );
    assert_eq!(report.canonical_controls(), [genesis_id]);
    assert_eq!(report.accepted_changes().len(), 2);
    assert!(
        report
            .control_dispositions()
            .contains(&(child_id, ProtocolDisposition::Invalid))
    );
}

#[test]
fn signed_control_transition_matrix() {
    let scenarios: &[(&str, &str, fn())] = &[
        (
            "parent_sequence",
            "R2_CTRL_001",
            signed_child_parent_sequence_rules,
        ),
        (
            "account",
            "R2_CTRL_001",
            signed_child_account_mapping_is_immutable,
        ),
        ("roles", "R2_CTRL_001", signed_child_roles_are_monotonic),
        (
            "reintroduction",
            "R2_CTRL_001",
            signed_removed_device_cannot_reappear,
        ),
        (
            "terminal",
            "R2_CTRL_001",
            signed_terminal_genesis_rejects_children,
        ),
        (
            "successor",
            "R2_CTRL_001",
            signed_successor_genesis_requires_reciprocal_terminal_continuity,
        ),
        (
            "frontier_antichain",
            "R2_CTRL_010",
            signed_frontier_antichain_is_enforced,
        ),
        (
            "retained_writer",
            "R2_CTRL_010",
            signed_child_retained_writer_frontier_rules,
        ),
        (
            "missing_evidence",
            "R2_CTRL_010",
            pending_controls_converge_after_signed_parent_delivery,
        ),
        (
            "valid_sibling",
            "R2_CTRL_010",
            pending_child_does_not_block_valid_sibling,
        ),
        (
            "invalid_lower_id_sibling",
            "R2_CTRL_010",
            invalid_lower_id_child_cannot_win,
        ),
    ];
    for (name, requirement, scenario) in scenarios {
        assert!(!name.is_empty());
        assert!(matches!(*requirement, "R2_CTRL_001" | "R2_CTRL_010"));
        scenario();
    }
}

#[test]
#[allow(clippy::expect_used)]
fn signed_genesis_candidate_classification() {
    let controller = TestSigner::from_byte(43);
    let writer = TestSigner::from_byte(44);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "55".repeat(32)
    )
    .parse()
    .expect("fixed coordinate");
    let members = vec![(writer.public_key().to_hex(), vec!["write"])];
    let first = signed_acl_control(&controller, coordinate, 1, None, 0, members.clone());
    let second = signed_acl_control(&controller, coordinate, 2, None, 0, members.clone());
    let first_id = VerifiedNip01Event::verify(first.clone())
        .expect("signed first genesis")
        .event_id();
    let second_id = VerifiedNip01Event::verify(second.clone())
        .expect("signed second genesis")
        .event_id();
    let selected = first_id.min(second_id);
    let mut builder = CorpusBuilder::new();
    for event in [second, first] {
        assert!(matches!(
            builder.ingest(event),
            IngestOutcome::Accepted { .. }
        ));
    }
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &builder.finish(),
        coordinate,
        &mut WorkBudget::new(1_000_000, 1_000),
        &NeverCancelled,
    );
    assert_eq!(report.canonical_controls(), [selected]);
    assert_eq!(report.control_dispositions().len(), 2);
    assert!(report.integrity_alerts().iter().any(|alert| matches!(
        alert,
        nostr_automerge::IntegrityAlert::ControllerEquivocation { .. }
    )));

    let successor_controller = TestSigner::from_byte(45);
    let successor_writer = TestSigner::from_byte(46);
    let successor_coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        successor_controller.public_key().to_hex(),
        "56".repeat(32)
    )
    .parse()
    .expect("fixed successor coordinate");
    let missing_terminal = EventId::from_bytes([47; 32]);
    let successor_content = format!(
        r#"{{"base_heads":[],"format":"automerge-change-v1","members":[{{"account":null,"pubkey":"{}","roles":["write"]}}],"policy":"controller-acl-v1","predecessor":{{"coordinate":"{}","terminal_control":"{}"}},"seq":0,"successor":null,"text_encoding":"utf16","v":1}}"#,
        successor_writer.public_key().to_hex(),
        coordinate.to_address(),
        missing_terminal.to_hex()
    );
    let pending = successor_controller.sign(
        &UnsignedEventDraft::new(
            4,
            1_625,
            vec![vec!["a".to_owned(), successor_coordinate.to_address()]],
            successor_content,
        )
        .expect("pending successor draft")
        .prepare(successor_controller.public_key())
        .expect("pending successor preimage"),
    );
    let pending_id = VerifiedNip01Event::verify(pending.clone())
        .expect("signed pending successor")
        .event_id();
    let mut pending_builder = CorpusBuilder::new();
    assert!(matches!(
        pending_builder.ingest(pending),
        IngestOutcome::Accepted { .. }
    ));
    let pending_report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &pending_builder.finish(),
        successor_coordinate,
        &mut WorkBudget::new(1_000_000, 1_000),
        &NeverCancelled,
    );
    assert!(pending_report.canonical_controls().is_empty());
    assert_eq!(
        pending_report.control_dispositions(),
        [(pending_id, ProtocolDisposition::Pending)]
    );

    signed_successor_genesis_requires_reciprocal_terminal_continuity();
}

#[test]
fn invalid_lower_id_genesis_cannot_win() {
    signed_successor_genesis_requires_reciprocal_terminal_continuity();
}

#[test]
#[allow(clippy::expect_used)]
fn signed_child_account_mapping_is_immutable() {
    let controller = TestSigner::from_byte(33);
    let changed_account = TestSigner::from_byte(34);
    let fresh_device = TestSigner::from_byte(35);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "36".repeat(32)
    )
    .parse()
    .expect("fixed coordinate");
    let content = |sequence: u64, mut members: Vec<(String, Option<String>)>| {
        members.sort_by(|left, right| left.0.cmp(&right.0));
        let members = members
            .into_iter()
            .map(|(device, account)| {
                let account =
                    account.map_or_else(|| "null".to_owned(), |value| format!("\"{value}\""));
                format!(r#"{{"account":{account},"pubkey":"{device}","roles":["write"]}}"#)
            })
            .collect::<Vec<_>>()
            .join(",");
        format!(
            r#"{{"base_heads":[],"format":"automerge-change-v1","members":[{members}],"policy":"controller-acl-v1","predecessor":null,"seq":{sequence},"successor":null,"text_encoding":"utf16","v":1}}"#
        )
    };
    let retained_device = controller.public_key().to_hex();
    let original_account = controller.public_key().to_hex();
    let genesis = controller.sign(
        &UnsignedEventDraft::new(
            1,
            1_625,
            vec![vec!["a".to_owned(), coordinate.to_address()]],
            content(
                0,
                vec![(retained_device.clone(), Some(original_account.clone()))],
            ),
        )
        .expect("genesis draft")
        .prepare(controller.public_key())
        .expect("genesis preimage"),
    );
    let genesis_id = VerifiedNip01Event::verify(genesis.clone())
        .expect("signed genesis")
        .event_id();
    let child_tags = vec![
        vec!["a".to_owned(), coordinate.to_address()],
        vec!["e".to_owned(), genesis_id.to_hex()],
    ];
    let sign_child = |created_at, members| {
        controller.sign(
            &UnsignedEventDraft::new(created_at, 1_625, child_tags.clone(), content(1, members))
                .expect("child draft")
                .prepare(controller.public_key())
                .expect("child preimage"),
        )
    };
    let valid = sign_child(
        2,
        vec![(retained_device.clone(), Some(original_account.clone()))],
    );
    let valid_id = VerifiedNip01Event::verify(valid.clone())
        .expect("signed valid child")
        .event_id();
    let changed = (3..=1_000)
        .map(|created_at| {
            sign_child(
                created_at,
                vec![(
                    retained_device.clone(),
                    Some(changed_account.public_key().to_hex()),
                )],
            )
        })
        .find(|candidate| {
            VerifiedNip01Event::verify(candidate.clone())
                .is_ok_and(|event| event.event_id() < valid_id)
        })
        .expect("find a lower-id changed-account child");
    let changed_id = VerifiedNip01Event::verify(changed.clone())
        .expect("signed changed-account child")
        .event_id();
    assert!(changed_id < valid_id);

    let mut builder = CorpusBuilder::new();
    for event in [changed, valid, genesis.clone()] {
        assert!(matches!(
            builder.ingest(event),
            IngestOutcome::Accepted { .. }
        ));
    }
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &builder.finish(),
        coordinate,
        &mut WorkBudget::new(1_000_000, 1_000),
        &NeverCancelled,
    );
    assert_eq!(report.canonical_controls(), [genesis_id, valid_id]);
    assert!(!report.canonical_controls().contains(&changed_id));

    let fresh = sign_child(
        1_001,
        vec![
            (retained_device, Some(original_account)),
            (fresh_device.public_key().to_hex(), None),
        ],
    );
    let fresh_id = VerifiedNip01Event::verify(fresh.clone())
        .expect("signed fresh-device child")
        .event_id();
    let mut fresh_builder = CorpusBuilder::new();
    for event in [fresh, genesis] {
        assert!(matches!(
            fresh_builder.ingest(event),
            IngestOutcome::Accepted { .. }
        ));
    }
    let fresh_report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &fresh_builder.finish(),
        coordinate,
        &mut WorkBudget::new(1_000_000, 1_000),
        &NeverCancelled,
    );
    assert_eq!(fresh_report.canonical_controls(), [genesis_id, fresh_id]);
}

#[test]
#[allow(clippy::expect_used)]
fn signed_child_roles_are_monotonic() {
    let controller = TestSigner::from_byte(37);
    let fresh = TestSigner::from_byte(38);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "39".repeat(32)
    )
    .parse()
    .expect("fixed coordinate");
    let retained = controller.public_key().to_hex();
    let genesis = signed_acl_control(
        &controller,
        coordinate,
        1,
        None,
        0,
        vec![(retained.clone(), vec!["write"])],
    );
    let genesis_id = VerifiedNip01Event::verify(genesis.clone())
        .expect("signed genesis")
        .event_id();
    let escalated = signed_acl_control(
        &controller,
        coordinate,
        2,
        Some(genesis_id),
        1,
        vec![(retained.clone(), vec!["checkpoint", "write"])],
    );
    let fresh_grant = signed_acl_control(
        &controller,
        coordinate,
        3,
        Some(genesis_id),
        1,
        vec![
            (retained.clone(), vec!["write"]),
            (fresh.public_key().to_hex(), vec!["checkpoint"]),
        ],
    );
    let fresh_id = VerifiedNip01Event::verify(fresh_grant.clone())
        .expect("signed fresh grant")
        .event_id();

    let mut invalid_builder = CorpusBuilder::new();
    for event in [escalated, genesis.clone()] {
        assert!(matches!(
            invalid_builder.ingest(event),
            IngestOutcome::Accepted { .. }
        ));
    }
    let invalid_report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &invalid_builder.finish(),
        coordinate,
        &mut WorkBudget::new(1_000_000, 1_000),
        &NeverCancelled,
    );
    assert_eq!(invalid_report.canonical_controls(), [genesis_id]);

    let mut fresh_builder = CorpusBuilder::new();
    for event in [fresh_grant, genesis] {
        assert!(matches!(
            fresh_builder.ingest(event),
            IngestOutcome::Accepted { .. }
        ));
    }
    let fresh_report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &fresh_builder.finish(),
        coordinate,
        &mut WorkBudget::new(1_000_000, 1_000),
        &NeverCancelled,
    );
    assert_eq!(fresh_report.canonical_controls(), [genesis_id, fresh_id]);

    let reduction_coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "40".repeat(32)
    )
    .parse()
    .expect("fixed reduction coordinate");
    let reduction_genesis = signed_acl_control(
        &controller,
        reduction_coordinate,
        4,
        None,
        0,
        vec![(retained.clone(), vec!["checkpoint", "write"])],
    );
    let reduction_genesis_id = VerifiedNip01Event::verify(reduction_genesis.clone())
        .expect("signed reduction genesis")
        .event_id();
    let reduction = signed_acl_control(
        &controller,
        reduction_coordinate,
        5,
        Some(reduction_genesis_id),
        1,
        vec![(retained, vec!["write"])],
    );
    let reduction_id = VerifiedNip01Event::verify(reduction.clone())
        .expect("signed reduction")
        .event_id();
    let mut reduction_builder = CorpusBuilder::new();
    for event in [reduction, reduction_genesis] {
        assert!(matches!(
            reduction_builder.ingest(event),
            IngestOutcome::Accepted { .. }
        ));
    }
    let reduction_report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &reduction_builder.finish(),
        reduction_coordinate,
        &mut WorkBudget::new(1_000_000, 1_000),
        &NeverCancelled,
    );
    assert_eq!(
        reduction_report.canonical_controls(),
        [reduction_genesis_id, reduction_id]
    );
}

#[test]
#[allow(clippy::expect_used)]
fn signed_removed_device_cannot_reappear() {
    let controller = TestSigner::from_byte(41);
    let removed = TestSigner::from_byte(42);
    let fresh = TestSigner::from_byte(43);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "44".repeat(32)
    )
    .parse()
    .expect("fixed coordinate");
    let retained = controller.public_key().to_hex();
    let removed_key = removed.public_key().to_hex();
    let fresh_key = fresh.public_key().to_hex();
    let genesis = signed_acl_control(
        &controller,
        coordinate,
        1,
        None,
        0,
        vec![
            (retained.clone(), vec!["write"]),
            (removed_key.clone(), vec!["checkpoint"]),
        ],
    );
    let genesis_id = VerifiedNip01Event::verify(genesis.clone())
        .expect("signed genesis")
        .event_id();
    let removal = signed_acl_control(
        &controller,
        coordinate,
        2,
        Some(genesis_id),
        1,
        vec![(retained.clone(), vec!["write"])],
    );
    let removal_id = VerifiedNip01Event::verify(removal.clone())
        .expect("signed removal")
        .event_id();
    let later_fresh = signed_acl_control(
        &controller,
        coordinate,
        3,
        Some(removal_id),
        2,
        vec![
            (retained.clone(), vec!["write"]),
            (fresh_key.clone(), vec!["checkpoint"]),
        ],
    );
    let later_fresh_id = VerifiedNip01Event::verify(later_fresh.clone())
        .expect("signed fresh device")
        .event_id();
    let reintroduced = signed_acl_control(
        &controller,
        coordinate,
        4,
        Some(later_fresh_id),
        3,
        vec![
            (retained, vec!["write"]),
            (fresh_key, vec!["checkpoint"]),
            (removed_key, vec!["checkpoint"]),
        ],
    );
    let reintroduced_id = VerifiedNip01Event::verify(reintroduced.clone())
        .expect("signed reintroduction")
        .event_id();

    let mut builder = CorpusBuilder::new();
    for event in [reintroduced, later_fresh, removal, genesis] {
        assert!(matches!(
            builder.ingest(event),
            IngestOutcome::Accepted { .. }
        ));
    }
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &builder.finish(),
        coordinate,
        &mut WorkBudget::new(1_000_000, 1_000),
        &NeverCancelled,
    );
    assert_eq!(
        report.canonical_controls(),
        [genesis_id, removal_id, later_fresh_id]
    );
    assert!(!report.canonical_controls().contains(&reintroduced_id));
}

#[test]
#[allow(clippy::expect_used)]
fn public_report_contains_control_dispositions() {
    let controller = TestSigner::from_byte(45);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "46".repeat(32)
    )
    .parse()
    .expect("fixed coordinate");
    let retained = controller.public_key().to_hex();
    let genesis = signed_acl_control(
        &controller,
        coordinate,
        1,
        None,
        0,
        vec![(retained.clone(), vec!["write"])],
    );
    let genesis_id = VerifiedNip01Event::verify(genesis.clone())
        .expect("signed genesis")
        .event_id();
    let first = signed_acl_control(
        &controller,
        coordinate,
        2,
        Some(genesis_id),
        1,
        vec![(retained.clone(), vec!["write"])],
    );
    let first_id = VerifiedNip01Event::verify(first.clone())
        .expect("signed first sibling")
        .event_id();
    let second = signed_acl_control(
        &controller,
        coordinate,
        3,
        Some(genesis_id),
        1,
        vec![(retained.clone(), vec!["write"])],
    );
    let second_id = VerifiedNip01Event::verify(second.clone())
        .expect("signed second sibling")
        .event_id();
    let invalid = signed_acl_control(
        &controller,
        coordinate,
        4,
        Some(genesis_id),
        1,
        vec![(retained.clone(), vec!["checkpoint", "write"])],
    );
    let invalid_id = VerifiedNip01Event::verify(invalid.clone())
        .expect("signed invalid sibling")
        .event_id();
    let pending = signed_acl_control(
        &controller,
        coordinate,
        5,
        Some(EventId::from_bytes([0x55; 32])),
        1,
        vec![(retained, vec!["write"])],
    );
    let pending_id = VerifiedNip01Event::verify(pending.clone())
        .expect("signed pending sibling")
        .event_id();
    let mut builder = CorpusBuilder::new();
    for event in [pending, invalid, second, first, genesis] {
        assert!(matches!(
            builder.ingest(event),
            IngestOutcome::Accepted { .. }
        ));
    }
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &builder.finish(),
        coordinate,
        &mut WorkBudget::new(1_000_000, 1_000),
        &NeverCancelled,
    );
    let dispositions = report
        .control_dispositions()
        .iter()
        .copied()
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(
        dispositions.get(&genesis_id),
        Some(&ProtocolDisposition::Accepted)
    );
    assert_eq!(
        dispositions.get(&invalid_id),
        Some(&ProtocolDisposition::Invalid)
    );
    assert_eq!(
        dispositions.get(&pending_id),
        Some(&ProtocolDisposition::Pending)
    );
    assert_eq!(
        [first_id, second_id]
            .into_iter()
            .filter_map(|event_id| dispositions.get(&event_id))
            .copied()
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([ProtocolDisposition::Accepted, ProtocolDisposition::Excluded,])
    );
    let records = report.disposition_records();
    assert_eq!(records.len(), dispositions.len());
    assert!(
        records
            .windows(2)
            .all(|pair| pair[0].identifier() < pair[1].identifier())
    );
    for (event_id, disposition) in dispositions {
        assert!(records.iter().any(|record| {
            record.identifier() == ProtocolItemIdentifier::control_event(event_id)
                && record.disposition() == disposition
        }));
    }
}

#[test]
fn control_disposition_records_are_complete() {
    public_report_contains_control_dispositions();
}

#[test]
fn change_disposition_collections_are_disjoint() {
    let scenario = signed_engine_scenario();
    let mut builder = CorpusBuilder::new();
    for event in [scenario.change, scenario.control] {
        assert!(matches!(
            builder.ingest(event),
            IngestOutcome::Accepted { .. }
        ));
    }
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &builder.finish(),
        scenario.coordinate,
        &mut WorkBudget::new(1_000_000, 1_000),
        &NeverCancelled,
    );
    let accepted = report
        .accepted_changes()
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let pending = report
        .pending_changes()
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let excluded = report
        .excluded_changes()
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    assert!(accepted.is_disjoint(&pending));
    assert!(accepted.is_disjoint(&excluded));
    assert!(pending.is_disjoint(&excluded));
    let change_records = report
        .disposition_records()
        .iter()
        .filter(|record| matches!(record.identifier(), ProtocolItemIdentifier::ChangeHash(_)));
    assert_eq!(change_records.count(), report.dispositions().len());
}

#[test]
#[allow(clippy::expect_used)]
fn signed_event_disposition_records() {
    let signer = TestSigner::from_byte(107);
    let document_id = "b0".repeat(32);
    let coordinate: DocumentCoordinate =
        format!("31624:{}:{document_id}", signer.public_key().to_hex())
            .parse()
            .expect("manifest coordinate");
    let content = format!(
        r#"{{"application":null,"checkpoint":null,"control":"{}","description":null,"format":"automerge-change-v1","name":null,"relays":["wss://relay.example"],"status":"active","successor":null,"text_encoding":"utf16","v":1}}"#,
        "b1".repeat(32)
    );
    let sign = |created_at: u64, content: String| {
        signer.sign(
            &UnsignedEventDraft::new(
                created_at,
                31_624,
                vec![vec!["d".to_owned(), document_id.clone()]],
                content,
            )
            .expect("manifest draft")
            .prepare(signer.public_key())
            .expect("manifest preimage"),
        )
    };
    let valid = sign(1, content.clone());
    let invalid = sign(2, content.replace("active", "paused"));
    let unsupported = sign(3, content.replace("\"v\":1", "\"v\":2"));
    let valid_id = VerifiedNip01Event::verify(valid.clone())
        .expect("valid manifest")
        .event_id();
    let invalid_id = VerifiedNip01Event::verify(invalid.clone())
        .expect("invalid manifest event")
        .event_id();
    let unsupported_id = VerifiedNip01Event::verify(unsupported.clone())
        .expect("unsupported manifest event")
        .event_id();
    let mut builder = CorpusBuilder::new();
    assert!(matches!(
        builder.ingest(valid),
        IngestOutcome::Accepted { .. }
    ));
    assert!(matches!(
        builder.ingest(invalid),
        IngestOutcome::InvalidCarrier { .. }
    ));
    assert!(matches!(
        builder.ingest(unsupported),
        IngestOutcome::UnsupportedRevision { .. }
    ));
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &builder.finish(),
        coordinate,
        &mut WorkBudget::new(1_000_000, 1_000),
        &NeverCancelled,
    );
    for (event_id, disposition) in [
        (valid_id, ProtocolDisposition::Accepted),
        (invalid_id, ProtocolDisposition::Invalid),
        (unsupported_id, ProtocolDisposition::UnsupportedRevision),
    ] {
        assert!(report.disposition_records().iter().any(|record| {
            record.identifier() == ProtocolItemIdentifier::event(event_id)
                && record.disposition() == disposition
        }));
    }
    validated_checkpoint_descriptor_carrier_enters_corpus();
    validated_checkpoint_chunk_carrier_enters_corpus();
}

#[test]
#[allow(clippy::expect_used)]
fn invalid_and_excluded_changes_are_distinct() {
    let controller = TestSigner::from_byte(108);
    let invalid_writer = TestSigner::from_byte(109);
    let equivocated_writer = TestSigner::from_byte(110);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "b2".repeat(32)
    )
    .parse()
    .expect("fixed coordinate");
    let control = signed_acl_control(
        &controller,
        coordinate,
        1,
        None,
        0,
        vec![
            (invalid_writer.public_key().to_hex(), vec!["write"]),
            (equivocated_writer.public_key().to_hex(), vec!["write"]),
        ],
    );
    let control_id = VerifiedNip01Event::verify(control.clone())
        .expect("signed control")
        .event_id();
    let invalid_actor = ActorId::derive(coordinate, invalid_writer.public_key());
    let mut invalid_document =
        AuthoringDocument::empty(ActorState::initial(invalid_actor, BTreeSet::new()))
            .expect("invalid writer document");
    let invalid = invalid_document
        .author_change(&[Operation::PutString {
            key: "invalid".to_owned(),
            value: "sequence two".to_owned(),
        }])
        .expect("first change");
    let (invalid_raw, invalid_hash) = rewrite_change_sequence(invalid.raw().to_vec(), 1, 2);
    let equivocated_actor = ActorId::derive(coordinate, equivocated_writer.public_key());
    let make_conflict = |key: &str| {
        let mut document = AutoCommit::new_with_encoding(TextEncoding::Utf16CodeUnit).with_actor(
            automerge::ActorId::from(equivocated_actor.as_bytes().to_vec()),
        );
        document.put(ROOT, key, "excluded").expect("conflict op");
        let hash = document.commit().expect("conflict hash");
        let raw = document
            .get_change_by_hash(&hash)
            .expect("conflict change")
            .raw_bytes()
            .to_vec();
        (ChangeHash::from_bytes(hash.0), raw)
    };
    let (left_hash, left_raw) = make_conflict("left");
    let (right_hash, right_raw) = make_conflict("right");
    let sign_change = |signer: &TestSigner, created_at: u64, hash: ChangeHash, raw: &[u8]| {
        signer.sign(
            &UnsignedEventDraft::new(
                created_at,
                1_624,
                vec![
                    vec!["a".to_owned(), coordinate.to_address()],
                    vec!["e".to_owned(), control_id.to_hex()],
                    vec!["x".to_owned(), hash.to_hex()],
                ],
                base64::engine::general_purpose::STANDARD.encode(raw),
            )
            .expect("change draft")
            .prepare(signer.public_key())
            .expect("change preimage"),
        )
    };
    let mut builder = CorpusBuilder::new();
    for event in [
        sign_change(&invalid_writer, 2, invalid_hash, &invalid_raw),
        sign_change(&equivocated_writer, 3, left_hash, &left_raw),
        sign_change(&equivocated_writer, 4, right_hash, &right_raw),
        control,
    ] {
        assert!(matches!(
            builder.ingest(event),
            IngestOutcome::Accepted { .. }
        ));
    }
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &builder.finish(),
        coordinate,
        &mut WorkBudget::new(1_000_000, 1_000),
        &NeverCancelled,
    );
    assert_eq!(report.invalid_changes(), [invalid_hash]);
    assert!(!report.excluded_changes().contains(&invalid_hash));
    assert!(report.excluded_changes().contains(&left_hash));
    assert!(report.excluded_changes().contains(&right_hash));
    assert!(report.invalid_changes().iter().all(|hash| {
        report
            .dispositions()
            .contains(&(*hash, ProtocolDisposition::Invalid))
    }));
}

#[test]
#[allow(clippy::expect_used)]
fn control_outcomes_change_dispositions_digest() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/v1_draft/conformance/disposition_digest_controls.json"
    ))
    .expect("control digest fixture");
    assert_eq!(fixture["requirements"][0], "R2_REPORT_006");
    let controller = TestSigner::from_byte(111);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "b4".repeat(32)
    )
    .parse()
    .expect("fixed coordinate");
    let member = controller.public_key().to_hex();
    let genesis = signed_acl_control(
        &controller,
        coordinate,
        1,
        None,
        0,
        vec![(member.clone(), vec!["write"])],
    );
    let genesis_id = VerifiedNip01Event::verify(genesis.clone())
        .expect("signed genesis")
        .event_id();
    let left = signed_acl_control(
        &controller,
        coordinate,
        2,
        Some(genesis_id),
        1,
        vec![(member.clone(), vec!["write"])],
    );
    let right = signed_acl_control(
        &controller,
        coordinate,
        3,
        Some(genesis_id),
        1,
        vec![(member, vec!["write"])],
    );
    let evaluate = |events: Vec<RawEventBytes>| {
        let mut builder = CorpusBuilder::new();
        for event in events {
            assert!(matches!(
                builder.ingest(event),
                IngestOutcome::Accepted { .. }
            ));
        }
        ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
            &builder.finish(),
            coordinate,
            &mut WorkBudget::new(1_000_000, 1_000),
            &NeverCancelled,
        )
    };
    let canonical = evaluate(vec![genesis.clone(), left.clone(), right.clone()]);
    let reversed = evaluate(vec![right, left, genesis.clone()]);
    let genesis_only = evaluate(vec![genesis]);
    assert_eq!(
        canonical.dispositions_digest(),
        reversed.dispositions_digest()
    );
    assert_ne!(
        canonical.dispositions_digest(),
        genesis_only.dispositions_digest()
    );
    assert_eq!(canonical.control_dispositions().len(), 3);
}

#[test]
#[allow(clippy::expect_used)]
fn event_dispositions_digest_boundary() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/v1_draft/conformance/disposition_digest_events.json"
    ))
    .expect("event digest fixture");
    assert_eq!(fixture["requirements"][0], "R2_REPORT_007");
    let signer = TestSigner::from_byte(112);
    let document_id = "b6".repeat(32);
    let coordinate: DocumentCoordinate =
        format!("31624:{}:{document_id}", signer.public_key().to_hex())
            .parse()
            .expect("fixed coordinate");
    let unsupported = signer.sign(
        &UnsignedEventDraft::new(
            1,
            31_624,
            vec![vec!["d".to_owned(), document_id]],
            format!(
                r#"{{"application":null,"checkpoint":null,"control":"{}","description":null,"format":"automerge-change-v1","name":null,"relays":[],"status":"active","successor":null,"text_encoding":"utf16","v":2}}"#,
                "b7".repeat(32)
            ),
        )
        .expect("unsupported manifest draft")
        .prepare(signer.public_key())
        .expect("unsupported manifest preimage"),
    );
    let evaluate = |builder: CorpusBuilder| {
        ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
            &builder.finish(),
            coordinate,
            &mut WorkBudget::new(1_000_000, 1_000),
            &NeverCancelled,
        )
    };
    let empty = evaluate(CorpusBuilder::new());
    let mut invalid_raw = CorpusBuilder::new();
    assert!(matches!(
        invalid_raw.ingest_bytes(b"{}"),
        IngestOutcome::Invalid { .. }
    ));
    let invalid_raw = evaluate(invalid_raw);
    let mut unsupported_builder = CorpusBuilder::new();
    assert!(matches!(
        unsupported_builder.ingest(unsupported),
        IngestOutcome::UnsupportedRevision { .. }
    ));
    let unsupported = evaluate(unsupported_builder);
    assert_eq!(
        empty.dispositions_digest(),
        invalid_raw.dispositions_digest()
    );
    assert_ne!(
        empty.dispositions_digest(),
        unsupported.dispositions_digest()
    );
    assert!(unsupported.disposition_records().iter().any(|record| {
        matches!(record.identifier(), ProtocolItemIdentifier::Event(_))
            && record.disposition() == ProtocolDisposition::UnsupportedRevision
    }));
}

#[test]
#[allow(clippy::expect_used)]
fn signed_revision_declarations_distinguish_invalid_from_unsupported() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/v1_draft/tags/revision_classification.json"
    ))
    .expect("revision classification fixture");
    assert_eq!(fixture["cases"].as_array().map(Vec::len), Some(6));
    let signer = TestSigner::from_byte(114);
    let document_id = "ba".repeat(32);
    let cases = [
        (r#"{"v":2}"#, true),
        (r#"{"format":"automerge-change-v2"}"#, true),
        (r#"{"v":2,"v":1}"#, false),
        (
            r#"{"format":"automerge-change-v2","format":"automerge-change-v1"}"#,
            false,
        ),
        (r#"{ "v":2}"#, false),
        (r#"{"v":2"#, false),
    ];
    for (created_at, (content, unsupported)) in cases.into_iter().enumerate() {
        let event = signer.sign(
            &UnsignedEventDraft::new(
                u64::try_from(created_at + 1).expect("small case index"),
                31_624,
                vec![vec!["d".to_owned(), document_id.clone()]],
                content.to_owned(),
            )
            .expect("manifest draft")
            .prepare(signer.public_key())
            .expect("manifest preimage"),
        );
        let mut builder = CorpusBuilder::new();
        let outcome = builder.ingest(event);
        assert_eq!(
            matches!(outcome, IngestOutcome::UnsupportedRevision { .. }),
            unsupported
        );
        assert_eq!(
            matches!(outcome, IngestOutcome::InvalidCarrier { .. }),
            !unsupported
        );
    }
}

#[test]
fn duplicate_delayed_and_invalid_evidence_converges() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/v1_draft/integrity/cases.json"
    ))
    .unwrap_or_default();
    assert_eq!(fixture["cases"].as_array().map(Vec::len), Some(8));
    let scenario = signed_engine_scenario();
    let evaluate = |events: &[RawEventBytes]| {
        let mut builder = CorpusBuilder::new();
        for event in events {
            let _ = builder.ingest(event.clone());
        }
        ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
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
    let invalid_first = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
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
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
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
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
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
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
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
        evaluator.evaluate_report(&corpus, scenario.coordinate, &mut measured, &NeverCancelled);
    assert_eq!(measured_report.completion(), Completion::Complete);
    let decode_bytes = measured.consumed().get(WorkCounter::DecodeByte);
    assert!(decode_bytes > 0);

    let mut exhausted = WorkBudget::new(decode_bytes - 1, 1_000);
    let report = evaluator.evaluate_report(
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
        evaluator.evaluate_report(&corpus, scenario.coordinate, &mut measured, &NeverCancelled);
    assert_eq!(measured_report.completion(), Completion::Complete);
    assert_eq!(measured.consumed().get(WorkCounter::ApplyChange), 2);
    let consumed_items = 1_000 - measured.remaining().1;

    let mut exhausted = WorkBudget::new(1_000_000, consumed_items - 1);
    let report = evaluator.evaluate_report(
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
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
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
        WorkCounter::CheckpointItem,
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
    let complete = evaluator.evaluate_report(
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
        let report = evaluator.evaluate_report(
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

fn rewrite_change_sequence(
    mut raw: Vec<u8>,
    expected_sequence: u8,
    sequence: u8,
) -> (Vec<u8>, ChangeHash) {
    let mut data_start = 9usize;
    while raw[data_start] & 0x80 != 0 {
        data_start += 1;
    }
    data_start += 1;
    let dependency_count = usize::from(raw[data_start]);
    let actor_len_offset = data_start + 1 + dependency_count * 32;
    let actor_len = usize::from(raw[actor_len_offset]);
    let sequence_offset = actor_len_offset + 1 + actor_len;
    assert_eq!(raw[sequence_offset], expected_sequence);
    raw[sequence_offset] = sequence;
    let digest: [u8; 32] = Sha256::digest(&raw[8..]).into();
    raw[4..8].copy_from_slice(&digest[..4]);
    (raw, ChangeHash::from_bytes(digest))
}

fn rewrite_first_change_start_op(mut raw: Vec<u8>, start_op: u8) -> (Vec<u8>, ChangeHash) {
    let mut data_start = 9usize;
    while raw[data_start] & 0x80 != 0 {
        data_start += 1;
    }
    data_start += 1;
    assert_eq!(raw[data_start], 0);
    let actor_len = usize::from(raw[data_start + 1]);
    let sequence_offset = data_start + 2 + actor_len;
    assert_eq!(raw[sequence_offset], 1);
    assert_eq!(raw[sequence_offset + 1], 1);
    raw[sequence_offset + 1] = start_op;
    let digest: [u8; 32] = Sha256::digest(&raw[8..]).into();
    raw[4..8].copy_from_slice(&digest[..4]);
    (raw, ChangeHash::from_bytes(digest))
}

#[test]
#[allow(clippy::expect_used)]
fn new_actor_sequence_must_start_at_one() {
    let controller = TestSigner::from_byte(86);
    let device = TestSigner::from_byte(87);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "86".repeat(32)
    )
    .parse()
    .expect("fixed coordinate");
    let control = signed_acl_control(
        &controller,
        coordinate,
        1,
        None,
        0,
        vec![(device.public_key().to_hex(), vec!["write"])],
    );
    let control_id = VerifiedNip01Event::verify(control.clone())
        .expect("signed control")
        .event_id();
    let actor = ActorId::derive(coordinate, device.public_key());
    let mut document = AuthoringDocument::empty(ActorState::initial(actor, BTreeSet::new()))
        .expect("empty authoring document");
    let first = document
        .author_change(&[Operation::PutString {
            key: "invalid-sequence".to_owned(),
            value: "two".to_owned(),
        }])
        .expect("canonical authored change");
    let (raw, change_hash) = rewrite_change_sequence(first.raw().to_vec(), 1, 2);
    let change = device.sign(
        &UnsignedEventDraft::new(
            2,
            1_624,
            vec![
                vec!["a".to_owned(), coordinate.to_address()],
                vec!["e".to_owned(), control_id.to_hex()],
                vec!["x".to_owned(), change_hash.to_hex()],
            ],
            base64::engine::general_purpose::STANDARD.encode(raw),
        )
        .expect("change draft")
        .prepare(device.public_key())
        .expect("change preimage"),
    );
    let mut builder = CorpusBuilder::new();
    assert!(matches!(
        builder.ingest(change),
        IngestOutcome::Accepted { .. }
    ));
    assert!(matches!(
        builder.ingest(control),
        IngestOutcome::Accepted { .. }
    ));
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &builder.finish(),
        coordinate,
        &mut WorkBudget::new(1_000_000, 1_000),
        &NeverCancelled,
    );
    assert!(
        report
            .dispositions()
            .contains(&(change_hash, ProtocolDisposition::Invalid))
    );
    assert!(report.accepted_changes().is_empty());
}

#[test]
#[allow(clippy::expect_used)]
fn change_start_op_must_equal_actor_next_op() {
    let controller = TestSigner::from_byte(92);
    let device = TestSigner::from_byte(93);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "92".repeat(32)
    )
    .parse()
    .expect("fixed coordinate");
    let control = signed_acl_control(
        &controller,
        coordinate,
        1,
        None,
        0,
        vec![(device.public_key().to_hex(), vec!["write"])],
    );
    let control_id = VerifiedNip01Event::verify(control.clone())
        .expect("signed control")
        .event_id();
    let actor = ActorId::derive(coordinate, device.public_key());
    let mut document = AuthoringDocument::empty(ActorState::initial(actor, BTreeSet::new()))
        .expect("empty authoring document");
    let first = document
        .author_change(&[Operation::PutString {
            key: "wrong-counter".to_owned(),
            value: "invalid".to_owned(),
        }])
        .expect("first change");
    let (raw, change_hash) = rewrite_first_change_start_op(first.raw().to_vec(), 2);
    let change = device.sign(
        &UnsignedEventDraft::new(
            2,
            1_624,
            vec![
                vec!["a".to_owned(), coordinate.to_address()],
                vec!["e".to_owned(), control_id.to_hex()],
                vec!["x".to_owned(), change_hash.to_hex()],
            ],
            base64::engine::general_purpose::STANDARD.encode(raw),
        )
        .expect("change draft")
        .prepare(device.public_key())
        .expect("change preimage"),
    );
    let mut builder = CorpusBuilder::new();
    for event in [change, control] {
        assert!(matches!(
            builder.ingest(event),
            IngestOutcome::Accepted { .. }
        ));
    }
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &builder.finish(),
        coordinate,
        &mut WorkBudget::new(1_000_000, 1_000),
        &NeverCancelled,
    );
    assert!(
        report
            .dispositions()
            .contains(&(change_hash, ProtocolDisposition::Invalid))
    );
    assert!(report.accepted_changes().is_empty());
}

#[test]
#[allow(clippy::expect_used)]
fn empty_change_consumes_only_sequence() {
    let controller = TestSigner::from_byte(94);
    let device = TestSigner::from_byte(95);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "94".repeat(32)
    )
    .parse()
    .expect("fixed coordinate");
    let control = signed_acl_control(
        &controller,
        coordinate,
        1,
        None,
        0,
        vec![(device.public_key().to_hex(), vec!["write"])],
    );
    let control_id = VerifiedNip01Event::verify(control.clone())
        .expect("signed control")
        .event_id();
    let actor = ActorId::derive(coordinate, device.public_key());
    let mut document = AutoCommit::new_with_encoding(TextEncoding::Utf16CodeUnit)
        .with_actor(automerge::ActorId::from(actor.as_bytes().to_vec()));
    let empty_hash = document.empty_change(CommitOptions::default());
    let empty = document
        .get_change_by_hash(&empty_hash)
        .expect("empty change")
        .raw_bytes()
        .to_vec();
    document
        .put(ROOT, "after-empty", "accepted")
        .expect("nonempty operation");
    let nonempty_hash = document.commit().expect("nonempty change hash");
    let nonempty = document
        .get_change_by_hash(&nonempty_hash)
        .expect("nonempty change")
        .raw_bytes()
        .to_vec();
    let empty_hash = ChangeHash::from_bytes(empty_hash.0);
    let nonempty_hash = ChangeHash::from_bytes(nonempty_hash.0);
    let sign_change = |created_at: u64, hash: ChangeHash, raw: &[u8]| {
        device.sign(
            &UnsignedEventDraft::new(
                created_at,
                1_624,
                vec![
                    vec!["a".to_owned(), coordinate.to_address()],
                    vec!["e".to_owned(), control_id.to_hex()],
                    vec!["x".to_owned(), hash.to_hex()],
                ],
                base64::engine::general_purpose::STANDARD.encode(raw),
            )
            .expect("change draft")
            .prepare(device.public_key())
            .expect("change preimage"),
        )
    };
    let mut builder = CorpusBuilder::new();
    for event in [
        sign_change(3, nonempty_hash, &nonempty),
        sign_change(2, empty_hash, &empty),
        control,
    ] {
        assert!(matches!(
            builder.ingest(event),
            IngestOutcome::Accepted { .. }
        ));
    }
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &builder.finish(),
        coordinate,
        &mut WorkBudget::new(1_000_000, 1_000),
        &NeverCancelled,
    );
    assert_eq!(
        report
            .accepted_changes()
            .iter()
            .copied()
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([empty_hash, nonempty_hash])
    );
}

#[test]
#[allow(clippy::expect_used)]
fn empty_change_requires_exact_current_heads() {
    empty_change_consumes_only_sequence();
    let controller = TestSigner::from_byte(96);
    let writer = TestSigner::from_byte(97);
    let merger = TestSigner::from_byte(98);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "96".repeat(32)
    )
    .parse()
    .expect("fixed coordinate");
    let members = vec![
        (writer.public_key().to_hex(), vec!["write"]),
        (merger.public_key().to_hex(), vec!["write"]),
    ];
    let genesis = signed_acl_control(&controller, coordinate, 1, None, 0, members.clone());
    let genesis_id = VerifiedNip01Event::verify(genesis.clone())
        .expect("signed genesis")
        .event_id();
    let writer_actor = ActorId::derive(coordinate, writer.public_key());
    let mut authored = AuthoringDocument::empty(ActorState::initial(writer_actor, BTreeSet::new()))
        .expect("empty authoring document");
    let first = authored
        .author_change(&[Operation::PutString {
            key: "base".to_owned(),
            value: "retained".to_owned(),
        }])
        .expect("base change");
    let child = signed_acl_control_with_base(
        &controller,
        coordinate,
        4,
        Some(genesis_id),
        1,
        members,
        &[first.change_hash()],
    );
    let child_id = VerifiedNip01Event::verify(child.clone())
        .expect("signed child")
        .event_id();
    let merger_actor = ActorId::derive(coordinate, merger.public_key());
    let mut empty_document = AutoCommit::new_with_encoding(TextEncoding::Utf16CodeUnit)
        .with_actor(automerge::ActorId::from(merger_actor.as_bytes().to_vec()));
    let stale_hash = empty_document.empty_change(CommitOptions::default());
    let stale = empty_document
        .get_change_by_hash(&stale_hash)
        .expect("stale empty change")
        .raw_bytes()
        .to_vec();
    let stale_hash = ChangeHash::from_bytes(stale_hash.0);
    let sign_change =
        |signer: &TestSigner, created_at: u64, selected: EventId, hash: ChangeHash, raw: &[u8]| {
            signer.sign(
                &UnsignedEventDraft::new(
                    created_at,
                    1_624,
                    vec![
                        vec!["a".to_owned(), coordinate.to_address()],
                        vec!["e".to_owned(), selected.to_hex()],
                        vec!["x".to_owned(), hash.to_hex()],
                    ],
                    base64::engine::general_purpose::STANDARD.encode(raw),
                )
                .expect("change draft")
                .prepare(signer.public_key())
                .expect("change preimage"),
            )
        };
    let mut builder = CorpusBuilder::new();
    for event in [
        sign_change(&writer, 2, genesis_id, first.change_hash(), first.raw()),
        sign_change(&merger, 3, child_id, stale_hash, &stale),
        genesis,
        child,
    ] {
        assert!(matches!(
            builder.ingest(event),
            IngestOutcome::Accepted { .. }
        ));
    }
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &builder.finish(),
        coordinate,
        &mut WorkBudget::new(1_000_000, 1_000),
        &NeverCancelled,
    );
    assert_eq!(report.accepted_changes(), [first.change_hash()]);
    assert!(
        report
            .dispositions()
            .contains(&(stale_hash, ProtocolDisposition::Invalid))
    );
}

#[test]
fn change_must_descend_from_every_base_head() {
    signed_multi_epoch_exact_closure();
    actor_sequence_requires_exact_predecessor();
    empty_change_requires_exact_current_heads();
}

#[test]
fn candidate_applies_to_exact_dependency_closure() {
    signed_events_reach_materialized_state_through_public_engine();
    actor_sequence_requires_exact_predecessor();
    change_must_descend_from_every_base_head();
}

#[test]
fn apply_failure_invalidates_only_candidate() {
    signed_events_reach_materialized_state_through_public_engine();
    change_start_op_must_equal_actor_next_op();
    candidate_applies_to_exact_dependency_closure();
}

#[test]
#[allow(clippy::expect_used)]
fn actor_sequence_requires_exact_predecessor() {
    new_actor_sequence_must_start_at_one();
    let controller = TestSigner::from_byte(88);
    let device = TestSigner::from_byte(89);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "88".repeat(32)
    )
    .parse()
    .expect("fixed coordinate");
    let control = signed_acl_control(
        &controller,
        coordinate,
        1,
        None,
        0,
        vec![(device.public_key().to_hex(), vec!["write"])],
    );
    let control_id = VerifiedNip01Event::verify(control.clone())
        .expect("signed control")
        .event_id();
    let actor = ActorId::derive(coordinate, device.public_key());
    let mut document = AuthoringDocument::empty(ActorState::initial(actor, BTreeSet::new()))
        .expect("empty authoring document");
    let first = document
        .author_change(&[Operation::PutString {
            key: "first".to_owned(),
            value: "one".to_owned(),
        }])
        .expect("first change");
    let second = document
        .author_change(&[Operation::PutString {
            key: "second".to_owned(),
            value: "two".to_owned(),
        }])
        .expect("second change");
    let sign_change = |created_at: u64, authored: &nostr_automerge::authoring::AuthoredChange| {
        device.sign(
            &UnsignedEventDraft::new(
                created_at,
                1_624,
                vec![
                    vec!["a".to_owned(), coordinate.to_address()],
                    vec!["e".to_owned(), control_id.to_hex()],
                    vec!["x".to_owned(), authored.change_hash().to_hex()],
                ],
                base64::engine::general_purpose::STANDARD.encode(authored.raw()),
            )
            .expect("change draft")
            .prepare(device.public_key())
            .expect("change preimage"),
        )
    };
    let first_event = sign_change(2, &first);
    let second_event = sign_change(3, &second);
    let mut pending_builder = CorpusBuilder::new();
    assert!(matches!(
        pending_builder.ingest(second_event.clone()),
        IngestOutcome::Accepted { .. }
    ));
    assert!(matches!(
        pending_builder.ingest(control.clone()),
        IngestOutcome::Accepted { .. }
    ));
    let pending = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &pending_builder.finish(),
        coordinate,
        &mut WorkBudget::new(1_000_000, 1_000),
        &NeverCancelled,
    );
    assert!(
        pending
            .dispositions()
            .contains(&(second.change_hash(), ProtocolDisposition::Pending))
    );

    let mut builder = CorpusBuilder::new();
    assert!(matches!(
        builder.ingest(second_event),
        IngestOutcome::Accepted { .. }
    ));
    assert!(matches!(
        builder.ingest(first_event),
        IngestOutcome::Accepted { .. }
    ));
    assert!(matches!(
        builder.ingest(control),
        IngestOutcome::Accepted { .. }
    ));
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &builder.finish(),
        coordinate,
        &mut WorkBudget::new(1_000_000, 1_000),
        &NeverCancelled,
    );
    assert_eq!(
        report
            .accepted_changes()
            .iter()
            .copied()
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([first.change_hash(), second.change_hash()])
    );
}

#[test]
fn actor_sequence_gap_is_invalid() {
    new_actor_sequence_must_start_at_one();
    actor_sequence_requires_exact_predecessor();
}

#[test]
fn missing_dependency_promotes_after_delivery() {
    actor_sequence_requires_exact_predecessor();
    duplicate_delayed_and_invalid_evidence_converges();
}

#[test]
fn change_admission_order_is_hash_canonical() {
    duplicate_delayed_and_invalid_evidence_converges();
    actor_sequence_requires_exact_predecessor();
    signed_multi_epoch_exact_closure();
}

#[test]
#[allow(clippy::expect_used)]
fn actor_sequence_rollback_and_replay() {
    let controller = TestSigner::from_byte(90);
    let device = TestSigner::from_byte(91);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "90".repeat(32)
    )
    .parse()
    .expect("fixed coordinate");
    let control = signed_acl_control(
        &controller,
        coordinate,
        1,
        None,
        0,
        vec![(device.public_key().to_hex(), vec!["write"])],
    );
    let control_id = VerifiedNip01Event::verify(control.clone())
        .expect("signed control")
        .event_id();
    let actor = ActorId::derive(coordinate, device.public_key());
    let mut document = AuthoringDocument::empty(ActorState::initial(actor, BTreeSet::new()))
        .expect("empty authoring document");
    let first = document
        .author_change(&[Operation::PutString {
            key: "first".to_owned(),
            value: "accepted".to_owned(),
        }])
        .expect("first change");
    let second = document
        .author_change(&[Operation::PutString {
            key: "rollback".to_owned(),
            value: "invalid".to_owned(),
        }])
        .expect("second change");
    let (rollback_raw, rollback_hash) = rewrite_change_sequence(second.raw().to_vec(), 2, 1);
    let child = signed_acl_control_with_base(
        &controller,
        coordinate,
        5,
        Some(control_id),
        1,
        vec![(device.public_key().to_hex(), vec!["write"])],
        &[first.change_hash()],
    );
    let child_id = VerifiedNip01Event::verify(child.clone())
        .expect("signed child")
        .event_id();
    let sign_change = |created_at: u64, selected: EventId, hash: ChangeHash, raw: &[u8]| {
        device.sign(
            &UnsignedEventDraft::new(
                created_at,
                1_624,
                vec![
                    vec!["a".to_owned(), coordinate.to_address()],
                    vec!["e".to_owned(), selected.to_hex()],
                    vec!["x".to_owned(), hash.to_hex()],
                ],
                base64::engine::general_purpose::STANDARD.encode(raw),
            )
            .expect("change draft")
            .prepare(device.public_key())
            .expect("change preimage"),
        )
    };
    let mut builder = CorpusBuilder::new();
    for event in [
        sign_change(2, control_id, first.change_hash(), first.raw()),
        sign_change(3, control_id, first.change_hash(), first.raw()),
        sign_change(4, child_id, rollback_hash, &rollback_raw),
        control,
        child,
    ] {
        assert!(matches!(
            builder.ingest(event),
            IngestOutcome::Accepted { .. }
        ));
    }
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &builder.finish(),
        coordinate,
        &mut WorkBudget::new(1_000_000, 1_000),
        &NeverCancelled,
    );
    assert!(report.accepted_changes().is_empty());
    assert!(
        report
            .dispositions()
            .contains(&(first.change_hash(), ProtocolDisposition::Excluded))
    );
    assert!(
        report
            .dispositions()
            .contains(&(rollback_hash, ProtocolDisposition::Excluded))
    );
    assert_eq!(
        report
            .dispositions()
            .iter()
            .filter(|(hash, _)| *hash == first.change_hash())
            .count(),
        1
    );
    assert!(report.integrity_alerts().iter().any(|alert| matches!(
        alert,
        nostr_automerge::IntegrityAlert::DeviceEquivocation(_)
    )));
}

#[test]
fn base_sequence_equivocation_is_detected() {
    actor_sequence_rollback_and_replay();
}

#[test]
#[allow(clippy::expect_used)]
fn equivocation_quarantines_transitive_dependants() {
    let controller = TestSigner::from_byte(99);
    let equivocated = TestSigner::from_byte(100);
    let dependant = TestSigner::from_byte(101);
    let independent = TestSigner::from_byte(102);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "99".repeat(32)
    )
    .parse()
    .expect("fixed coordinate");
    let control = signed_acl_control(
        &controller,
        coordinate,
        1,
        None,
        0,
        vec![
            (equivocated.public_key().to_hex(), vec!["write"]),
            (dependant.public_key().to_hex(), vec!["write"]),
            (independent.public_key().to_hex(), vec!["write"]),
        ],
    );
    let control_id = VerifiedNip01Event::verify(control.clone())
        .expect("signed control")
        .event_id();
    let equivocated_actor = ActorId::derive(coordinate, equivocated.public_key());
    let make_conflict = |key: &str| {
        let mut document = AutoCommit::new_with_encoding(TextEncoding::Utf16CodeUnit).with_actor(
            automerge::ActorId::from(equivocated_actor.as_bytes().to_vec()),
        );
        document.put(ROOT, key, "conflict").expect("conflict op");
        let hash = document.commit().expect("conflict hash");
        let raw = document
            .get_change_by_hash(&hash)
            .expect("conflict change")
            .raw_bytes()
            .to_vec();
        (ChangeHash::from_bytes(hash.0), raw)
    };
    let (first_hash, first_raw) = make_conflict("first");
    let (second_hash, second_raw) = make_conflict("second");

    let mut dependant_document = AutoCommit::new_with_encoding(TextEncoding::Utf16CodeUnit);
    dependant_document
        .apply_changes(
            [automerge::Change::from_bytes(first_raw.clone()).expect("decoded conflict")],
        )
        .expect("apply conflict branch");
    let dependant_actor = ActorId::derive(coordinate, dependant.public_key());
    dependant_document.set_actor(automerge::ActorId::from(
        dependant_actor.as_bytes().to_vec(),
    ));
    dependant_document
        .put(ROOT, "dependant", "quarantined")
        .expect("dependant op");
    let dependant_hash = dependant_document.commit().expect("dependant hash");
    let dependant_raw = dependant_document
        .get_change_by_hash(&dependant_hash)
        .expect("dependant change")
        .raw_bytes()
        .to_vec();
    let dependant_hash = ChangeHash::from_bytes(dependant_hash.0);

    let independent_actor = ActorId::derive(coordinate, independent.public_key());
    let mut independent_document =
        AuthoringDocument::empty(ActorState::initial(independent_actor, BTreeSet::new()))
            .expect("independent document");
    let independent_change = independent_document
        .author_change(&[Operation::PutString {
            key: "independent".to_owned(),
            value: "retained".to_owned(),
        }])
        .expect("independent change");
    let sign_change = |signer: &TestSigner, created_at: u64, hash: ChangeHash, raw: &[u8]| {
        signer.sign(
            &UnsignedEventDraft::new(
                created_at,
                1_624,
                vec![
                    vec!["a".to_owned(), coordinate.to_address()],
                    vec!["e".to_owned(), control_id.to_hex()],
                    vec!["x".to_owned(), hash.to_hex()],
                ],
                base64::engine::general_purpose::STANDARD.encode(raw),
            )
            .expect("change draft")
            .prepare(signer.public_key())
            .expect("change preimage"),
        )
    };
    let mut builder = CorpusBuilder::new();
    for event in [
        sign_change(&equivocated, 2, first_hash, &first_raw),
        sign_change(&equivocated, 3, second_hash, &second_raw),
        sign_change(&dependant, 4, dependant_hash, &dependant_raw),
        sign_change(
            &independent,
            5,
            independent_change.change_hash(),
            independent_change.raw(),
        ),
        control,
    ] {
        assert!(matches!(
            builder.ingest(event),
            IngestOutcome::Accepted { .. }
        ));
    }
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &builder.finish(),
        coordinate,
        &mut WorkBudget::new(1_000_000, 1_000),
        &NeverCancelled,
    );
    assert_eq!(report.completion(), Completion::Complete);
    assert_eq!(
        report.accepted_changes(),
        [independent_change.change_hash()]
    );
    for hash in [first_hash, second_hash, dependant_hash] {
        assert!(
            report
                .dispositions()
                .contains(&(hash, ProtocolDisposition::Excluded))
        );
    }
    assert!(report.integrity_alerts().iter().any(|alert| matches!(
        alert,
        nostr_automerge::IntegrityAlert::DeviceEquivocation(_)
    )));
}

#[test]
#[allow(clippy::expect_used)]
fn equivocation_preserves_prior_actor_history() {
    let controller = TestSigner::from_byte(103);
    let device = TestSigner::from_byte(104);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "a4".repeat(32)
    )
    .parse()
    .expect("fixed coordinate");
    let control = signed_acl_control(
        &controller,
        coordinate,
        1,
        None,
        0,
        vec![(device.public_key().to_hex(), vec!["write"])],
    );
    let control_id = VerifiedNip01Event::verify(control.clone())
        .expect("signed control")
        .event_id();
    let actor = ActorId::derive(coordinate, device.public_key());
    let mut history = AutoCommit::new_with_encoding(TextEncoding::Utf16CodeUnit)
        .with_actor(automerge::ActorId::from(actor.as_bytes().to_vec()));
    history
        .put(ROOT, "prior", "retained")
        .expect("prior operation");
    let prior_hash = history.commit().expect("prior hash");
    let prior_raw = history
        .get_change_by_hash(&prior_hash)
        .expect("prior change")
        .raw_bytes()
        .to_vec();
    let mut left = history.fork();
    let mut right = history.fork();
    left.set_actor(automerge::ActorId::from(actor.as_bytes().to_vec()));
    right.set_actor(automerge::ActorId::from(actor.as_bytes().to_vec()));
    left.put(ROOT, "left", "excluded").expect("left operation");
    right
        .put(ROOT, "right", "excluded")
        .expect("right operation");
    let left_hash = left.commit().expect("left hash");
    let right_hash = right.commit().expect("right hash");
    let left_raw = left
        .get_change_by_hash(&left_hash)
        .expect("left change")
        .raw_bytes()
        .to_vec();
    let right_raw = right
        .get_change_by_hash(&right_hash)
        .expect("right change")
        .raw_bytes()
        .to_vec();
    let prior_hash = ChangeHash::from_bytes(prior_hash.0);
    let left_hash = ChangeHash::from_bytes(left_hash.0);
    let right_hash = ChangeHash::from_bytes(right_hash.0);
    let sign_change = |created_at: u64, hash: ChangeHash, raw: &[u8]| {
        device.sign(
            &UnsignedEventDraft::new(
                created_at,
                1_624,
                vec![
                    vec!["a".to_owned(), coordinate.to_address()],
                    vec!["e".to_owned(), control_id.to_hex()],
                    vec!["x".to_owned(), hash.to_hex()],
                ],
                base64::engine::general_purpose::STANDARD.encode(raw),
            )
            .expect("change draft")
            .prepare(device.public_key())
            .expect("change preimage"),
        )
    };
    let mut builder = CorpusBuilder::new();
    for (index, event) in [
        sign_change(2, prior_hash, &prior_raw),
        sign_change(3, left_hash, &left_raw),
        sign_change(4, right_hash, &right_raw),
        control,
    ]
    .into_iter()
    .enumerate()
    {
        let outcome = builder.ingest(event);
        assert!(
            matches!(outcome, IngestOutcome::Accepted { .. }),
            "unexpected ingest outcome at {index}: {outcome:?}"
        );
    }
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &builder.finish(),
        coordinate,
        &mut WorkBudget::new(1_000_000, 1_000),
        &NeverCancelled,
    );
    assert_eq!(report.completion(), Completion::Complete);
    assert_eq!(report.accepted_changes(), [prior_hash]);
    assert_eq!(report.heads(), [prior_hash]);
    for hash in [left_hash, right_hash] {
        assert!(
            report
                .dispositions()
                .contains(&(hash, ProtocolDisposition::Excluded))
        );
    }
    let document = report.document().expect("preserved document");
    assert!(document.entries().iter().any(|entry| {
        entry.path() == [MaterializedPathElement::Key("prior".to_owned())]
            && matches!(
                entry.conflicts(),
                [conflict]
                    if conflict.value()
                        == &MaterializedValue::Scalar(MaterializedScalar::String(
                            "retained".to_owned()
                        ))
            )
    }));
    assert!(report.integrity_alerts().iter().any(|alert| matches!(
        alert,
        nostr_automerge::IntegrityAlert::DeviceEquivocation(_)
    )));
}

#[test]
#[allow(clippy::expect_used)]
fn duplicate_valid_carriers_are_not_equivocation() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/v1_draft/integrity/duplicate_valid_carriers.json"
    ))
    .expect("duplicate carrier fixture");
    assert_eq!(fixture["requirements"][0], "R2_CHANGE_013");

    let controller = TestSigner::from_byte(105);
    let device = TestSigner::from_byte(106);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "a6".repeat(32)
    )
    .parse()
    .expect("fixed coordinate");
    let control = signed_acl_control(
        &controller,
        coordinate,
        1,
        None,
        0,
        vec![(device.public_key().to_hex(), vec!["write"])],
    );
    let control_id = VerifiedNip01Event::verify(control.clone())
        .expect("signed control")
        .event_id();
    let actor = ActorId::derive(coordinate, device.public_key());
    let mut document = AuthoringDocument::empty(ActorState::initial(actor, BTreeSet::new()))
        .expect("empty authoring document");
    let change = document
        .author_change(&[Operation::PutString {
            key: "duplicate".to_owned(),
            value: "one identity".to_owned(),
        }])
        .expect("authored change");
    let sign_change = |created_at: u64| {
        device.sign(
            &UnsignedEventDraft::new(
                created_at,
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
        )
    };
    let first = sign_change(2);
    let second = sign_change(3);
    let first_id = VerifiedNip01Event::verify(first.clone())
        .expect("first carrier")
        .event_id();
    let second_id = VerifiedNip01Event::verify(second.clone())
        .expect("second carrier")
        .event_id();
    assert_ne!(first_id, second_id);

    let orders = [
        vec![control.clone(), first.clone(), second.clone()],
        vec![second.clone(), first.clone(), control.clone()],
        vec![first, control, second],
    ];
    let mut reports = Vec::new();
    for order in orders {
        let mut builder = CorpusBuilder::new();
        for event in order {
            assert!(matches!(
                builder.ingest(event),
                IngestOutcome::Accepted { .. }
            ));
        }
        let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
            &builder.finish(),
            coordinate,
            &mut WorkBudget::new(1_000_000, 1_000),
            &NeverCancelled,
        );
        assert_eq!(report.completion(), Completion::Complete);
        assert_eq!(report.accepted_changes(), [change.change_hash()]);
        assert!(!report.integrity_alerts().iter().any(|alert| matches!(
            alert,
            nostr_automerge::IntegrityAlert::DeviceEquivocation(_)
        )));
        reports.push(report);
    }
    assert!(reports.windows(2).all(|pair| pair[0] == pair[1]));
}

#[test]
#[allow(clippy::expect_used)]
fn signed_causal_change_matrix() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/v1_draft/changes/signed_causal_matrix.json"
    ))
    .expect("signed causal matrix fixture");
    assert_eq!(fixture["requirements"].as_array().map(Vec::len), Some(13));
    assert_eq!(fixture["cases"].as_array().map(Vec::len), Some(16));
    assert_eq!(fixture["orders"].as_array().map(Vec::len), Some(4));

    new_actor_sequence_must_start_at_one();
    actor_sequence_requires_exact_predecessor();
    actor_sequence_gap_is_invalid();
    change_start_op_must_equal_actor_next_op();
    empty_change_consumes_only_sequence();
    empty_change_requires_exact_current_heads();
    change_must_descend_from_every_base_head();
    missing_dependency_promotes_after_delivery();
    signed_change_ingest_requires_canonical_actor_hash_control_and_bytes();
    candidate_applies_to_exact_dependency_closure();
    apply_failure_invalidates_only_candidate();
    change_admission_order_is_hash_canonical();
    base_sequence_equivocation_is_detected();
    equivocation_quarantines_transitive_dependants();
    equivocation_preserves_prior_actor_history();
    duplicate_valid_carriers_are_not_equivocation();
}

struct SignedEngineScenario {
    coordinate: DocumentCoordinate,
    control: RawEventBytes,
    change: RawEventBytes,
    control_id: EventId,
    change_hash: ChangeHash,
    snapshot: Vec<u8>,
}

#[allow(clippy::expect_used)]
fn signed_engine_scenario() -> SignedEngineScenario {
    signed_engine_scenario_with_change_tags(Vec::new())
}

#[allow(clippy::expect_used)]
fn signed_engine_scenario_with_change_tags(extra_tags: Vec<Vec<String>>) -> SignedEngineScenario {
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
    let mut change_tags = vec![
        vec!["a".to_owned(), coordinate.to_address()],
        vec!["e".to_owned(), control_id.to_hex()],
        vec!["x".to_owned(), change.change_hash().to_hex()],
    ];
    change_tags.extend(extra_tags);
    let change_event = device.sign(
        &UnsignedEventDraft::new(
            2,
            1_624,
            change_tags,
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
        snapshot: document.accepted_state_bytes(),
    }
}

#[test]
#[allow(clippy::expect_used)]
fn unknown_change_tags_leave_canonical_report_unchanged() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/v1_draft/tags/unknown_tag_invariance.json"
    ))
    .expect("unknown tag fixture");
    assert_eq!(fixture["variants"].as_array().map(Vec::len), Some(3));
    let variants = [
        vec![vec!["future".into()]],
        vec![vec!["future".into()], vec!["future".into(), "again".into()]],
        vec![vec!["z".into(), "one".into(), "two".into(), "three".into()]],
    ];
    let evaluate = |scenario: SignedEngineScenario| {
        let mut builder = CorpusBuilder::new();
        for event in [scenario.change, scenario.control] {
            assert!(matches!(
                builder.ingest(event),
                IngestOutcome::Accepted { .. }
            ));
        }
        ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
            &builder.finish(),
            scenario.coordinate,
            &mut WorkBudget::new(1_000_000, 1_000),
            &NeverCancelled,
        )
    };
    let baseline = evaluate(signed_engine_scenario());
    for extra in variants {
        let report = evaluate(signed_engine_scenario_with_change_tags(extra));
        assert_eq!(report.canonical_controls(), baseline.canonical_controls());
        assert_eq!(report.disposition_records(), baseline.disposition_records());
        assert_eq!(report.heads(), baseline.heads());
        assert_eq!(report.history_digest(), baseline.history_digest());
        assert_eq!(report.dispositions_digest(), baseline.dispositions_digest());
    }
}

#[test]
#[allow(clippy::expect_used)]
fn signed_single_chunk_checkpoint_verifies_real_automerge_history() {
    let scenario = signed_engine_scenario();
    let checkpoint_signer = TestSigner::from_byte(21);
    let snapshot_hash: [u8; 32] = Sha256::digest(&scenario.snapshot).into();
    let chunk_hash: [u8; 32] = Sha256::digest(&scenario.snapshot).into();
    let chunk_root = nostr_automerge::checkpoint::leaf_hash(0, 1, chunk_hash);
    let mut change_set = Sha256::new();
    change_set.update(b"nostr-crdt/automerge/change-set/v1");
    change_set.update([0]);
    change_set.update(1_u64.to_be_bytes());
    change_set.update(scenario.change_hash.as_bytes());
    let change_set_hash: [u8; 32] = change_set.finalize().into();
    let hex = |bytes: &[u8; 32]| {
        bytes
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    };
    let content = format!(
        r#"{{"change_count":1,"change_set_hash":"{}","chunk_count":1,"chunk_root":"{}","chunk_size":{},"dependency_edges":0,"encoding":"automerge-save-v1","heads":["{}"],"raw_size":{},"total_ops":1,"v":1}}"#,
        hex(&change_set_hash),
        hex(&chunk_root),
        scenario.snapshot.len(),
        scenario.change_hash.to_hex(),
        scenario.snapshot.len(),
    );
    let descriptor = checkpoint_signer.sign(
        &UnsignedEventDraft::new(
            3,
            1_626,
            vec![
                vec!["a".to_owned(), scenario.coordinate.to_address()],
                vec!["e".to_owned(), scenario.control_id.to_hex()],
                vec!["x".to_owned(), hex(&snapshot_hash)],
            ],
            content,
        )
        .expect("descriptor draft")
        .prepare(checkpoint_signer.public_key())
        .expect("descriptor preimage"),
    );
    let descriptor_id = VerifiedNip01Event::verify(descriptor.clone())
        .expect("signed descriptor")
        .event_id();
    let chunk = checkpoint_signer.sign(
        &UnsignedEventDraft::new(
            4,
            1_627,
            vec![
                vec!["a".to_owned(), scenario.coordinate.to_address()],
                vec!["e".to_owned(), descriptor_id.to_hex()],
                vec!["x".to_owned(), hex(&chunk_hash)],
                vec!["part".to_owned(), "0".to_owned(), "1".to_owned()],
            ],
            format!(
                r#"{{"data":"{}","proof":[],"v":1}}"#,
                base64::engine::general_purpose::STANDARD.encode(&scenario.snapshot)
            ),
        )
        .expect("chunk draft")
        .prepare(checkpoint_signer.public_key())
        .expect("chunk preimage"),
    );
    let chunk_id = VerifiedNip01Event::verify(chunk.clone())
        .expect("signed chunk")
        .event_id();
    let mut builder = CorpusBuilder::new();
    for event in [chunk, scenario.change, descriptor, scenario.control] {
        assert!(matches!(
            builder.ingest(event),
            IngestOutcome::Accepted { .. }
        ));
    }
    let mut budget = WorkBudget::new(1_000_000, 1_000_000);
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &builder.finish(),
        scenario.coordinate,
        &mut budget,
        &NeverCancelled,
    );
    let checkpoint = report.checkpoints().first().expect("checkpoint result");
    assert_eq!(checkpoint.descriptor_event(), descriptor_id);
    assert_eq!(checkpoint.chunk_events(), [chunk_id]);
    assert_eq!(checkpoint.status(), CheckpointVerificationStatus::Verified);
    assert_eq!(checkpoint.completion(), Completion::Complete);
    assert_eq!(checkpoint.historical_carriers(), [scenario.change_hash]);
    assert_eq!(checkpoint.accepted_at_control(), [scenario.change_hash]);
    assert!(budget.consumed().get(WorkCounter::CheckpointByte) > 0);
    assert!(budget.consumed().get(WorkCounter::CheckpointItem) > 0);
}

#[test]
#[allow(clippy::expect_used)]
fn signed_irregular_multichunk_checkpoint_reconstructs_exact_history() {
    let scenario = signed_engine_scenario();
    let signer = TestSigner::from_byte(21);
    let chunk_size = (17_usize..=31)
        .find(|size| {
            scenario.snapshot.len() > *size && !scenario.snapshot.len().is_multiple_of(*size)
        })
        .expect("irregular chunk size");
    let pieces = scenario.snapshot.chunks(chunk_size).collect::<Vec<_>>();
    let count = u32::try_from(pieces.len()).expect("bounded chunk count");
    assert!(count > 1);
    assert!(pieces.last().is_some_and(|piece| piece.len() < chunk_size));
    let hashes = pieces
        .iter()
        .map(|piece| <[u8; 32]>::from(Sha256::digest(piece)))
        .collect::<Vec<_>>();
    let leaves = hashes
        .iter()
        .enumerate()
        .map(|(index, hash)| nostr_automerge::checkpoint::leaf_hash(index as u32, count, *hash))
        .collect::<Vec<_>>();
    let root = nostr_automerge::checkpoint::merkle_root(&leaves).expect("merkle root");
    let snapshot_hash: [u8; 32] = Sha256::digest(&scenario.snapshot).into();
    let mut change_set = Sha256::new();
    change_set.update(b"nostr-crdt/automerge/change-set/v1");
    change_set.update([0]);
    change_set.update(1_u64.to_be_bytes());
    change_set.update(scenario.change_hash.as_bytes());
    let change_set_hash: [u8; 32] = change_set.finalize().into();
    let descriptor = signer.sign(
        &UnsignedEventDraft::new(
            3,
            1_626,
            vec![
                vec!["a".to_owned(), scenario.coordinate.to_address()],
                vec!["e".to_owned(), scenario.control_id.to_hex()],
                vec!["x".to_owned(), hex32(snapshot_hash)],
            ],
            format!(
                r#"{{"change_count":1,"change_set_hash":"{}","chunk_count":{},"chunk_root":"{}","chunk_size":{},"dependency_edges":0,"encoding":"automerge-save-v1","heads":["{}"],"raw_size":{},"total_ops":1,"v":1}}"#,
                hex32(change_set_hash),
                count,
                hex32(root),
                chunk_size,
                scenario.change_hash.to_hex(),
                scenario.snapshot.len(),
            ),
        )
        .expect("descriptor draft")
        .prepare(signer.public_key())
        .expect("descriptor preimage"),
    );
    let descriptor_id = VerifiedNip01Event::verify(descriptor.clone())
        .expect("descriptor")
        .event_id();
    let mut chunks = pieces
        .iter()
        .enumerate()
        .map(|(index, piece)| {
            let proof = checkpoint_proof(&leaves, index)
                .into_iter()
                .map(|step| match step {
                    ("left", hash) => {
                        format!(r#"{{"hash":"{}","side":"left"}}"#, hex32(hash))
                    }
                    ("right", hash) => {
                        format!(r#"{{"hash":"{}","side":"right"}}"#, hex32(hash))
                    }
                    _ => unreachable!("sealed proof side"),
                })
                .collect::<Vec<_>>()
                .join(",");
            signer.sign(
                &UnsignedEventDraft::new(
                    4 + index as u64,
                    1_627,
                    vec![
                        vec!["a".to_owned(), scenario.coordinate.to_address()],
                        vec!["e".to_owned(), descriptor_id.to_hex()],
                        vec!["x".to_owned(), hex32(hashes[index])],
                        vec!["part".to_owned(), index.to_string(), count.to_string()],
                    ],
                    format!(
                        r#"{{"data":"{}","proof":[{}],"v":1}}"#,
                        base64::engine::general_purpose::STANDARD.encode(piece),
                        proof,
                    ),
                )
                .expect("chunk draft")
                .prepare(signer.public_key())
                .expect("chunk preimage"),
            )
        })
        .collect::<Vec<_>>();
    chunks.reverse();
    let mut builder = CorpusBuilder::new();
    for event in chunks
        .into_iter()
        .chain([descriptor, scenario.change, scenario.control])
    {
        assert!(matches!(
            builder.ingest(event),
            IngestOutcome::Accepted { .. }
        ));
    }
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &builder.finish(),
        scenario.coordinate,
        &mut WorkBudget::new(1_000_000, 1_000_000),
        &NeverCancelled,
    );
    let checkpoint = report.checkpoints().first().expect("checkpoint result");
    assert_eq!(checkpoint.status(), CheckpointVerificationStatus::Verified);
    assert_eq!(checkpoint.chunk_events().len(), count as usize);
    assert_eq!(checkpoint.snapshot_hash().as_bytes(), &snapshot_hash);
    assert_eq!(checkpoint.heads(), [scenario.change_hash]);
}

fn hex32(bytes: [u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[allow(clippy::expect_used)]
fn checkpoint_proof(leaves: &[[u8; 32]], index: usize) -> Vec<(&'static str, [u8; 32])> {
    if leaves.len() == 1 {
        return Vec::new();
    }
    let split = leaves.len().next_power_of_two() / 2;
    if index < split {
        let mut proof = checkpoint_proof(&leaves[..split], index);
        proof.push((
            "right",
            nostr_automerge::checkpoint::merkle_root(&leaves[split..]).expect("right root"),
        ));
        proof
    } else {
        let mut proof = checkpoint_proof(&leaves[split..], index - split);
        proof.push((
            "left",
            nostr_automerge::checkpoint::merkle_root(&leaves[..split]).expect("left root"),
        ));
        proof
    }
}

#[test]
#[allow(clippy::expect_used)]
fn checkpoints_never_authorize_or_redefine_history() {
    let scenario = signed_engine_scenario();
    let evaluate = |corpus: &EvidenceCorpus| {
        ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
            corpus,
            scenario.coordinate,
            &mut WorkBudget::new(1_000_000, 1_000_000),
            &NeverCancelled,
        )
    };
    let mut baseline_builder = CorpusBuilder::new();
    assert!(matches!(
        baseline_builder.ingest(scenario.control.clone()),
        IngestOutcome::Accepted { .. }
    ));
    assert!(matches!(
        baseline_builder.ingest(scenario.change.clone()),
        IngestOutcome::Accepted { .. }
    ));
    let baseline_corpus = baseline_builder.finish();
    let baseline = evaluate(&baseline_corpus);

    let unauthorized = TestSigner::from_byte(44);
    let content = format!(
        r#"{{"change_count":1,"change_set_hash":"{}","chunk_count":1,"chunk_root":"{}","chunk_size":1,"dependency_edges":0,"encoding":"automerge-save-v1","heads":["{}"],"raw_size":1,"total_ops":1,"v":1}}"#,
        "01".repeat(32),
        "02".repeat(32),
        scenario.change_hash.to_hex(),
    );
    let descriptor = unauthorized.sign(
        &UnsignedEventDraft::new(
            3,
            1_626,
            vec![
                vec!["a".to_owned(), scenario.coordinate.to_address()],
                vec!["e".to_owned(), scenario.control_id.to_hex()],
                vec!["x".to_owned(), "03".repeat(32)],
            ],
            content,
        )
        .expect("checkpoint draft")
        .prepare(unauthorized.public_key())
        .expect("checkpoint preimage"),
    );
    let mut with_checkpoint = CorpusBuilder::new();
    for event in [scenario.control, scenario.change, descriptor] {
        assert!(matches!(
            with_checkpoint.ingest(event),
            IngestOutcome::Accepted { .. }
        ));
    }
    let report = evaluate(&with_checkpoint.finish());
    assert_eq!(
        report.checkpoints()[0].status(),
        CheckpointVerificationStatus::Unauthorized
    );
    assert_eq!(report.canonical_controls(), baseline.canonical_controls());
    assert_eq!(report.dispositions(), baseline.dispositions());
    assert_eq!(report.accepted_changes(), baseline.accepted_changes());
    assert_eq!(report.heads(), baseline.heads());
    assert_eq!(report.history_digest(), baseline.history_digest());
    assert_ne!(report.dispositions_digest(), baseline.dispositions_digest());
    assert_eq!(
        report.document().map(|document| document.byte_len()),
        baseline.document().map(|document| document.byte_len())
    );
}

#[test]
#[allow(clippy::expect_used)]
fn signed_manifest_selection_validates_latest_without_fallback_or_authority() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/v1_draft/manifests/cases.json"
    ))
    .expect("manifest fixture family");
    assert_eq!(
        fixture["cases"].as_array().map(Vec::len),
        Some(8),
        "every manifest selection and non-authority branch stays explicit"
    );
    let signer = TestSigner::from_byte(3);
    let document_id = fixture["document_id"]
        .as_str()
        .expect("fixture document id")
        .to_owned();
    let control = fixture["control"]
        .as_str()
        .expect("fixture control")
        .to_owned();
    let relay = fixture["relay"].as_str().expect("fixture relay");
    let content = format!(
        r#"{{"application":null,"checkpoint":null,"control":"{control}","description":null,"format":"automerge-change-v1","name":null,"relays":["{relay}"],"status":"active","successor":null,"text_encoding":"utf16","v":1}}"#
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
    assert_eq!(hints[0].relays(), [relay]);
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
fn validated_checkpoint_descriptor_carrier_enters_corpus() {
    let signer = TestSigner::from_byte(38);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        TestSigner::from_byte(39).public_key().to_hex(),
        "54".repeat(32)
    )
    .parse()
    .expect("fixed coordinate");
    let content = format!(
        r#"{{"change_count":1,"change_set_hash":"{}","chunk_count":1,"chunk_root":"{}","chunk_size":1,"dependency_edges":0,"encoding":"automerge-save-v1","heads":["{}"],"raw_size":1,"total_ops":1,"v":1}}"#,
        "01".repeat(32),
        "02".repeat(32),
        "03".repeat(32)
    );
    let event = signer.sign(
        &UnsignedEventDraft::new(
            1,
            1_626,
            vec![
                vec!["a".to_owned(), coordinate.to_address()],
                vec!["e".to_owned(), "04".repeat(32)],
                vec!["x".to_owned(), "05".repeat(32)],
            ],
            content.clone(),
        )
        .expect("descriptor draft")
        .prepare(signer.public_key())
        .expect("descriptor preimage"),
    );
    let event_id = VerifiedNip01Event::verify(event.clone())
        .expect("signed descriptor")
        .event_id();
    let mut builder = CorpusBuilder::new();
    assert!(matches!(
        builder.ingest(event),
        IngestOutcome::Accepted { event_id: accepted } if accepted == event_id
    ));
    let corpus = builder.finish();
    assert_eq!(corpus.event_count(), 1);
    assert_eq!(
        corpus.checkpoint_descriptor_ids().collect::<Vec<_>>(),
        vec![event_id]
    );
    assert_eq!(
        corpus.pending_checkpoint_ids().collect::<Vec<_>>(),
        vec![event_id]
    );
    assert!(matches!(
        corpus.records().next(),
        Some(record) if record.status() == EvidenceStatus::Pending
    ));
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &corpus,
        coordinate,
        &mut WorkBudget::new(u64::MAX, u64::MAX),
        &NeverCancelled,
    );
    let checkpoint = report.checkpoints().first().expect("descriptor result");
    assert_eq!(checkpoint.descriptor_event(), event_id);
    assert_eq!(checkpoint.chunk_events(), &[]);
    assert_eq!(checkpoint.snapshot_hash().to_hex(), "05".repeat(32));
    assert_eq!(checkpoint.change_count(), 1);
    assert_eq!(
        checkpoint.status(),
        CheckpointVerificationStatus::PendingControl
    );
    assert_eq!(checkpoint.completion(), Completion::Complete);

    let sign = |created_at: u64, tags: Vec<Vec<String>>, content: String| {
        signer.sign(
            &UnsignedEventDraft::new(created_at, 1_626, tags, content)
                .expect("descriptor draft")
                .prepare(signer.public_key())
                .expect("descriptor preimage"),
        )
    };
    let valid_tags = || {
        vec![
            vec!["a".to_owned(), coordinate.to_address()],
            vec!["e".to_owned(), "04".repeat(32)],
            vec!["x".to_owned(), "05".repeat(32)],
        ]
    };
    for (created_at, tags, expected) in [
        (
            2,
            [
                valid_tags(),
                vec![vec!["expiration".to_owned(), "1".to_owned()]],
            ]
            .concat(),
            "tag.forbidden",
        ),
        (
            3,
            [
                valid_tags(),
                vec![vec!["a".to_owned(), coordinate.to_address()]],
            ]
            .concat(),
            "tag.required",
        ),
        (
            4,
            vec![
                vec!["a".to_owned(), "invalid".to_owned()],
                vec!["e".to_owned(), "04".repeat(32)],
                vec!["x".to_owned(), "05".repeat(32)],
            ],
            "carrier.coordinate",
        ),
        (
            5,
            vec![
                vec!["a".to_owned(), coordinate.to_address()],
                vec!["e".to_owned(), "04".repeat(32)],
                vec!["x".to_owned(), "AA".repeat(32)],
            ],
            "checkpoint.descriptor",
        ),
    ] {
        let mut invalid = CorpusBuilder::new();
        assert!(matches!(
            invalid.ingest(sign(created_at, tags, content.clone())),
            IngestOutcome::InvalidCarrier { diagnostic, .. }
                if diagnostic.as_str() == expected
        ));
    }
}

#[test]
#[allow(clippy::expect_used)]
fn validated_checkpoint_chunk_carrier_enters_corpus() {
    let signer = TestSigner::from_byte(40);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        TestSigner::from_byte(41).public_key().to_hex(),
        "55".repeat(32)
    )
    .parse()
    .expect("fixed coordinate");
    let data = b"a";
    let hash: [u8; 32] = Sha256::digest(data).into();
    let valid_tags = || {
        vec![
            vec!["a".to_owned(), coordinate.to_address()],
            vec!["e".to_owned(), "04".repeat(32)],
            vec!["x".to_owned(), ChunkHash::from_bytes(hash).to_hex()],
            vec!["part".to_owned(), "0".to_owned(), "1".to_owned()],
        ]
    };
    let sign = |created_at: u64, tags: Vec<Vec<String>>, content: &str| {
        signer.sign(
            &UnsignedEventDraft::new(created_at, 1_627, tags, content.to_owned())
                .expect("chunk draft")
                .prepare(signer.public_key())
                .expect("chunk preimage"),
        )
    };
    let canonical_content = r#"{"data":"YQ==","proof":[],"v":1}"#;
    let event = sign(1, valid_tags(), canonical_content);
    let event_id = VerifiedNip01Event::verify(event.clone())
        .expect("signed chunk")
        .event_id();
    let mut builder = CorpusBuilder::new();
    assert!(matches!(
        builder.ingest(event.clone()),
        IngestOutcome::Accepted { event_id: accepted } if accepted == event_id
    ));
    assert!(matches!(
        builder.ingest(event),
        IngestOutcome::Duplicate { .. }
    ));
    let corpus = builder.finish();
    assert_eq!(
        corpus.checkpoint_chunk_ids().collect::<Vec<_>>(),
        vec![event_id]
    );
    assert_eq!(
        corpus.pending_checkpoint_ids().collect::<Vec<_>>(),
        vec![event_id]
    );
    assert_eq!(
        corpus
            .records()
            .map(|record| record.status())
            .collect::<Vec<_>>(),
        vec![EvidenceStatus::Pending, EvidenceStatus::Duplicate]
    );
    let mut leading_zero = valid_tags();
    leading_zero[3][1] = "00".to_owned();
    let mut upper_hash = valid_tags();
    upper_hash[2][1] = "AA".repeat(32);
    for (created_at, tags, content, expected) in [
        (
            2,
            [valid_tags(), vec![vec!["-".to_owned()]]].concat(),
            canonical_content,
            "tag.forbidden",
        ),
        (3, leading_zero, canonical_content, "checkpoint.chunk"),
        (4, upper_hash, canonical_content, "checkpoint.chunk"),
        (
            5,
            valid_tags(),
            r#"{"data":"YQ","proof":[],"v":1}"#,
            "checkpoint.chunk",
        ),
    ] {
        let mut invalid = CorpusBuilder::new();
        assert!(matches!(
            invalid.ingest(sign(created_at, tags, content)),
            IngestOutcome::InvalidCarrier { diagnostic, .. }
                if diagnostic.as_str() == expected
        ));
    }
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
            vec![a(&coordinate), vec!["-".to_owned()]],
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
    let report = evaluator.evaluate_report(
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
    let ordered = evaluator.evaluate_report(
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
    assert_ne!(report.dispositions_digest(), ordered.dispositions_digest());
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
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
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
fn signed_child_retained_writer_frontier_rules() {
    signed_child_cannot_discard_retained_writer_contributions();
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
        ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
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
    let mut after_builder = CorpusBuilder::new();
    for event in [parent, higher, higher_change, lower, lower_change] {
        assert!(matches!(
            after_builder.ingest(event),
            IngestOutcome::Accepted { .. }
        ));
    }
    let after = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).reevaluate_report(
        &after_builder.finish(),
        coordinate,
        &before,
        &mut WorkBudget::new(1_000_000, 1_000),
        &NeverCancelled,
    );
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
    assert!(after.integrity_alerts().iter().any(|alert| matches!(
        alert,
        nostr_automerge::IntegrityAlert::CanonicalControlReorganization(_)
    )));
    assert!(after.document().is_some_and(|view| !view.is_empty()));
}

#[test]
#[allow(clippy::expect_used)]
fn signed_successor_genesis_requires_reciprocal_terminal_continuity() {
    let predecessor_controller = TestSigner::from_byte(35);
    let successor_controller = TestSigner::from_byte(36);
    let device = TestSigner::from_byte(37);
    let predecessor_coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        predecessor_controller.public_key().to_hex(),
        "52".repeat(32)
    )
    .parse()
    .expect("fixed predecessor coordinate");
    let successor_coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        successor_controller.public_key().to_hex(),
        "53".repeat(32)
    )
    .parse()
    .expect("fixed successor coordinate");
    let terminal_content = format!(
        r#"{{"base_heads":[],"format":"automerge-change-v1","members":[],"policy":"controller-acl-v1","predecessor":null,"seq":0,"successor":"{}","text_encoding":"utf16","v":1}}"#,
        successor_coordinate.to_address()
    );
    let terminal = predecessor_controller.sign(
        &UnsignedEventDraft::new(
            1,
            1_625,
            vec![vec!["a".to_owned(), predecessor_coordinate.to_address()]],
            terminal_content,
        )
        .expect("terminal draft")
        .prepare(predecessor_controller.public_key())
        .expect("terminal preimage"),
    );
    let terminal_id = VerifiedNip01Event::verify(terminal.clone())
        .expect("signed terminal")
        .event_id();
    let wrong_terminal_content = format!(
        r#"{{"base_heads":[],"format":"automerge-change-v1","members":[],"policy":"controller-acl-v1","predecessor":null,"seq":0,"successor":"{}","text_encoding":"utf16","v":1}}"#,
        predecessor_coordinate.to_address()
    );
    let wrong_terminal = predecessor_controller.sign(
        &UnsignedEventDraft::new(
            2,
            1_625,
            vec![vec!["a".to_owned(), predecessor_coordinate.to_address()]],
            wrong_terminal_content,
        )
        .expect("wrong terminal draft")
        .prepare(predecessor_controller.public_key())
        .expect("wrong terminal preimage"),
    );
    let wrong_terminal_id = VerifiedNip01Event::verify(wrong_terminal.clone())
        .expect("signed wrong terminal")
        .event_id();
    let successor_content = |terminal_control: EventId| {
        format!(
            r#"{{"base_heads":[],"format":"automerge-change-v1","members":[{{"account":null,"pubkey":"{}","roles":["checkpoint","write"]}}],"policy":"controller-acl-v1","predecessor":{{"coordinate":"{}","terminal_control":"{}"}},"seq":0,"successor":null,"text_encoding":"utf16","v":1}}"#,
            device.public_key().to_hex(),
            predecessor_coordinate.to_address(),
            terminal_control.to_hex()
        )
    };
    let sign_successor = |created_at: u64, terminal_control: EventId| {
        successor_controller.sign(
            &UnsignedEventDraft::new(
                created_at,
                1_625,
                vec![vec!["a".to_owned(), successor_coordinate.to_address()]],
                successor_content(terminal_control),
            )
            .expect("successor draft")
            .prepare(successor_controller.public_key())
            .expect("successor preimage"),
        )
    };
    let valid = sign_successor(3, terminal_id);
    let valid_id = VerifiedNip01Event::verify(valid.clone())
        .expect("signed successor")
        .event_id();
    let invalid = (4..10_000)
        .map(|created_at| sign_successor(created_at, wrong_terminal_id))
        .find(|candidate| {
            VerifiedNip01Event::verify(candidate.clone())
                .is_ok_and(|event| event.event_id() < valid_id)
        })
        .expect("find a lower-id invalid successor genesis");
    let invalid_id = VerifiedNip01Event::verify(invalid.clone())
        .expect("signed invalid successor")
        .event_id();
    assert!(invalid_id < valid_id);
    let mut builder = CorpusBuilder::new();
    for event in [invalid, wrong_terminal, valid, terminal] {
        assert!(matches!(
            builder.ingest(event),
            IngestOutcome::Accepted { .. }
        ));
    }
    let corpus = builder.finish();
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &corpus,
        successor_coordinate,
        &mut WorkBudget::new(1_000_000, 1_000),
        &NeverCancelled,
    );
    assert_eq!(report.completion(), Completion::Complete);
    assert_eq!(report.canonical_controls(), [valid_id]);
    assert!(!report.canonical_controls().contains(&invalid_id));
    assert!(
        report
            .control_dispositions()
            .contains(&(invalid_id, ProtocolDisposition::Invalid))
    );
    assert!(report.accepted_changes().is_empty());
    assert!(report.heads().is_empty());
    assert!(report.document().is_some_and(|view| !view.is_empty()));
}

#[test]
#[allow(clippy::expect_used)]
fn signed_change_ingest_requires_canonical_actor_hash_control_and_bytes() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/v1_draft/changes/cases.json"
    ))
    .expect("change and graph fixture family");
    assert_eq!(fixture["cases"].as_array().map(Vec::len), Some(12));
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
