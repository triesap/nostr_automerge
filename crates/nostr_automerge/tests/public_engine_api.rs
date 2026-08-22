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
    EventId, EvidenceCorpus, EvidenceStatus, IngestOutcome, ManifestControlStatus,
    ManifestPendingReason, MaterializedPathElement, MaterializedScalar, MaterializedValue,
    NeverCancelled, ProtocolDisposition, ProtocolItemIdentifier, ProtocolRevision, RawEventBytes,
    ReferenceEvaluator, ResolvedManifestAvailability, VerifiedNip01Event, WorkBudget, WorkCounter,
    WorkCounters,
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

fn assert_canonical_control_outcomes_are_consistent(report: &EvaluationReport) {
    for control in report.canonical_controls() {
        assert_eq!(
            report
                .control_dispositions()
                .binary_search_by_key(control, |(event_id, _)| *event_id)
                .ok()
                .and_then(|index| report.control_dispositions().get(index))
                .map(|(_, disposition)| *disposition),
            Some(ProtocolDisposition::Accepted),
            "canonical control {control:?} must have one accepted outcome"
        );
    }
}

fn event_disposition(report: &EvaluationReport, event_id: EventId) -> Option<ProtocolDisposition> {
    report
        .disposition_records()
        .iter()
        .find(|record| record.identifier() == ProtocolItemIdentifier::event(event_id))
        .map(|record| record.disposition())
}

fn event_diagnostic(report: &EvaluationReport, event_id: EventId) -> Option<&'static str> {
    report
        .disposition_records()
        .iter()
        .find(|record| record.identifier() == ProtocolItemIdentifier::event(event_id))
        .and_then(|record| record.diagnostic())
        .map(nostr_automerge::DiagnosticCode::as_str)
}

fn assert_checkpoint_event_dispositions(report: &EvaluationReport, expected: ProtocolDisposition) {
    for checkpoint in report.checkpoints() {
        assert_eq!(
            event_disposition(report, checkpoint.descriptor_event()),
            Some(expected)
        );
        for event_id in checkpoint.chunk_events() {
            assert_eq!(event_disposition(report, *event_id), Some(expected));
        }
    }
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
    assert!(evaluator.contains("ProjectionError::Invalid"));
    assert!(evaluator.contains("settle_reserved_error("));
    assert!(evaluator.contains("EvaluationError::Projection"));
    assert!(!evaluator.contains("applied state must project"));
}

#[test]
fn conflict_aware_projection_types_are_public_and_exact() {
    use nostr_automerge::{MaterializedMarkExpansion, MaterializedPathElement};

    let branch = MaterializedPathElement::branch("_root", "1@actor", "1@actor");
    assert_eq!(
        branch.branch_identity(),
        Some(("_root", "1@actor", "1@actor"))
    );
    assert!(MaterializedPathElement::Key("key".to_owned()) < branch);
    assert_eq!(
        [
            MaterializedMarkExpansion::None,
            MaterializedMarkExpansion::Before,
            MaterializedMarkExpansion::After,
            MaterializedMarkExpansion::Both,
        ]
        .len(),
        4
    );
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
#[allow(clippy::expect_used)]
fn deep_noncanonical_branch_is_validated_before_exclusion() {
    let controller = TestSigner::from_byte(155);
    let writer = TestSigner::from_byte(156);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "cb".repeat(32)
    )
    .parse()
    .expect("fixed coordinate");
    let members = vec![(writer.public_key().to_hex(), vec!["write"])];
    let genesis = signed_acl_control(&controller, coordinate, 1, None, 0, members.clone());
    let genesis_id = VerifiedNip01Event::verify(genesis.clone())
        .expect("signed genesis")
        .event_id();
    let first = signed_acl_control(
        &controller,
        coordinate,
        2,
        Some(genesis_id),
        1,
        members.clone(),
    );
    let second = signed_acl_control(
        &controller,
        coordinate,
        3,
        Some(genesis_id),
        1,
        members.clone(),
    );
    let first_id = VerifiedNip01Event::verify(first.clone())
        .expect("signed first fork")
        .event_id();
    let second_id = VerifiedNip01Event::verify(second.clone())
        .expect("signed second fork")
        .event_id();
    let noncanonical_id = first_id.max(second_id);
    let grandchild = signed_acl_control(
        &controller,
        coordinate,
        4,
        Some(noncanonical_id),
        2,
        members,
    );
    let grandchild_id = VerifiedNip01Event::verify(grandchild.clone())
        .expect("signed noncanonical grandchild")
        .event_id();
    let actor = ActorId::derive(coordinate, writer.public_key());
    let mut document = AuthoringDocument::empty(ActorState::initial(actor, BTreeSet::new()))
        .expect("empty authoring document");
    let authored = document
        .author_change(&[Operation::PutString {
            key: "branch".to_owned(),
            value: "noncanonical".to_owned(),
        }])
        .expect("canonical authored change");
    let change_hash = authored.change_hash();
    let claim = writer.sign(
        &UnsignedEventDraft::new(
            5,
            1_624,
            vec![
                vec!["a".to_owned(), coordinate.to_address()],
                vec!["e".to_owned(), grandchild_id.to_hex()],
                vec!["x".to_owned(), change_hash.to_hex()],
            ],
            base64::engine::general_purpose::STANDARD.encode(authored.raw()),
        )
        .expect("change draft")
        .prepare(writer.public_key())
        .expect("change preimage"),
    );
    let events = vec![genesis, first, second, grandchild, claim];
    let mut reversed = events.clone();
    reversed.reverse();
    let mut duplicated = reversed.clone();
    duplicated.extend(events.clone());
    let reports = [events, reversed, duplicated].map(|events| {
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
        assert!(
            report
                .control_dispositions()
                .contains(&(grandchild_id, ProtocolDisposition::Excluded))
        );
        assert_eq!(report.excluded_changes(), [change_hash]);
        assert!(report.invalid_changes().is_empty());
    }
    assert_eq!(
        reports[0].control_dispositions(),
        reports[1].control_dispositions()
    );
    assert_eq!(reports[0].dispositions(), reports[2].dispositions());
}

#[test]
#[allow(clippy::expect_used)]
fn noncanonical_child_uses_its_actual_parent_frontier() {
    let controller = TestSigner::from_byte(157);
    let writer = TestSigner::from_byte(158);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "cc".repeat(32)
    )
    .parse()
    .expect("fixed coordinate");
    let members = vec![(writer.public_key().to_hex(), vec!["write"])];
    let genesis = signed_acl_control(&controller, coordinate, 1, None, 0, members.clone());
    let genesis_id = VerifiedNip01Event::verify(genesis.clone())
        .expect("signed genesis")
        .event_id();
    let first = signed_acl_control(
        &controller,
        coordinate,
        2,
        Some(genesis_id),
        1,
        members.clone(),
    );
    let second = signed_acl_control(
        &controller,
        coordinate,
        3,
        Some(genesis_id),
        1,
        members.clone(),
    );
    let first_id = VerifiedNip01Event::verify(first.clone())
        .expect("signed first fork")
        .event_id();
    let second_id = VerifiedNip01Event::verify(second.clone())
        .expect("signed second fork")
        .event_id();
    let canonical_id = first_id.min(second_id);
    let noncanonical_id = first_id.max(second_id);
    let actor = ActorId::derive(coordinate, writer.public_key());
    let mut document = AuthoringDocument::empty(ActorState::initial(actor, BTreeSet::new()))
        .expect("empty authoring document");
    let authored = document
        .author_change(&[Operation::PutString {
            key: "canonical".to_owned(),
            value: "only".to_owned(),
        }])
        .expect("canonical branch change");
    let change_hash = authored.change_hash();
    let claim = writer.sign(
        &UnsignedEventDraft::new(
            4,
            1_624,
            vec![
                vec!["a".to_owned(), coordinate.to_address()],
                vec!["e".to_owned(), canonical_id.to_hex()],
                vec!["x".to_owned(), change_hash.to_hex()],
            ],
            base64::engine::general_purpose::STANDARD.encode(authored.raw()),
        )
        .expect("change draft")
        .prepare(writer.public_key())
        .expect("change preimage"),
    );
    let grandchild = signed_acl_control_with_base(
        &controller,
        coordinate,
        5,
        Some(noncanonical_id),
        2,
        members,
        &[change_hash],
    );
    let grandchild_id = VerifiedNip01Event::verify(grandchild.clone())
        .expect("signed noncanonical grandchild")
        .event_id();

    let events = [genesis, first, second, claim, grandchild];
    let reports = [events.to_vec(), events.into_iter().rev().collect()].map(|events| {
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
            &mut WorkBudget::new(1_000_000, 2_000),
            &NeverCancelled,
        )
    });
    for report in &reports {
        assert!(
            report
                .control_dispositions()
                .contains(&(grandchild_id, ProtocolDisposition::Invalid))
        );
        assert_eq!(report.accepted_changes(), [change_hash]);
    }
    assert_eq!(reports[0], reports[1]);
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
fn terminal_control_change_is_invalid() {
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
    assert!(evaluator_source.contains("evaluate_branch_table("));
    assert!(evaluator_source.contains("ParentEpochView::from_result(&branch.epoch)"));
    assert!(public_adapter_source.contains("envelope: Some(envelope)"));
    children_are_evaluated_one_epoch_at_a_time();
}

#[test]
fn accepted_at_control_is_exact_closure() {
    let evaluator_source = include_str!("../src/reference/evaluate.rs");
    for field in [
        "epoch: EpochEvaluationResult",
        "validated_base: BTreeSet<ChangeHash>",
        "ancestry: Vec<ControlEnvelope>",
        "prior_knowledge: BTreeMap<ChangeHash, PriorChangeKnowledge>",
    ] {
        assert!(evaluator_source.contains(field));
    }
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
        [(change_hash, ProtocolDisposition::Invalid)]
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
#[allow(clippy::expect_used)]
fn change_disposition_collections_are_disjoint() {
    let scenario = signed_engine_scenario();
    let change_event_id = VerifiedNip01Event::verify(scenario.change.clone())
        .expect("signed change carrier")
        .event_id();
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
    let carrier_records = report
        .disposition_records()
        .iter()
        .filter(|record| record.identifier() == ProtocolItemIdentifier::event(change_event_id))
        .collect::<Vec<_>>();
    assert_eq!(carrier_records.len(), 1);
    assert_eq!(
        carrier_records[0].disposition(),
        ProtocolDisposition::Accepted
    );
}

#[test]
#[allow(clippy::expect_used)]
fn mixed_change_carrier_outcomes_are_visible_and_order_stable() {
    let scenario = signed_engine_scenario();
    let device = TestSigner::from_byte(21);
    let encoded: serde_json::Value =
        serde_json::from_str(scenario.change.as_str()).expect("signed change JSON");
    let content = encoded["content"]
        .as_str()
        .expect("signed change content")
        .to_owned();
    let sign = |created_at: u64, control: EventId, hash: String| {
        device.sign(
            &UnsignedEventDraft::new(
                created_at,
                1_624,
                vec![
                    vec!["a".to_owned(), scenario.coordinate.to_address()],
                    vec!["e".to_owned(), control.to_hex()],
                    vec!["x".to_owned(), hash],
                ],
                content.clone(),
            )
            .expect("change draft")
            .prepare(device.public_key())
            .expect("change preimage"),
        )
    };
    let pending = sign(
        3,
        EventId::from_bytes([0xee; 32]),
        scenario.change_hash.to_hex(),
    );
    let invalid = sign(4, scenario.control_id, "00".repeat(32));
    let accepted_id = VerifiedNip01Event::verify(scenario.change.clone())
        .expect("accepted carrier")
        .event_id();
    let pending_id = VerifiedNip01Event::verify(pending.clone())
        .expect("pending carrier")
        .event_id();
    let invalid_id = VerifiedNip01Event::verify(invalid.clone())
        .expect("invalid carrier")
        .event_id();
    let evaluate = |events: Vec<RawEventBytes>| {
        let mut builder = CorpusBuilder::new();
        for event in events {
            let _ = builder.ingest(event);
        }
        ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
            &builder.finish(),
            scenario.coordinate,
            &mut WorkBudget::new(1_000_000, 1_000),
            &NeverCancelled,
        )
    };
    let ordered = evaluate(vec![
        scenario.control.clone(),
        scenario.change.clone(),
        pending.clone(),
        invalid.clone(),
    ]);
    let reversed = evaluate(vec![invalid, pending, scenario.change, scenario.control]);
    assert_eq!(
        event_disposition(&ordered, accepted_id),
        Some(ProtocolDisposition::Accepted)
    );
    assert_eq!(
        event_disposition(&ordered, pending_id),
        Some(ProtocolDisposition::Pending)
    );
    assert_eq!(
        event_disposition(&ordered, invalid_id),
        Some(ProtocolDisposition::Invalid)
    );
    assert_eq!(ordered.accepted_changes(), [scenario.change_hash]);
    assert_eq!(
        ordered.disposition_records(),
        reversed.disposition_records()
    );
    assert_eq!(
        ordered.dispositions_digest(),
        reversed.dispositions_digest()
    );
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
        (valid_id, ProtocolDisposition::Excluded),
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
#[allow(clippy::expect_used)]
fn change_before_control_has_a_pending_hash_outcome() {
    let scenario = signed_engine_scenario();
    let mut builder = CorpusBuilder::new();
    assert!(matches!(
        builder.ingest(scenario.change),
        IngestOutcome::Accepted { .. }
    ));
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &builder.finish(),
        scenario.coordinate,
        &mut WorkBudget::new(1_000_000, 10_000),
        &NeverCancelled,
    );
    assert_eq!(
        report.dispositions(),
        [(scenario.change_hash, ProtocolDisposition::Pending)]
    );
    assert_eq!(report.pending_changes(), [scenario.change_hash]);
}

#[test]
#[allow(clippy::expect_used)]
fn valid_claim_dominates_a_missing_control_duplicate() {
    let scenario = signed_engine_scenario();
    let device = TestSigner::from_byte(21);
    let verified = VerifiedNip01Event::verify(scenario.change.clone()).expect("valid change");
    let duplicate = device.sign(
        &UnsignedEventDraft::new(
            3,
            1_624,
            verified
                .tags()
                .iter()
                .map(|tag| {
                    if tag.first().is_some_and(|name| name == "e") {
                        vec!["e".to_owned(), "ee".repeat(32)]
                    } else {
                        tag.clone()
                    }
                })
                .collect(),
            verified.content().to_owned(),
        )
        .expect("duplicate claim")
        .prepare(device.public_key())
        .expect("duplicate preimage"),
    );
    let mut builder = CorpusBuilder::new();
    for event in [scenario.control, scenario.change, duplicate] {
        assert!(matches!(
            builder.ingest(event),
            IngestOutcome::Accepted { .. }
        ));
    }
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &builder.finish(),
        scenario.coordinate,
        &mut WorkBudget::new(2_000_000, 20_000),
        &NeverCancelled,
    );
    assert_eq!(report.accepted_changes(), [scenario.change_hash]);
    assert_eq!(
        report.dispositions(),
        [(scenario.change_hash, ProtocolDisposition::Accepted)]
    );
}

#[test]
#[allow(clippy::expect_used)]
fn unsupported_control_reference_is_invalid() {
    let scenario = signed_engine_scenario();
    let controller = TestSigner::from_byte(20);
    let device = TestSigner::from_byte(21);
    let verified_control = VerifiedNip01Event::verify(scenario.control).expect("signed control");
    let unsupported_control = controller.sign(
        &UnsignedEventDraft::new(
            verified_control.created_at(),
            verified_control.kind(),
            verified_control.tags().to_vec(),
            verified_control.content().replace("\"v\":1", "\"v\":2"),
        )
        .expect("unsupported control draft")
        .prepare(controller.public_key())
        .expect("unsupported control preimage"),
    );
    let unsupported_control_id = VerifiedNip01Event::verify(unsupported_control.clone())
        .expect("unsupported control event")
        .event_id();
    let verified_change = VerifiedNip01Event::verify(scenario.change).expect("signed change");
    let change = device.sign(
        &UnsignedEventDraft::new(
            verified_change.created_at(),
            verified_change.kind(),
            verified_change
                .tags()
                .iter()
                .map(|tag| {
                    if tag.first().is_some_and(|name| name == "e") {
                        vec!["e".to_owned(), unsupported_control_id.to_hex()]
                    } else {
                        tag.clone()
                    }
                })
                .collect(),
            verified_change.content().to_owned(),
        )
        .expect("dependent change draft")
        .prepare(device.public_key())
        .expect("dependent change preimage"),
    );
    let mut builder = CorpusBuilder::new();
    assert!(matches!(
        builder.ingest(unsupported_control),
        IngestOutcome::UnsupportedRevision { .. }
    ));
    assert!(matches!(
        builder.ingest(change),
        IngestOutcome::Accepted { .. }
    ));
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &builder.finish(),
        scenario.coordinate,
        &mut WorkBudget::new(2_000_000, 20_000),
        &NeverCancelled,
    );
    assert_eq!(
        report.dispositions(),
        [(scenario.change_hash, ProtocolDisposition::Invalid)]
    );
}

#[test]
#[allow(clippy::expect_used)]
fn noncanonical_authorization_is_enforced_before_exclusion() {
    let scenario = signed_engine_scenario();
    let controller = TestSigner::from_byte(20);
    let unauthorized = TestSigner::from_byte(21);
    let permitted = TestSigner::from_byte(22);
    let members = vec![(permitted.public_key().to_hex(), vec!["write"])];
    let first = signed_acl_control(
        &controller,
        scenario.coordinate,
        1,
        None,
        0,
        members.clone(),
    );
    let first_id = VerifiedNip01Event::verify(first.clone())
        .expect("first control")
        .event_id();
    let competing = signed_acl_control(&controller, scenario.coordinate, 3, None, 0, members);
    let competing_id = VerifiedNip01Event::verify(competing.clone())
        .expect("competing control")
        .event_id();
    let noncanonical_id = first_id.max(competing_id);
    let verified_change = VerifiedNip01Event::verify(scenario.change).expect("signed change");
    let claim = unauthorized.sign(
        &UnsignedEventDraft::new(
            verified_change.created_at(),
            verified_change.kind(),
            verified_change
                .tags()
                .iter()
                .map(|tag| {
                    if tag.first().is_some_and(|name| name == "e") {
                        vec!["e".to_owned(), noncanonical_id.to_hex()]
                    } else {
                        tag.clone()
                    }
                })
                .collect(),
            verified_change.content().to_owned(),
        )
        .expect("unauthorized claim draft")
        .prepare(unauthorized.public_key())
        .expect("unauthorized claim preimage"),
    );
    let mut builder = CorpusBuilder::new();
    for event in [first, competing, claim] {
        assert!(matches!(
            builder.ingest(event),
            IngestOutcome::Accepted { .. }
        ));
    }
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &builder.finish(),
        scenario.coordinate,
        &mut WorkBudget::new(2_000_000, 20_000),
        &NeverCancelled,
    );
    assert_eq!(
        report.dispositions(),
        [(scenario.change_hash, ProtocolDisposition::Invalid)]
    );
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
    assert_eq!(budget.consumed().get(WorkCounter::Event), 0);
    assert_eq!(budget.consumed().get(WorkCounter::Carrier), 0);
    assert_eq!(budget.consumed().get(WorkCounter::Assertion), 0);
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
    let mut budget = WorkBudget::new(1_000_000, 1_000);
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &builder.finish(),
        scenario.coordinate,
        &mut budget,
        &|| true,
    );

    assert_eq!(report.completion(), Completion::Cancelled);
    assert_eq!(report.failure(), Some(EvaluationFailure::Cancelled));
    assert!(report.canonical_controls().is_empty());
    assert!(report.dispositions().is_empty());
    assert!(report.accepted_changes().is_empty());
    assert!(report.document().is_none());
    assert_eq!(budget.remaining(), (1_000_000, 1_000));
}

#[test]
fn zero_budget_target_entry_consumes_no_work() {
    let scenario = signed_engine_scenario();
    let mut builder = CorpusBuilder::new();
    for event in [scenario.change, scenario.control] {
        assert!(matches!(
            builder.ingest(event),
            IngestOutcome::Accepted { .. }
        ));
    }
    let mut budget = WorkBudget::new(0, 0);
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &builder.finish(),
        scenario.coordinate,
        &mut budget,
        &NeverCancelled,
    );
    assert_eq!(report.completion(), Completion::BudgetExhausted);
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
        assert_eq!(budget.consumed().get(counter), 0, "{counter:?}");
    }
}

#[test]
#[allow(clippy::expect_used)]
fn adversarial_deep_control_chain_is_deterministically_bounded() {
    let controller = TestSigner::from_byte(118);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "bd".repeat(32)
    )
    .parse()
    .expect("deep-chain coordinate");
    let members = vec![(controller.public_key().to_hex(), vec!["write"])];
    let mut parent = None;
    let mut events = Vec::new();
    let mut expected = Vec::new();
    for sequence in 0_u64..64 {
        let event = signed_acl_control(
            &controller,
            coordinate,
            sequence.saturating_add(1),
            parent,
            sequence,
            members.clone(),
        );
        let event_id = VerifiedNip01Event::verify(event.clone())
            .expect("deep-chain control")
            .event_id();
        parent = Some(event_id);
        expected.push(event_id);
        events.push(event);
    }
    let mut builder = CorpusBuilder::new();
    for event in events.into_iter().rev() {
        assert!(matches!(
            builder.ingest(event),
            IngestOutcome::Accepted { .. }
        ));
    }
    let corpus = builder.finish();
    let evaluator = ReferenceEvaluator::new(ProtocolRevision::draft_v1());
    let run = |items| {
        let mut budget = WorkBudget::new(10_000_000, items);
        let report = evaluator.evaluate_report(&corpus, coordinate, &mut budget, &NeverCancelled);
        (report, budget.consumed())
    };
    let (first, first_work) = run(10_000_000);
    let (second, second_work) = run(10_000_000);
    assert_eq!(first.completion(), Completion::Complete);
    assert_eq!(first.canonical_controls(), expected);
    assert!(first == second);
    assert_eq!(first_work, second_work);
    assert!(first_work.get(WorkCounter::Control) < 100_000);
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
    let corpus = builder.finish();
    let evaluator = ReferenceEvaluator::new(ProtocolRevision::draft_v1());
    let mut measured = WorkBudget::new(1_000_000, 1_000);
    let complete =
        evaluator.evaluate_report(&corpus, scenario.coordinate, &mut measured, &NeverCancelled);
    assert_eq!(complete.completion(), Completion::Complete);
    let first_work = measured.consumed();
    let mut repeated = WorkBudget::new(1_000_000, 1_000);
    let repeated_report =
        evaluator.evaluate_report(&corpus, scenario.coordinate, &mut repeated, &NeverCancelled);
    assert_eq!(complete, repeated_report);
    assert_eq!(first_work, repeated.consumed());
    assert!(first_work.get(WorkCounter::Control) >= 2);
    let source = include_str!("../src/reference/evaluate.rs");
    assert!(source.contains("evaluate_branch_table("));
    assert!(source.contains("select_valid_outcomes_with_alert(parent_id, outcomes)"));
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
    assert_eq!(report.canonical_controls(), [scenario.control_id]);
    assert_eq!(report.accepted_changes(), [scenario.change_hash]);
    assert!(report.document().is_none());
    assert!(exhausted.consumed().get(WorkCounter::DecodeByte) < decode_bytes);
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
    assert_eq!(measured.consumed().get(WorkCounter::ApplyChange), 3);
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
    assert_eq!(report.history_digest(), measured_report.history_digest());
    assert_ne!(
        report.dispositions_digest(),
        measured_report.dispositions_digest(),
        "the interrupted report has not finalized the carrier Event record"
    );
    assert!(report.document().is_none());
    assert_eq!(exhausted.consumed().get(WorkCounter::ApplyChange), 3);
    assert_eq!(exhausted.remaining().1, 0);
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
    assert_eq!(budget.consumed().get(WorkCounter::ApplyChange), 2);
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
fn every_v3_work_counter_boundary() {
    let matrix: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/v1_draft/resources/work_boundaries.json"
    ))
    .unwrap_or_default();
    assert_eq!(matrix["schema"], "nostr_automerge.work_boundaries.v3");
    assert_eq!(
        matrix["boundaries"],
        serde_json::json!([
            "one_before",
            "exact",
            "one_after",
            "cancelled",
            "atomic_failure",
            "no_post_stop"
        ])
    );
    let required = matrix["required_amount"].as_u64().unwrap_or_default();
    assert_eq!(required, 2);
    let rows = matrix["rows"].as_array().cloned().unwrap_or_default();
    assert_eq!(rows.len(), 10);
    let mut covered = std::collections::BTreeSet::new();
    for row in rows {
        let name = row["counter"].as_str().unwrap_or_default();
        let counter = match name {
            "event" => WorkCounter::Event,
            "carrier" => WorkCounter::Carrier,
            "control" => WorkCounter::Control,
            "graph_node" => WorkCounter::GraphNode,
            "graph_edge" => WorkCounter::GraphEdge,
            "decode_byte" => WorkCounter::DecodeByte,
            "apply_change" => WorkCounter::ApplyChange,
            "checkpoint_byte" => WorkCounter::CheckpointByte,
            "checkpoint_item" => WorkCounter::CheckpointItem,
            "assertion" => WorkCounter::Assertion,
            _ => return,
        };
        assert_eq!(counter.as_str(), name);
        assert!(
            row["phases"]
                .as_array()
                .is_some_and(|phases| !phases.is_empty())
        );
        covered.insert(name.to_owned());
        let bytes = row["capacity"] == "bytes";
        let budget = |capacity| {
            if bytes {
                WorkBudget::new(capacity, 0)
            } else {
                WorkBudget::new(0, capacity)
            }
        };

        let mut before = budget(required - 1);
        let before_state = before;
        assert!(before.charge(counter, required).is_err(), "{name}");
        assert_eq!(before, before_state, "{name}");

        let mut exact = budget(required);
        assert!(exact.charge(counter, required).is_ok(), "{name}");
        assert_eq!(exact.consumed().get(counter), required, "{name}");
        assert_eq!(exact.remaining(), (0, 0), "{name}");

        let mut after = budget(required + 1);
        assert!(after.charge(counter, required).is_ok(), "{name}");
        assert_eq!(after.consumed().get(counter), required, "{name}");
        assert_eq!(after.remaining(), if bytes { (1, 0) } else { (0, 1) });
    }
    assert_eq!(covered.len(), 10);
    let evaluator_source = include_str!("../src/engine/reference_evaluator.rs");
    assert!(evaluator_source.contains("build_control_ancestry_index"));
    assert!(evaluator_source.contains("prepare_interrupted_batch_report"));
    assert!(!evaluator_source.contains("fn checkpoint_refusals"));

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
    let mut cancelled = WorkBudget::new(1_000_000, 1_000);
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &builder.finish(),
        scenario.coordinate,
        &mut cancelled,
        &|| true,
    );
    assert_eq!(report.completion(), Completion::Cancelled);
    assert!(report.document().is_none());
    for counter in [
        WorkCounter::ApplyChange,
        WorkCounter::CheckpointByte,
        WorkCounter::CheckpointItem,
    ] {
        assert_eq!(cancelled.consumed().get(counter), 0, "{}", counter.as_str());
    }
    assert_eq!(cancelled.consumed().get(WorkCounter::Assertion), 0);
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
        let cancelled = Cell::new(false);
        let report = evaluator.evaluate_report(
            &corpus,
            scenario.coordinate,
            &mut WorkBudget::new(1_000_000, 1_000),
            &|| {
                let boundary = calls.get();
                calls.set(boundary + 1);
                if boundary == cancel_at {
                    cancelled.set(true);
                }
                cancelled.get()
            },
        );
        assert_eq!(report.completion(), Completion::Cancelled, "{cancel_at}");
        assert_eq!(report.failure(), Some(EvaluationFailure::Cancelled));
        assert_canonical_control_outcomes_are_consistent(&report);
        assert!(report.document().is_none());
        assert!(
            report
                .heads()
                .iter()
                .all(|head| report.accepted_changes().contains(head))
        );
    }
}

#[test]
fn prior_knowledge_classification_has_cooperative_stop_boundaries() {
    let evaluator = include_str!("../src/engine/reference_evaluator.rs");
    let prior = evaluator
        .split_once("fn additional_prior_knowledge(")
        .and_then(|(_, source)| source.split_once("enum ChangeClaimReason"))
        .map(|(source, _)| source)
        .unwrap_or_default();
    assert!(prior.contains("cancellation: &impl CancellationCheck"));
    assert!(prior.matches("charge_evaluation_work(").count() >= 6);
    cancellation_is_safe_at_every_evaluator_boundary();
}

#[test]
fn every_item_budget_boundary_preserves_canonical_control_outcomes() {
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
    let complete =
        evaluator.evaluate_report(&corpus, scenario.coordinate, &mut measured, &NeverCancelled);
    assert_eq!(complete.completion(), Completion::Complete);
    let consumed_items = 1_000 - measured.remaining().1;

    for item_budget in 0..=consumed_items {
        let report = evaluator.evaluate_report(
            &corpus,
            scenario.coordinate,
            &mut WorkBudget::new(1_000_000, item_budget),
            &NeverCancelled,
        );
        assert_canonical_control_outcomes_are_consistent(&report);
        if report.disposition_records() == complete.disposition_records()
            && report.canonical_controls() == complete.canonical_controls()
            && report.heads() == complete.heads()
        {
            assert_eq!(report.history_digest(), complete.history_digest());
            assert_eq!(report.dispositions_digest(), complete.dispositions_digest());
        }
    }
}

#[test]
fn prior_knowledge_exhaustion_is_deterministic_at_every_item_boundary() {
    every_item_budget_boundary_preserves_canonical_control_outcomes();
    let evaluator = include_str!("../src/engine/reference_evaluator.rs");
    let prior = evaluator
        .split_once("fn additional_prior_knowledge(")
        .and_then(|(_, source)| source.split_once("enum ChangeClaimReason"))
        .map(|(source, _)| source)
        .unwrap_or_default();
    assert!(prior.contains("Completion"));
    assert!(prior.contains("Result<"));
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
fn invalid_start_op_cannot_poison_valid_same_sequence_change() {
    let controller = TestSigner::from_byte(114);
    let device = TestSigner::from_byte(115);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "b4".repeat(32)
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
    let valid = document
        .author_change(&[Operation::PutString {
            key: "winner".to_owned(),
            value: "valid".to_owned(),
        }])
        .expect("valid change");
    let (invalid_raw, invalid_hash) = rewrite_first_change_start_op(valid.raw().to_vec(), 2);
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
        sign_change(2, valid.change_hash(), valid.raw()),
        sign_change(3, invalid_hash, &invalid_raw),
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
    assert_eq!(report.accepted_changes(), [valid.change_hash()]);
    assert!(
        report
            .dispositions()
            .contains(&(valid.change_hash(), ProtocolDisposition::Accepted))
    );
    assert!(
        report
            .dispositions()
            .contains(&(invalid_hash, ProtocolDisposition::Invalid))
    );
    assert!(report.integrity_alerts().is_empty());
    let view = report.document().expect("valid materialized document");
    assert!(view.entries().iter().any(|entry| {
        entry.path() == [MaterializedPathElement::Key("winner".to_owned())]
            && matches!(
                entry.conflicts(),
                [conflict]
                    if conflict.value()
                        == &MaterializedValue::Scalar(MaterializedScalar::String(
                            "valid".to_owned()
                        ))
            )
    }));
}

#[test]
#[allow(clippy::expect_used)]
fn missing_predecessor_cannot_poison_valid_same_sequence_change() {
    let controller = TestSigner::from_byte(116);
    let device = TestSigner::from_byte(117);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "b6".repeat(32)
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
    let state = ActorState::initial(actor, BTreeSet::new());
    let mut valid_document =
        AuthoringDocument::empty(state.clone()).expect("valid authoring document");
    let first = valid_document
        .author_change(&[Operation::PutString {
            key: "first".to_owned(),
            value: "accepted".to_owned(),
        }])
        .expect("first change");
    let valid = valid_document
        .author_change(&[Operation::PutString {
            key: "second".to_owned(),
            value: "accepted".to_owned(),
        }])
        .expect("valid second change");
    let mut missing_document =
        AuthoringDocument::empty(state).expect("missing predecessor document");
    let missing = missing_document
        .author_change(&[Operation::PutString {
            key: "poison".to_owned(),
            value: "invalid".to_owned(),
        }])
        .expect("candidate without predecessor");
    let (missing_raw, missing_hash) = rewrite_change_sequence(missing.raw().to_vec(), 1, 2);
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
        sign_change(2, first.change_hash(), first.raw()),
        sign_change(3, valid.change_hash(), valid.raw()),
        sign_change(4, missing_hash, &missing_raw),
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
        BTreeSet::from([first.change_hash(), valid.change_hash()])
    );
    assert!(
        report
            .dispositions()
            .contains(&(missing_hash, ProtocolDisposition::Invalid))
    );
    assert!(report.integrity_alerts().is_empty());
}

#[test]
#[allow(clippy::expect_used)]
fn base_omission_cannot_poison_valid_same_sequence_change() {
    let controller = TestSigner::from_byte(118);
    let left_device = TestSigner::from_byte(119);
    let right_device = TestSigner::from_byte(120);
    let target_device = TestSigner::from_byte(121);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "b8".repeat(32)
    )
    .parse()
    .expect("fixed coordinate");
    let members = vec![
        (left_device.public_key().to_hex(), vec!["write"]),
        (right_device.public_key().to_hex(), vec!["write"]),
        (target_device.public_key().to_hex(), vec!["write"]),
    ];
    let genesis = signed_acl_control(&controller, coordinate, 1, None, 0, members.clone());
    let genesis_id = VerifiedNip01Event::verify(genesis.clone())
        .expect("signed genesis")
        .event_id();
    let make_base = |device: &TestSigner, key: &str| {
        let actor = ActorId::derive(coordinate, device.public_key());
        let mut document = AutoCommit::new_with_encoding(TextEncoding::Utf16CodeUnit)
            .with_actor(automerge::ActorId::from(actor.as_bytes().to_vec()));
        document.put(ROOT, key, "base").expect("base operation");
        let hash = document.commit().expect("base hash");
        let raw = document
            .get_change_by_hash(&hash)
            .expect("base change")
            .raw_bytes()
            .to_vec();
        (ChangeHash::from_bytes(hash.0), raw)
    };
    let (left_hash, left_raw) = make_base(&left_device, "left");
    let (right_hash, right_raw) = make_base(&right_device, "right");
    let child = signed_acl_control_with_base(
        &controller,
        coordinate,
        6,
        Some(genesis_id),
        1,
        members,
        &[left_hash, right_hash],
    );
    let child_id = VerifiedNip01Event::verify(child.clone())
        .expect("signed child")
        .event_id();
    let target_actor = ActorId::derive(coordinate, target_device.public_key());
    let decoded_left = automerge::Change::from_bytes(left_raw.clone()).expect("decoded left");
    let decoded_right = automerge::Change::from_bytes(right_raw.clone()).expect("decoded right");
    let make_target = |bases: Vec<automerge::Change>, key: &str| {
        let mut document = AutoCommit::new_with_encoding(TextEncoding::Utf16CodeUnit);
        document.apply_changes(bases).expect("apply bases");
        document.set_actor(automerge::ActorId::from(target_actor.as_bytes().to_vec()));
        document
            .put(ROOT, key, "candidate")
            .expect("target operation");
        let hash = document.commit().expect("target hash");
        let raw = document
            .get_change_by_hash(&hash)
            .expect("target change")
            .raw_bytes()
            .to_vec();
        (ChangeHash::from_bytes(hash.0), raw)
    };
    let (valid_hash, valid_raw) = make_target(
        vec![decoded_left.clone(), decoded_right],
        "valid-descendant",
    );
    let (omitting_hash, omitting_raw) = make_target(vec![decoded_left], "omits-right");
    let sign_change = |device: &TestSigner,
                       created_at: u64,
                       control_id: EventId,
                       hash: ChangeHash,
                       raw: &[u8]| {
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
        sign_change(&left_device, 2, genesis_id, left_hash, &left_raw),
        sign_change(&right_device, 3, genesis_id, right_hash, &right_raw),
        sign_change(&target_device, 4, child_id, valid_hash, &valid_raw),
        sign_change(&target_device, 5, child_id, omitting_hash, &omitting_raw),
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
    assert_eq!(
        report
            .accepted_changes()
            .iter()
            .copied()
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([left_hash, right_hash, valid_hash])
    );
    assert!(
        report
            .dispositions()
            .contains(&(omitting_hash, ProtocolDisposition::Invalid))
    );
    assert!(report.integrity_alerts().is_empty());
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
    assert_eq!(report.accepted_changes(), [first.change_hash()]);
    assert!(
        report
            .dispositions()
            .contains(&(first.change_hash(), ProtocolDisposition::Accepted))
    );
    assert!(
        report
            .dispositions()
            .contains(&(rollback_hash, ProtocolDisposition::Invalid))
    );
    assert_eq!(
        report
            .dispositions()
            .iter()
            .filter(|(hash, _)| *hash == first.change_hash())
            .count(),
        1
    );
    assert!(report.integrity_alerts().is_empty());
}

#[test]
fn accepted_base_is_not_quarantined_by_invalid_sequence_reuse() {
    actor_sequence_rollback_and_replay();
}

#[test]
#[allow(clippy::expect_used)]
fn equivocation_regression_case_contracts_are_complete() {
    let cases = [
        include_str!(
            "../../../fixtures/v1_draft/scenarios/equivocation/equivocation_valid_vs_bad_start_op.regression.json"
        ),
        include_str!(
            "../../../fixtures/v1_draft/scenarios/equivocation/equivocation_valid_vs_missing_predecessor.regression.json"
        ),
        include_str!(
            "../../../fixtures/v1_draft/scenarios/equivocation/equivocation_valid_vs_base_omission.regression.json"
        ),
        include_str!(
            "../../../fixtures/v1_draft/scenarios/equivocation/equivocation_reused_base_sequence.regression.json"
        ),
        include_str!(
            "../../../fixtures/v1_draft/scenarios/equivocation/equivocation_two_otherwise_valid.regression.json"
        ),
    ];
    let parsed = cases
        .into_iter()
        .map(|case| serde_json::from_str::<serde_json::Value>(case).expect("regression case"))
        .collect::<Vec<_>>();
    assert_eq!(parsed.len(), 5);
    assert!(parsed.iter().all(|case| {
        case["schema"] == "nostr_automerge.regression_case.v1"
            && case["requirements"]
                .as_array()
                .is_some_and(|requirements| !requirements.is_empty())
            && case["expected"].is_object()
    }));
    assert_eq!(
        parsed
            .iter()
            .filter(|case| case["expected"]["integrity_alerts"] == 0)
            .count(),
        4
    );
    assert_eq!(parsed[4]["expected"]["integrity_alerts"], 1);
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
    accepted_base_is_not_quarantined_by_invalid_sequence_reuse();
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

#[test]
#[allow(clippy::expect_used)]
fn unrelated_coordinate_evidence_is_report_and_budget_inert() {
    let target = signed_engine_scenario();
    let unrelated = TestSigner::from_byte(122);
    let unrelated_document = "d1".repeat(32);
    let unrelated_coordinate: DocumentCoordinate = format!(
        "31624:{}:{unrelated_document}",
        unrelated.public_key().to_hex()
    )
    .parse()
    .expect("unrelated coordinate");
    assert_ne!(target.coordinate, unrelated_coordinate);
    let unrelated_manifest = unrelated.sign(
        &UnsignedEventDraft::new(
            9,
            31_624,
            vec![vec!["d".to_owned(), unrelated_document]],
            format!(
                r#"{{"application":null,"checkpoint":null,"control":"{}","description":null,"format":"automerge-change-v1","name":null,"relays":[],"status":"active","successor":null,"text_encoding":"utf16","v":2}}"#,
                "d2".repeat(32)
            ),
        )
        .expect("unrelated manifest")
        .prepare(unrelated.public_key())
        .expect("unrelated preimage"),
    );
    let evaluate = |include_unrelated: bool| {
        let mut builder = CorpusBuilder::new();
        assert!(matches!(
            builder.ingest(target.control.clone()),
            IngestOutcome::Accepted { .. }
        ));
        assert!(matches!(
            builder.ingest(target.change.clone()),
            IngestOutcome::Accepted { .. }
        ));
        if include_unrelated {
            assert!(matches!(
                builder.ingest(unrelated_manifest.clone()),
                IngestOutcome::UnsupportedRevision { .. }
            ));
            assert!(matches!(
                builder.ingest(unrelated_manifest.clone()),
                IngestOutcome::Duplicate { .. }
            ));
            assert!(matches!(
                builder.ingest_bytes(b"{}"),
                IngestOutcome::Invalid { .. }
            ));
        }
        let corpus = builder.finish();
        if include_unrelated {
            assert!(matches!(
                corpus.selected_manifest(target.coordinate),
                nostr_automerge::ManifestAvailability::Missing
            ));
            assert!(matches!(
                corpus.selected_manifest(unrelated_coordinate),
                nostr_automerge::ManifestAvailability::Unavailable { .. }
            ));
        }
        let mut budget = WorkBudget::new(10_000_000, 10_000_000);
        let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
            &corpus,
            target.coordinate,
            &mut budget,
            &NeverCancelled,
        );
        (report, budget.consumed())
    };
    let (target_only, target_work) = evaluate(false);
    let (with_unrelated, unrelated_work) = evaluate(true);
    assert_eq!(target_only.completion(), with_unrelated.completion());
    assert_eq!(
        target_only.history_digest(),
        with_unrelated.history_digest()
    );
    assert_eq!(
        target_only.disposition_records(),
        with_unrelated.disposition_records()
    );
    assert_eq!(
        target_only.dispositions_digest(),
        with_unrelated.dispositions_digest()
    );
    assert_eq!(target_only.evidence(), with_unrelated.evidence());
    assert_eq!(target_work, unrelated_work);
}

#[allow(clippy::expect_used)]
fn signed_engine_scenario() -> SignedEngineScenario {
    signed_engine_scenario_with_change_tags(Vec::new())
}

#[allow(clippy::expect_used)]
fn signed_engine_scenario_with_change_tags(extra_tags: Vec<Vec<String>>) -> SignedEngineScenario {
    signed_engine_scenario_with_roles(extra_tags, r#"["checkpoint","write"]"#)
}

#[allow(clippy::expect_used)]
fn signed_engine_scenario_with_roles(
    extra_tags: Vec<Vec<String>>,
    roles_json: &str,
) -> SignedEngineScenario {
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
        r#"{{"base_heads":[],"format":"automerge-change-v1","members":[{{"account":null,"pubkey":"{}","roles":{}}}],"policy":"controller-acl-v1","predecessor":null,"seq":0,"successor":null,"text_encoding":"utf16","v":1}}"#,
        device.public_key().to_hex(),
        roles_json,
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
fn unknown_change_tags_preserve_semantics_but_change_carrier_identity() {
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
        assert_eq!(report.dispositions(), baseline.dispositions());
        assert_eq!(report.accepted_changes(), baseline.accepted_changes());
        assert_eq!(report.heads(), baseline.heads());
        assert_eq!(report.history_digest(), baseline.history_digest());
        assert_ne!(report.dispositions_digest(), baseline.dispositions_digest());
        let carrier_records = report
            .disposition_records()
            .iter()
            .filter(|record| matches!(record.identifier(), ProtocolItemIdentifier::Event(_)))
            .collect::<Vec<_>>();
        assert_eq!(carrier_records.len(), 1);
        assert_eq!(
            carrier_records[0].disposition(),
            ProtocolDisposition::Accepted
        );
    }
}

#[test]
#[allow(clippy::expect_used)]
fn signed_single_chunk_checkpoint_verifies_real_automerge_history() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/v1_draft/checkpoints/cases.json"
    ))
    .expect("checkpoint fixture family");
    assert_eq!(
        fixture["accepted_state_source"].as_str(),
        Some("exact_parent_epoch_closure_at_referenced_control")
    );
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
    assert_checkpoint_event_dispositions(&report, ProtocolDisposition::Accepted);
    assert_eq!(checkpoint.completion(), Completion::Complete);
    assert_eq!(checkpoint.historical_carriers(), [scenario.change_hash]);
    assert_eq!(checkpoint.accepted_at_control(), [scenario.change_hash]);
    assert!(budget.consumed().get(WorkCounter::CheckpointByte) > 0);
    assert!(budget.consumed().get(WorkCounter::CheckpointItem) > 0);
}

#[test]
#[allow(clippy::expect_used)]
fn orphan_checkpoint_chunk_promotes_after_descriptor_arrival() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/v1_draft/scenarios/checkpoints/checkpoints_single_chunk.input.json"
    ))
    .expect("signed checkpoint scenario");
    let coordinate: DocumentCoordinate = fixture["coordinate"]
        .as_str()
        .expect("fixture coordinate")
        .parse()
        .expect("valid fixture coordinate");
    let raw_events = fixture["raw_events"]
        .as_array()
        .expect("fixture raw events");
    let chunk = raw_events
        .iter()
        .find(|entry| {
            entry["data"]
                .as_str()
                .and_then(|data| serde_json::from_str::<serde_json::Value>(data).ok())
                .and_then(|event| event["kind"].as_u64())
                == Some(1_627)
        })
        .and_then(|entry| entry["data"].as_str())
        .expect("signed chunk");
    let chunk_id = serde_json::from_str::<serde_json::Value>(chunk)
        .expect("chunk event")
        .get("id")
        .and_then(serde_json::Value::as_str)
        .expect("chunk identifier")
        .parse()
        .expect("valid chunk identifier");

    let build = |include_descriptor: bool| {
        let mut builder = CorpusBuilder::new();
        for entry in raw_events {
            let data = entry["data"].as_str().expect("signed event bytes");
            let kind = serde_json::from_str::<serde_json::Value>(data)
                .expect("signed event")
                .get("kind")
                .and_then(serde_json::Value::as_u64)
                .expect("event kind");
            if include_descriptor || kind != 1_626 {
                assert!(matches!(
                    builder.ingest_bytes(data.as_bytes()),
                    IngestOutcome::Accepted { .. }
                ));
            }
        }
        builder.finish()
    };
    let evaluator = ReferenceEvaluator::new(ProtocolRevision::draft_v1());
    let initial = evaluator.evaluate_report(
        &build(false),
        coordinate,
        &mut WorkBudget::new(1_000_000, 1_000_000),
        &NeverCancelled,
    );
    assert_eq!(
        event_disposition(&initial, chunk_id),
        Some(ProtocolDisposition::Pending)
    );
    assert!(initial.checkpoints().is_empty());

    let promoted = evaluator.reevaluate_report(
        &build(true),
        coordinate,
        &initial,
        &mut WorkBudget::new(1_000_000, 1_000_000),
        &NeverCancelled,
    );
    assert_eq!(
        event_disposition(&promoted, chunk_id),
        Some(ProtocolDisposition::Accepted)
    );
    assert_eq!(
        promoted.checkpoints().first().map(|result| result.status()),
        Some(CheckpointVerificationStatus::Verified)
    );
}

#[test]
#[allow(clippy::expect_used)]
fn descriptor_reference_evidence_delivery_permutations_converge() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/v1_draft/scenarios/checkpoints/checkpoints_single_chunk.input.json"
    ))
    .expect("signed checkpoint scenario");
    let coordinate: DocumentCoordinate = fixture["coordinate"]
        .as_str()
        .expect("fixture coordinate")
        .parse()
        .expect("valid fixture coordinate");
    let raw_events = fixture["raw_events"]
        .as_array()
        .expect("fixture raw events");
    let orders = [
        vec![0, 1, 2, 3],
        vec![3, 2, 1, 0],
        vec![3, 0, 2, 1],
        vec![2, 3, 0, 1],
        vec![3, 3, 2, 1, 0],
    ];
    let evaluate = |order: &[usize]| {
        let mut builder = CorpusBuilder::new();
        for index in order {
            let data = raw_events[*index]["data"]
                .as_str()
                .expect("signed event bytes");
            assert!(matches!(
                builder.ingest_bytes(data.as_bytes()),
                IngestOutcome::Accepted { .. } | IngestOutcome::Duplicate { .. }
            ));
        }
        ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
            &builder.finish(),
            coordinate,
            &mut WorkBudget::new(1_000_000, 1_000_000),
            &NeverCancelled,
        )
    };
    let baseline = evaluate(&orders[0]);
    for order in &orders[1..] {
        let report = evaluate(order);
        assert_eq!(report.canonical_controls(), baseline.canonical_controls());
        assert_eq!(report.disposition_records(), baseline.disposition_records());
        assert_eq!(report.checkpoints(), baseline.checkpoints());
        assert_eq!(report.heads(), baseline.heads());
        assert_eq!(report.history_digest(), baseline.history_digest());
        assert_eq!(report.dispositions_digest(), baseline.dispositions_digest());
        assert_eq!(report.document(), baseline.document());
    }
}

#[test]
#[allow(clippy::expect_used)]
fn signed_empty_history_checkpoint_verifies_without_redefining_history() {
    let controller = TestSigner::from_byte(60);
    let checkpoint_signer = TestSigner::from_byte(61);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "60".repeat(32)
    )
    .parse()
    .expect("fixed coordinate");
    let actor = ActorId::derive(coordinate, checkpoint_signer.public_key());
    let snapshot = AuthoringDocument::empty(ActorState::initial(actor, BTreeSet::new()))
        .expect("empty document")
        .accepted_state_bytes();
    assert!(!snapshot.is_empty());

    let control = controller.sign(
        &UnsignedEventDraft::new(
            1,
            1_625,
            vec![vec!["a".to_owned(), coordinate.to_address()]],
            format!(
                r#"{{"base_heads":[],"format":"automerge-change-v1","members":[{{"account":null,"pubkey":"{}","roles":["checkpoint"]}}],"policy":"controller-acl-v1","predecessor":null,"seq":0,"successor":null,"text_encoding":"utf16","v":1}}"#,
                checkpoint_signer.public_key().to_hex()
            ),
        )
        .expect("control draft")
        .prepare(controller.public_key())
        .expect("control preimage"),
    );
    let control_id = VerifiedNip01Event::verify(control.clone())
        .expect("signed control")
        .event_id();
    let snapshot_hash: [u8; 32] = Sha256::digest(&snapshot).into();
    let chunk_hash = snapshot_hash;
    let chunk_root = nostr_automerge::checkpoint::leaf_hash(0, 1, chunk_hash);
    let empty_change_set_hash = Sha256::digest(
        [
            b"nostr-crdt/automerge/change-set/v1".as_slice(),
            &[0],
            &0_u64.to_be_bytes(),
        ]
        .concat(),
    );
    let descriptor = checkpoint_signer.sign(
        &UnsignedEventDraft::new(
            2,
            1_626,
            vec![
                vec!["a".to_owned(), coordinate.to_address()],
                vec!["e".to_owned(), control_id.to_hex()],
                vec!["x".to_owned(), hex32(snapshot_hash)],
            ],
            format!(
                r#"{{"change_count":0,"change_set_hash":"{}","chunk_count":1,"chunk_root":"{}","chunk_size":{},"dependency_edges":0,"encoding":"automerge-save-v1","heads":[],"raw_size":{},"total_ops":0,"v":1}}"#,
                hex32(empty_change_set_hash.into()),
                hex32(chunk_root),
                snapshot.len(),
                snapshot.len(),
            ),
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
            3,
            1_627,
            vec![
                vec!["a".to_owned(), coordinate.to_address()],
                vec!["e".to_owned(), descriptor_id.to_hex()],
                vec!["x".to_owned(), hex32(chunk_hash)],
                vec!["part".to_owned(), "0".to_owned(), "1".to_owned()],
            ],
            format!(
                r#"{{"data":"{}","proof":[],"v":1}}"#,
                base64::engine::general_purpose::STANDARD.encode(&snapshot)
            ),
        )
        .expect("chunk draft")
        .prepare(checkpoint_signer.public_key())
        .expect("chunk preimage"),
    );
    let mut builder = CorpusBuilder::new();
    for event in [chunk, descriptor, control] {
        let outcome = builder.ingest(event);
        assert!(
            matches!(outcome, IngestOutcome::Accepted { .. }),
            "unexpected ingest outcome: {outcome:?}"
        );
    }
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &builder.finish(),
        coordinate,
        &mut WorkBudget::new(1_000_000, 1_000_000),
        &NeverCancelled,
    );
    let checkpoint = report.checkpoints().first().expect("checkpoint result");
    assert_eq!(checkpoint.descriptor_event(), descriptor_id);
    assert_eq!(checkpoint.status(), CheckpointVerificationStatus::Verified);
    assert_checkpoint_event_dispositions(&report, ProtocolDisposition::Accepted);
    assert!(checkpoint.heads().is_empty());
    assert!(checkpoint.historical_carriers().is_empty());
    assert!(checkpoint.accepted_at_control().is_empty());
    assert!(report.accepted_changes().is_empty());
    assert!(report.heads().is_empty());
    assert!(report.document().is_some());
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
    assert_checkpoint_event_dispositions(&report, ProtocolDisposition::Accepted);
    assert_eq!(checkpoint.chunk_events().len(), count as usize);
    assert_eq!(checkpoint.snapshot_hash().as_bytes(), &snapshot_hash);
    assert_eq!(checkpoint.heads(), [scenario.change_hash]);
}

fn hex32(bytes: [u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[allow(clippy::expect_used)]
fn signed_single_chunk_checkpoint_for_scenario(
    scenario: &SignedEngineScenario,
    signer: &TestSigner,
) -> (RawEventBytes, EventId, RawEventBytes, EventId) {
    let snapshot_hash: [u8; 32] = Sha256::digest(&scenario.snapshot).into();
    let chunk_root = nostr_automerge::checkpoint::leaf_hash(0, 1, snapshot_hash);
    let mut change_set = Sha256::new();
    change_set.update(b"nostr-crdt/automerge/change-set/v1");
    change_set.update([0]);
    change_set.update(1_u64.to_be_bytes());
    change_set.update(scenario.change_hash.as_bytes());
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
                r#"{{"change_count":1,"change_set_hash":"{}","chunk_count":1,"chunk_root":"{}","chunk_size":{},"dependency_edges":0,"encoding":"automerge-save-v1","heads":["{}"],"raw_size":{},"total_ops":1,"v":1}}"#,
                hex32(change_set.finalize().into()),
                hex32(chunk_root),
                scenario.snapshot.len(),
                scenario.change_hash.to_hex(),
                scenario.snapshot.len(),
            ),
        )
        .expect("descriptor draft")
        .prepare(signer.public_key())
        .expect("descriptor preimage"),
    );
    let descriptor_id = VerifiedNip01Event::verify(descriptor.clone())
        .expect("signed descriptor")
        .event_id();
    let chunk = signer.sign(
        &UnsignedEventDraft::new(
            4,
            1_627,
            vec![
                vec!["a".to_owned(), scenario.coordinate.to_address()],
                vec!["e".to_owned(), descriptor_id.to_hex()],
                vec!["x".to_owned(), hex32(snapshot_hash)],
                vec!["part".to_owned(), "0".to_owned(), "1".to_owned()],
            ],
            format!(
                r#"{{"data":"{}","proof":[],"v":1}}"#,
                base64::engine::general_purpose::STANDARD.encode(&scenario.snapshot)
            ),
        )
        .expect("chunk draft")
        .prepare(signer.public_key())
        .expect("chunk preimage"),
    );
    let chunk_id = VerifiedNip01Event::verify(chunk.clone())
        .expect("signed chunk")
        .event_id();
    (descriptor, descriptor_id, chunk, chunk_id)
}

struct SignedCheckpointRoleEvaluation {
    report: EvaluationReport,
    work: WorkCounters,
    cancellation_checks: u64,
    descriptor_id: EventId,
    chunk_id: EventId,
    control_id: EventId,
    change_hash: ChangeHash,
}

#[allow(clippy::expect_used)]
fn evaluate_signed_checkpoint_role_case(
    roles_json: &str,
    descriptor_signer: &TestSigner,
    order: &[usize],
    evaluation_control: CheckpointEvaluationControl,
) -> SignedCheckpointRoleEvaluation {
    let scenario = signed_engine_scenario_with_roles(Vec::new(), roles_json);
    let (descriptor, descriptor_id, chunk, chunk_id) =
        signed_single_chunk_checkpoint_for_scenario(&scenario, descriptor_signer);
    let events = [scenario.control, scenario.change, descriptor, chunk];
    let mut builder = CorpusBuilder::new();
    for index in order {
        assert!(matches!(
            builder.ingest(events[*index].clone()),
            IngestOutcome::Accepted { .. } | IngestOutcome::Duplicate { .. }
        ));
    }
    let item_budget = match evaluation_control {
        CheckpointEvaluationControl::ItemBudget(limit) => limit,
        CheckpointEvaluationControl::Normal | CheckpointEvaluationControl::CancelAfter(_) => {
            1_000_000
        }
    };
    let cancellation_checks = Cell::new(0_u64);
    let cancellation = || {
        let current = cancellation_checks.get();
        cancellation_checks.set(current.saturating_add(1));
        matches!(
            evaluation_control,
            CheckpointEvaluationControl::CancelAfter(limit) if current >= limit
        )
    };
    let mut budget = WorkBudget::new(1_000_000, item_budget);
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &builder.finish(),
        scenario.coordinate,
        &mut budget,
        &cancellation,
    );
    SignedCheckpointRoleEvaluation {
        report,
        work: budget.consumed(),
        cancellation_checks: cancellation_checks.get(),
        descriptor_id,
        chunk_id,
        control_id: scenario.control_id,
        change_hash: scenario.change_hash,
    }
}

#[derive(Clone, Copy)]
struct CheckpointCommitmentVariant {
    head: ChangeHash,
    change_count: u64,
    change_set_hash: [u8; 32],
    dependency_edges: u64,
    total_ops: u64,
}

#[derive(Clone, Copy)]
enum CheckpointHistoryVariant {
    Accepted,
    MissingCarrier,
    NotAccepted,
}

#[derive(Clone, Copy)]
enum CheckpointEvaluationControl {
    Normal,
    ItemBudget(u64),
    CancelAfter(u64),
}

#[allow(clippy::expect_used, clippy::too_many_arguments)]
fn evaluate_single_chunk_variant(
    chunk_signer: &TestSigner,
    chunk_coordinate: Option<DocumentCoordinate>,
    chunk_descriptor: Option<EventId>,
    parts: &[(u64, &str, &str)],
    chunk_data: Option<&[u8]>,
    snapshot_hash_override: Option<[u8; 32]>,
    chunk_root_override: Option<[u8; 32]>,
    commitments_override: Option<CheckpointCommitmentVariant>,
    history_variant: CheckpointHistoryVariant,
) -> nostr_automerge::EvaluationReport {
    evaluate_single_chunk_variant_controlled(
        chunk_signer,
        chunk_coordinate,
        chunk_descriptor,
        parts,
        chunk_data,
        snapshot_hash_override,
        chunk_root_override,
        commitments_override,
        history_variant,
        CheckpointEvaluationControl::Normal,
    )
    .0
}

#[allow(clippy::expect_used, clippy::too_many_arguments)]
fn evaluate_single_chunk_variant_controlled(
    chunk_signer: &TestSigner,
    chunk_coordinate: Option<DocumentCoordinate>,
    chunk_descriptor: Option<EventId>,
    parts: &[(u64, &str, &str)],
    chunk_data: Option<&[u8]>,
    snapshot_hash_override: Option<[u8; 32]>,
    chunk_root_override: Option<[u8; 32]>,
    commitments_override: Option<CheckpointCommitmentVariant>,
    history_variant: CheckpointHistoryVariant,
    evaluation_control: CheckpointEvaluationControl,
) -> (
    nostr_automerge::EvaluationReport,
    nostr_automerge::WorkCounters,
    u64,
) {
    let scenario = match history_variant {
        CheckpointHistoryVariant::Accepted | CheckpointHistoryVariant::MissingCarrier => {
            signed_engine_scenario()
        }
        CheckpointHistoryVariant::NotAccepted => {
            signed_engine_scenario_with_roles(Vec::new(), r#"["checkpoint"]"#)
        }
    };
    let descriptor_signer = TestSigner::from_byte(21);
    let snapshot_hash: [u8; 32] = Sha256::digest(&scenario.snapshot).into();
    let chunk_hash = snapshot_hash;
    let chunk_root = nostr_automerge::checkpoint::leaf_hash(0, 1, chunk_hash);
    let mut change_set = Sha256::new();
    change_set.update(b"nostr-crdt/automerge/change-set/v1");
    change_set.update([0]);
    change_set.update(1_u64.to_be_bytes());
    change_set.update(scenario.change_hash.as_bytes());
    let commitments = commitments_override.unwrap_or(CheckpointCommitmentVariant {
        head: scenario.change_hash,
        change_count: 1,
        change_set_hash: change_set.finalize().into(),
        dependency_edges: 0,
        total_ops: 1,
    });
    let descriptor = descriptor_signer.sign(
        &UnsignedEventDraft::new(
            3,
            1_626,
            vec![
                vec!["a".to_owned(), scenario.coordinate.to_address()],
                vec!["e".to_owned(), scenario.control_id.to_hex()],
                vec![
                    "x".to_owned(),
                    hex32(snapshot_hash_override.unwrap_or(snapshot_hash)),
                ],
            ],
            format!(
                r#"{{"change_count":{},"change_set_hash":"{}","chunk_count":1,"chunk_root":"{}","chunk_size":{},"dependency_edges":{},"encoding":"automerge-save-v1","heads":["{}"],"raw_size":{},"total_ops":{},"v":1}}"#,
                commitments.change_count,
                hex32(commitments.change_set_hash),
                hex32(chunk_root_override.unwrap_or(chunk_root)),
                scenario.snapshot.len(),
                commitments.dependency_edges,
                commitments.head.to_hex(),
                scenario.snapshot.len(),
                commitments.total_ops,
            ),
        )
        .expect("descriptor draft")
        .prepare(descriptor_signer.public_key())
        .expect("descriptor preimage"),
    );
    let descriptor_id = VerifiedNip01Event::verify(descriptor.clone())
        .expect("descriptor")
        .event_id();
    let chunks = parts.iter().map(|(created_at, index, count)| {
        let data = chunk_data.unwrap_or(&scenario.snapshot);
        let data_hash: [u8; 32] = Sha256::digest(data).into();
        chunk_signer.sign(
            &UnsignedEventDraft::new(
                *created_at,
                1_627,
                vec![
                    vec![
                        "a".to_owned(),
                        chunk_coordinate.unwrap_or(scenario.coordinate).to_address(),
                    ],
                    vec![
                        "e".to_owned(),
                        chunk_descriptor.unwrap_or(descriptor_id).to_hex(),
                    ],
                    vec!["x".to_owned(), hex32(data_hash)],
                    vec!["part".to_owned(), (*index).to_owned(), (*count).to_owned()],
                ],
                format!(
                    r#"{{"data":"{}","proof":[],"v":1}}"#,
                    base64::engine::general_purpose::STANDARD.encode(data)
                ),
            )
            .expect("chunk draft")
            .prepare(chunk_signer.public_key())
            .expect("chunk preimage"),
        )
    });
    let mut events = chunks.collect::<Vec<_>>();
    events.push(descriptor);
    if !matches!(history_variant, CheckpointHistoryVariant::MissingCarrier) {
        events.push(scenario.change);
    }
    events.push(scenario.control);
    let mut builder = CorpusBuilder::new();
    for event in events {
        assert!(matches!(
            builder.ingest(event),
            IngestOutcome::Accepted { .. }
        ));
    }
    let item_budget = match evaluation_control {
        CheckpointEvaluationControl::ItemBudget(limit) => limit,
        CheckpointEvaluationControl::Normal | CheckpointEvaluationControl::CancelAfter(_) => {
            1_000_000
        }
    };
    let cancellation_checks = std::cell::Cell::new(0_u64);
    let cancellation = || {
        let current = cancellation_checks.get();
        cancellation_checks.set(current.saturating_add(1));
        matches!(
            evaluation_control,
            CheckpointEvaluationControl::CancelAfter(limit) if current >= limit
        )
    };
    let mut budget = WorkBudget::new(1_000_000, item_budget);
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &builder.finish(),
        scenario.coordinate,
        &mut budget,
        &cancellation,
    );
    (report, budget.consumed(), cancellation_checks.get())
}

#[test]
#[allow(clippy::expect_used)]
fn signed_checkpoint_role_gate_is_exact_and_delivery_order_independent() {
    let orders: [&[usize]; 8] = [
        &[0, 1, 2, 3],
        &[3, 2, 1, 0],
        &[2, 0, 3, 1],
        &[1, 3, 0, 2],
        &[0, 0, 1, 1, 2, 2, 3, 3],
        &[0, 2, 3, 1],
        &[1, 2, 3, 0],
        &[2, 3, 1, 0],
    ];
    let authorized_device = TestSigner::from_byte(21);
    let controller = TestSigner::from_byte(20);
    let cases = [
        (
            r#"["checkpoint","write"]"#,
            &authorized_device,
            CheckpointVerificationStatus::Verified,
            "checkpoint role",
        ),
        (
            r#"["write"]"#,
            &authorized_device,
            CheckpointVerificationStatus::Unauthorized,
            "other role",
        ),
        (
            r#"["checkpoint","write"]"#,
            &controller,
            CheckpointVerificationStatus::Unauthorized,
            "controller without grant",
        ),
    ];

    for (roles, signer, expected_status, case_name) in cases {
        let baseline = evaluate_signed_checkpoint_role_case(
            roles,
            signer,
            orders[0],
            CheckpointEvaluationControl::Normal,
        );
        assert_eq!(
            baseline.report.completion(),
            Completion::Complete,
            "{case_name}"
        );
        assert_eq!(
            baseline.report.canonical_controls(),
            [baseline.control_id],
            "{case_name}"
        );
        assert_eq!(
            baseline
                .report
                .control_dispositions()
                .iter()
                .find(|(event_id, _)| *event_id == baseline.control_id)
                .map(|(_, disposition)| *disposition),
            Some(ProtocolDisposition::Accepted),
            "{case_name}"
        );
        let checkpoint = baseline
            .report
            .checkpoints()
            .iter()
            .find(|checkpoint| checkpoint.descriptor_event() == baseline.descriptor_id)
            .expect("checkpoint role result");
        assert_eq!(checkpoint.status(), expected_status, "{case_name}");
        assert_eq!(
            checkpoint.chunk_events(),
            [baseline.chunk_id],
            "{case_name}"
        );
        assert_eq!(
            checkpoint.historical_carriers(),
            [baseline.change_hash],
            "{case_name}"
        );
        assert_eq!(
            checkpoint.accepted_at_control(),
            [baseline.change_hash],
            "{case_name}"
        );

        let (event_outcome, expected_diagnostic, performed_snapshot_work) =
            if expected_status == CheckpointVerificationStatus::Verified {
                (ProtocolDisposition::Accepted, None, true)
            } else {
                (
                    ProtocolDisposition::Invalid,
                    Some("checkpoint.history"),
                    false,
                )
            };
        for event_id in [baseline.descriptor_id, baseline.chunk_id] {
            assert_eq!(
                event_disposition(&baseline.report, event_id),
                Some(event_outcome),
                "{case_name}"
            );
            assert_eq!(
                event_diagnostic(&baseline.report, event_id),
                expected_diagnostic,
                "{case_name}"
            );
        }
        assert_eq!(
            baseline.work.get(WorkCounter::CheckpointByte) > 0,
            performed_snapshot_work,
            "{case_name}"
        );
        assert!(
            baseline.work.get(WorkCounter::CheckpointItem) > 0,
            "{case_name}"
        );
        assert!(baseline.cancellation_checks > 0, "{case_name}");

        for (order_index, order) in orders.iter().enumerate().skip(1) {
            let permuted = evaluate_signed_checkpoint_role_case(
                roles,
                signer,
                order,
                CheckpointEvaluationControl::Normal,
            );
            assert_eq!(
                permuted.report.canonical_controls(),
                baseline.report.canonical_controls(),
                "{case_name}"
            );
            assert_eq!(
                permuted.report.disposition_records(),
                baseline.report.disposition_records(),
                "{case_name}"
            );
            assert_eq!(
                permuted.report.checkpoints(),
                baseline.report.checkpoints(),
                "{case_name}"
            );
            assert_eq!(
                permuted.report.heads(),
                baseline.report.heads(),
                "{case_name}"
            );
            assert_eq!(
                permuted.report.history_digest(),
                baseline.report.history_digest(),
                "{case_name}"
            );
            assert_eq!(
                permuted.report.document(),
                baseline.report.document(),
                "{case_name}"
            );
            assert_eq!(
                permuted.work.get(WorkCounter::CheckpointByte),
                baseline.work.get(WorkCounter::CheckpointByte),
                "{case_name}"
            );
            assert_eq!(
                permuted.work.get(WorkCounter::CheckpointItem),
                baseline.work.get(WorkCounter::CheckpointItem),
                "{case_name}"
            );
            if order_index == 4 {
                assert!(
                    permuted.work.get(WorkCounter::Event) > baseline.work.get(WorkCounter::Event),
                    "{case_name}"
                );
                assert_eq!(
                    permuted.cancellation_checks, baseline.cancellation_checks,
                    "{case_name}"
                );
            } else {
                assert_eq!(permuted.work, baseline.work, "{case_name}");
                assert_eq!(
                    permuted.cancellation_checks, baseline.cancellation_checks,
                    "{case_name}"
                );
            }
        }

        let cancelled = evaluate_signed_checkpoint_role_case(
            roles,
            signer,
            orders[0],
            CheckpointEvaluationControl::CancelAfter(0),
        );
        assert_eq!(
            cancelled.report.completion(),
            Completion::Cancelled,
            "{case_name}"
        );
        assert_eq!(
            cancelled.work.get(WorkCounter::CheckpointByte),
            0,
            "{case_name}"
        );
    }
}

#[test]
fn checkpoint_author_and_binding_refusals() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/v1_draft/checkpoints/negative_binding.json"
    ))
    .unwrap_or_default();
    assert_eq!(fixture["cases"].as_array().map(Vec::len), Some(3));
    checkpoint_unauthorized_signed_fixture();
    let wrong_author = evaluate_single_chunk_variant(
        &TestSigner::from_byte(62),
        None,
        None,
        &[(4, "0", "1")],
        None,
        None,
        None,
        None,
        CheckpointHistoryVariant::Accepted,
    );
    assert_eq!(
        wrong_author.checkpoints()[0].status(),
        CheckpointVerificationStatus::ChunkAuthorMismatch
    );
    assert_checkpoint_event_dispositions(&wrong_author, ProtocolDisposition::Invalid);

    let other_coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        TestSigner::from_byte(20).public_key().to_hex(),
        "ff".repeat(32)
    )
    .parse()
    .unwrap_or_else(|_| signed_engine_scenario().coordinate);
    let wrong_coordinate = evaluate_single_chunk_variant(
        &TestSigner::from_byte(21),
        Some(other_coordinate),
        None,
        &[(4, "0", "1")],
        None,
        None,
        None,
        None,
        CheckpointHistoryVariant::Accepted,
    );
    assert_eq!(
        wrong_coordinate.checkpoints()[0].status(),
        CheckpointVerificationStatus::MissingChunk
    );
    assert_checkpoint_event_dispositions(&wrong_coordinate, ProtocolDisposition::Pending);

    let wrong_descriptor = evaluate_single_chunk_variant(
        &TestSigner::from_byte(21),
        None,
        Some(EventId::from_bytes([0x77; 32])),
        &[(4, "0", "1")],
        None,
        None,
        None,
        None,
        CheckpointHistoryVariant::Accepted,
    );
    assert_eq!(
        wrong_descriptor.checkpoints()[0].status(),
        CheckpointVerificationStatus::MissingChunk
    );
    assert_checkpoint_event_dispositions(&wrong_descriptor, ProtocolDisposition::Pending);
}

#[test]
fn checkpoint_index_refusals() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/v1_draft/checkpoints/negative_indices.json"
    ))
    .unwrap_or_default();
    assert_eq!(fixture["cases"].as_array().map(Vec::len), Some(5));
    let signer = TestSigner::from_byte(21);
    let duplicate = evaluate_single_chunk_variant(
        &signer,
        None,
        None,
        &[(4, "0", "1"), (5, "0", "1")],
        None,
        None,
        None,
        None,
        CheckpointHistoryVariant::Accepted,
    );
    assert_eq!(
        duplicate.checkpoints()[0].status(),
        CheckpointVerificationStatus::DuplicateChunk
    );
    assert_checkpoint_event_dispositions(&duplicate, ProtocolDisposition::Invalid);
    let missing = evaluate_single_chunk_variant(
        &signer,
        None,
        None,
        &[],
        None,
        None,
        None,
        None,
        CheckpointHistoryVariant::Accepted,
    );
    assert_eq!(
        missing.checkpoints()[0].status(),
        CheckpointVerificationStatus::MissingChunk
    );
    assert_checkpoint_event_dispositions(&missing, ProtocolDisposition::Pending);
    let wrong_count = evaluate_single_chunk_variant(
        &signer,
        None,
        None,
        &[(4, "0", "2")],
        None,
        None,
        None,
        None,
        CheckpointHistoryVariant::Accepted,
    );
    assert_eq!(
        wrong_count.checkpoints()[0].status(),
        CheckpointVerificationStatus::ChunkCountMismatch
    );
    assert_checkpoint_event_dispositions(&wrong_count, ProtocolDisposition::Invalid);
    validated_checkpoint_chunk_carrier_enters_corpus();
}

#[test]
fn checkpoint_size_refusals() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/v1_draft/checkpoints/negative_sizes.json"
    ))
    .unwrap_or_default();
    assert_eq!(fixture["cases"].as_array().map(Vec::len), Some(3));
    let scenario = signed_engine_scenario();
    let shortened = &scenario.snapshot[..scenario.snapshot.len() - 1];
    let report = evaluate_single_chunk_variant(
        &TestSigner::from_byte(21),
        None,
        None,
        &[(4, "0", "1")],
        Some(shortened),
        None,
        None,
        None,
        CheckpointHistoryVariant::Accepted,
    );
    assert_eq!(
        report.checkpoints()[0].status(),
        CheckpointVerificationStatus::ChunkSizeMismatch
    );
    validated_checkpoint_descriptor_carrier_enters_corpus();
    validated_checkpoint_chunk_carrier_enters_corpus();
}

#[test]
fn checkpoint_merkle_refusals() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/v1_draft/checkpoints/negative_merkle.json"
    ))
    .unwrap_or_default();
    assert_eq!(fixture["cases"].as_array().map(Vec::len), Some(4));
    let scenario = signed_engine_scenario();
    let mut mutated = scenario.snapshot.clone();
    mutated[0] ^= 1;
    let merkle = evaluate_single_chunk_variant(
        &TestSigner::from_byte(21),
        None,
        None,
        &[(4, "0", "1")],
        Some(&mutated),
        None,
        None,
        None,
        CheckpointHistoryVariant::Accepted,
    );
    assert_eq!(
        merkle.checkpoints()[0].status(),
        CheckpointVerificationStatus::MerkleMismatch
    );
    let snapshot_hash = evaluate_single_chunk_variant(
        &TestSigner::from_byte(21),
        None,
        None,
        &[(4, "0", "1")],
        None,
        Some([0x88; 32]),
        None,
        None,
        CheckpointHistoryVariant::Accepted,
    );
    assert_eq!(
        snapshot_hash.checkpoints()[0].status(),
        CheckpointVerificationStatus::SnapshotHashMismatch
    );
    validated_checkpoint_chunk_carrier_enters_corpus();
    signed_irregular_multichunk_checkpoint_reconstructs_exact_history();
}

#[test]
fn checkpoint_snapshot_refusals() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/v1_draft/checkpoints/negative_snapshot.json"
    ))
    .unwrap_or_default();
    assert_eq!(fixture["cases"].as_array().map(Vec::len), Some(6));
    let scenario = signed_engine_scenario();
    let invalid_bytes = vec![0xff; scenario.snapshot.len()];
    let invalid_hash: [u8; 32] = Sha256::digest(&invalid_bytes).into();
    let invalid_root = nostr_automerge::checkpoint::leaf_hash(0, 1, invalid_hash);
    let load = evaluate_single_chunk_variant(
        &TestSigner::from_byte(21),
        None,
        None,
        &[(4, "0", "1")],
        Some(&invalid_bytes),
        Some(invalid_hash),
        Some(invalid_root),
        None,
        CheckpointHistoryVariant::Accepted,
    );
    assert_eq!(
        load.checkpoints()[0].status(),
        CheckpointVerificationStatus::SnapshotLoad
    );

    let mut digest = Sha256::new();
    digest.update(b"nostr-crdt/automerge/change-set/v1");
    digest.update([0]);
    digest.update(1_u64.to_be_bytes());
    digest.update(scenario.change_hash.as_bytes());
    let base = CheckpointCommitmentVariant {
        head: scenario.change_hash,
        change_count: 1,
        change_set_hash: digest.finalize().into(),
        dependency_edges: 0,
        total_ops: 1,
    };
    let head = evaluate_single_chunk_variant(
        &TestSigner::from_byte(21),
        None,
        None,
        &[(4, "0", "1")],
        None,
        None,
        None,
        Some(CheckpointCommitmentVariant {
            head: ChangeHash::from_bytes([0x99; 32]),
            ..base
        }),
        CheckpointHistoryVariant::Accepted,
    );
    assert_eq!(
        head.checkpoints()[0].status(),
        CheckpointVerificationStatus::HeadMismatch
    );
    for commitments in [
        CheckpointCommitmentVariant {
            change_count: 2,
            ..base
        },
        CheckpointCommitmentVariant {
            change_set_hash: [0x77; 32],
            ..base
        },
        CheckpointCommitmentVariant {
            dependency_edges: 1,
            ..base
        },
        CheckpointCommitmentVariant {
            total_ops: 2,
            ..base
        },
    ] {
        let report = evaluate_single_chunk_variant(
            &TestSigner::from_byte(21),
            None,
            None,
            &[(4, "0", "1")],
            None,
            None,
            None,
            Some(commitments),
            CheckpointHistoryVariant::Accepted,
        );
        assert_eq!(
            report.checkpoints()[0].status(),
            CheckpointVerificationStatus::CommitmentMismatch
        );
    }
}

#[test]
fn checkpoint_history_refusals() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/v1_draft/checkpoints/negative_history.json"
    ))
    .unwrap_or_default();
    assert_eq!(fixture["cases"].as_array().map(Vec::len), Some(2));
    let missing = evaluate_single_chunk_variant(
        &TestSigner::from_byte(21),
        None,
        None,
        &[(4, "0", "1")],
        None,
        None,
        None,
        None,
        CheckpointHistoryVariant::MissingCarrier,
    );
    assert_eq!(
        missing.checkpoints()[0].status(),
        CheckpointVerificationStatus::MissingHistoricalCarrier
    );
    let not_accepted = evaluate_single_chunk_variant(
        &TestSigner::from_byte(21),
        None,
        None,
        &[(4, "0", "1")],
        None,
        None,
        None,
        None,
        CheckpointHistoryVariant::NotAccepted,
    );
    assert_eq!(
        not_accepted.checkpoints()[0].status(),
        CheckpointVerificationStatus::NotAcceptedAtControl
    );
}

#[test]
fn checkpoint_interruption_is_non_authoritative() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/v1_draft/checkpoints/interruption.json"
    ))
    .unwrap_or_default();
    assert_eq!(fixture["cases"].as_array().map(Vec::len), Some(2));
    let run = |control| {
        evaluate_single_chunk_variant_controlled(
            &TestSigner::from_byte(21),
            None,
            None,
            &[(4, "0", "1")],
            None,
            None,
            None,
            None,
            CheckpointHistoryVariant::Accepted,
            control,
        )
    };
    let (baseline, counters, checks) = run(CheckpointEvaluationControl::Normal);
    assert!(counters.get(WorkCounter::CheckpointItem) > 0);
    let budgeted = (0..=1_000).find_map(|limit| {
        let (report, _, _) = run(CheckpointEvaluationControl::ItemBudget(limit));
        (report.completion() == Completion::BudgetExhausted
            && report.accepted_changes() == baseline.accepted_changes()
            && report.heads() == baseline.heads()
            && report.history_digest() == baseline.history_digest()
            && report.checkpoints().iter().all(|checkpoint| {
                checkpoint.status() == CheckpointVerificationStatus::BudgetExhausted
            }))
        .then_some(report)
    });
    assert!(budgeted.is_some());
    let Some(budgeted) = budgeted else { return };
    assert_eq!(budgeted.completion(), Completion::BudgetExhausted);
    assert_eq!(budgeted.accepted_changes(), baseline.accepted_changes());
    assert_eq!(budgeted.heads(), baseline.heads());
    assert_eq!(budgeted.history_digest(), baseline.history_digest());
    assert!(budgeted.checkpoints().iter().all(|checkpoint| {
        checkpoint.status() == CheckpointVerificationStatus::BudgetExhausted
    }));

    let cancelled = (0..checks).find_map(|limit| {
        let (report, _, _) = run(CheckpointEvaluationControl::CancelAfter(limit));
        (report.accepted_changes() == baseline.accepted_changes()
            && report.heads() == baseline.heads()
            && report.history_digest() == baseline.history_digest()
            && report
                .checkpoints()
                .iter()
                .all(|checkpoint| checkpoint.status() == CheckpointVerificationStatus::Cancelled))
        .then_some(report)
    });
    assert!(cancelled.is_some());
    assert_eq!(
        cancelled.map(|report| report.completion()),
        Some(Completion::Cancelled)
    );
}

#[test]
#[allow(clippy::expect_used)]
fn adversarial_many_checkpoints_stop_without_refusal_expansion() {
    let scenario = signed_engine_scenario();
    let unauthorized = TestSigner::from_byte(119);
    let content = format!(
        r#"{{"change_count":1,"change_set_hash":"{}","chunk_count":1,"chunk_root":"{}","chunk_size":1,"dependency_edges":0,"encoding":"automerge-save-v1","heads":["{}"],"raw_size":1,"total_ops":1,"v":1}}"#,
        "01".repeat(32),
        "02".repeat(32),
        scenario.change_hash.to_hex(),
    );
    let mut builder = CorpusBuilder::new();
    for event in [scenario.control, scenario.change] {
        assert!(matches!(
            builder.ingest(event),
            IngestOutcome::Accepted { .. }
        ));
    }
    for offset in 0_u64..64 {
        let descriptor = unauthorized.sign(
            &UnsignedEventDraft::new(
                100_u64.saturating_add(offset),
                1_626,
                vec![
                    vec!["a".to_owned(), scenario.coordinate.to_address()],
                    vec!["e".to_owned(), scenario.control_id.to_hex()],
                    vec!["x".to_owned(), format!("{offset:064x}")],
                ],
                content.clone(),
            )
            .expect("many-checkpoint draft")
            .prepare(unauthorized.public_key())
            .expect("many-checkpoint preimage"),
        );
        assert!(matches!(
            builder.ingest(descriptor),
            IngestOutcome::Accepted { .. }
        ));
    }
    let corpus = builder.finish();
    let evaluator = ReferenceEvaluator::new(ProtocolRevision::draft_v1());
    let evaluate = |items| {
        let mut budget = WorkBudget::new(10_000_000, items);
        evaluator.evaluate_report(&corpus, scenario.coordinate, &mut budget, &NeverCancelled)
    };
    let complete = evaluate(10_000_000);
    assert_eq!(complete.completion(), Completion::Complete);
    assert_eq!(complete.checkpoints().len(), 64);
    let mut low = 0_u64;
    let mut high = 10_000_000_u64;
    while low < high {
        let middle = low + (high - low) / 2;
        if evaluate(middle).checkpoints().is_empty() {
            low = middle.saturating_add(1);
        } else {
            high = middle;
        }
    }
    let interrupted = evaluate(low);
    assert_eq!(interrupted.completion(), Completion::BudgetExhausted);
    assert!(!interrupted.checkpoints().is_empty());
    assert!(interrupted.checkpoints().len() < complete.checkpoints().len());
    assert_eq!(interrupted.accepted_changes(), complete.accepted_changes());
    assert_eq!(interrupted.history_digest(), complete.history_digest());
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
    assert_checkpoint_event_dispositions(&report, ProtocolDisposition::Invalid);
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
fn checkpoint_unauthorized_signed_fixture() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/v1_draft/checkpoints/cases.json"
    ))
    .unwrap_or_default();
    assert_eq!(
        fixture["refusals"]["checkpoint.unauthorized.signed"]["status"].as_str(),
        Some("unauthorized")
    );
    checkpoints_never_authorize_or_redefine_history();
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

#[allow(clippy::expect_used)]
fn signed_manifest_hint(
    controller: &TestSigner,
    coordinate: DocumentCoordinate,
    created_at: u64,
    control: EventId,
) -> RawEventBytes {
    let content = format!(
        r#"{{"application":null,"checkpoint":null,"control":"{}","description":null,"format":"automerge-change-v1","name":null,"relays":[],"status":"active","successor":null,"text_encoding":"utf16","v":1}}"#,
        control.to_hex()
    );
    controller.sign(
        &UnsignedEventDraft::new(
            created_at,
            31_624,
            vec![vec!["d".to_owned(), coordinate.document_id().to_hex()]],
            content,
        )
        .expect("manifest draft")
        .prepare(controller.public_key())
        .expect("manifest preimage"),
    )
}

#[test]
#[allow(clippy::expect_used)]
fn selected_manifest_control_references_are_resolved_after_replacement() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/v1_draft/manifests/dynamic_control_references.json"
    ))
    .expect("dynamic manifest fixture contract");
    assert_eq!(fixture["cases"].as_array().map(Vec::len), Some(5));
    assert_eq!(fixture["orders"].as_array().map(Vec::len), Some(3));

    let controller = TestSigner::from_byte(122);
    let device = TestSigner::from_byte(123);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "c2".repeat(32)
    )
    .parse()
    .expect("manifest coordinate");
    let members = vec![(device.public_key().to_hex(), vec!["write"])];
    let left = signed_acl_control(&controller, coordinate, 1, None, 0, members.clone());
    let right = signed_acl_control(&controller, coordinate, 2, None, 0, members.clone());
    let left_id = VerifiedNip01Event::verify(left.clone())
        .expect("left control")
        .event_id();
    let right_id = VerifiedNip01Event::verify(right.clone())
        .expect("right control")
        .event_id();
    let canonical_id = left_id.min(right_id);
    let noncanonical_id = left_id.max(right_id);

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

    let baseline = evaluate(vec![left.clone(), right.clone()]);
    let canonical_manifest = signed_manifest_hint(&controller, coordinate, 3, canonical_id);
    let canonical_manifest_id = VerifiedNip01Event::verify(canonical_manifest.clone())
        .expect("canonical manifest")
        .event_id();
    let canonical = evaluate(vec![
        canonical_manifest.clone(),
        right.clone(),
        left.clone(),
    ]);
    assert!(matches!(
        canonical.manifest(),
        ResolvedManifestAvailability::Available {
            hints,
            control_status: ManifestControlStatus::Canonical,
        } if hints.control() == canonical_id
    ));
    assert!(ManifestControlStatus::Canonical < ManifestControlStatus::Noncanonical);
    let manifest_debug = format!("{:?}", canonical.manifest());
    assert!(manifest_debug.contains("Canonical"));
    assert!(!manifest_debug.contains(&canonical_id.to_hex()));
    assert_eq!(
        event_disposition(&canonical, canonical_manifest_id),
        Some(ProtocolDisposition::Accepted)
    );

    let noncanonical_manifest = signed_manifest_hint(&controller, coordinate, 3, noncanonical_id);
    let noncanonical_manifest_id = VerifiedNip01Event::verify(noncanonical_manifest.clone())
        .expect("noncanonical manifest")
        .event_id();
    let orders = [
        vec![left.clone(), right.clone(), noncanonical_manifest.clone()],
        vec![noncanonical_manifest.clone(), right.clone(), left.clone()],
        vec![right.clone(), left.clone(), noncanonical_manifest.clone()],
    ];
    let noncanonical_reports = orders.into_iter().map(evaluate).collect::<Vec<_>>();
    assert!(noncanonical_reports.iter().all(|report| matches!(
        report.manifest(),
        ResolvedManifestAvailability::Available {
            hints,
            control_status: ManifestControlStatus::Noncanonical,
        } if hints.control() == noncanonical_id
    )));
    assert!(
        noncanonical_reports
            .windows(2)
            .all(|pair| pair[0] == pair[1])
    );
    assert_eq!(
        event_disposition(&noncanonical_reports[0], noncanonical_manifest_id),
        Some(ProtocolDisposition::Accepted)
    );

    let missing_id = EventId::from_bytes([0xee; 32]);
    let older = signed_manifest_hint(&controller, coordinate, 3, canonical_id);
    let older_id = VerifiedNip01Event::verify(older.clone())
        .expect("older manifest")
        .event_id();
    let selected_missing = signed_manifest_hint(&controller, coordinate, 4, missing_id);
    let selected_missing_id = VerifiedNip01Event::verify(selected_missing.clone())
        .expect("missing-control manifest")
        .event_id();
    let missing = evaluate(vec![left.clone(), right.clone(), older, selected_missing]);
    assert!(matches!(
        missing.manifest(),
        ResolvedManifestAvailability::Pending {
            hints,
            reason: ManifestPendingReason::MissingControl,
        } if hints.event_id() == selected_missing_id && hints.control() == missing_id
    ));
    assert_eq!(
        event_disposition(&missing, selected_missing_id),
        Some(ProtocolDisposition::Pending)
    );
    assert_eq!(
        event_disposition(&missing, older_id),
        Some(ProtocolDisposition::Excluded)
    );

    let other_coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "c3".repeat(32)
    )
    .parse()
    .expect("other coordinate");
    let other_control =
        signed_acl_control(&controller, other_coordinate, 1, None, 0, members.clone());
    let other_control_id = VerifiedNip01Event::verify(other_control.clone())
        .expect("other control")
        .event_id();
    let wrong_manifest = signed_manifest_hint(&controller, coordinate, 3, other_control_id);
    let wrong_manifest_id = VerifiedNip01Event::verify(wrong_manifest.clone())
        .expect("wrong-coordinate manifest")
        .event_id();
    let wrong_coordinate = evaluate(vec![
        left.clone(),
        right.clone(),
        other_control,
        wrong_manifest,
    ]);
    assert!(matches!(
        wrong_coordinate.manifest(),
        ResolvedManifestAvailability::Unavailable { control: Some(control), diagnostic, .. }
            if *control == other_control_id && diagnostic.as_str() == "carrier.coordinate"
    ));
    assert_eq!(
        event_disposition(&wrong_coordinate, wrong_manifest_id),
        Some(ProtocolDisposition::Invalid)
    );

    let invalid_child =
        signed_acl_control(&controller, coordinate, 2, Some(canonical_id), 2, members);
    let invalid_child_id = VerifiedNip01Event::verify(invalid_child.clone())
        .expect("invalid child")
        .event_id();
    let invalid_manifest = signed_manifest_hint(&controller, coordinate, 3, invalid_child_id);
    let invalid_manifest_id = VerifiedNip01Event::verify(invalid_manifest.clone())
        .expect("invalid-control manifest")
        .event_id();
    let invalid = evaluate(vec![left, right, invalid_child, invalid_manifest]);
    assert!(matches!(
        invalid.manifest(),
        ResolvedManifestAvailability::Unavailable { control: Some(control), .. }
            if *control == invalid_child_id
    ));
    assert_eq!(
        event_disposition(&invalid, invalid_manifest_id),
        Some(ProtocolDisposition::Invalid)
    );

    for report in [
        &canonical,
        &noncanonical_reports[0],
        &missing,
        &wrong_coordinate,
        &invalid,
    ] {
        assert_eq!(report.canonical_controls(), baseline.canonical_controls());
        assert_eq!(report.accepted_changes(), baseline.accepted_changes());
        assert_eq!(report.heads(), baseline.heads());
        assert_eq!(report.history_digest(), baseline.history_digest());
        assert_eq!(report.document(), baseline.document());
    }
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
    assert_eq!(
        event_disposition(&report, event_id),
        Some(ProtocolDisposition::Pending)
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
    let mut invalid_arithmetic = CorpusBuilder::new();
    assert!(matches!(
        invalid_arithmetic.ingest(sign(
            6,
            valid_tags(),
            content.replace("\"raw_size\":1", "\"raw_size\":2")
        )),
        IngestOutcome::InvalidCarrier { diagnostic, .. }
            if diagnostic.as_str() == "checkpoint.descriptor"
    ));
}

#[test]
fn checkpoint_pending_control_signed_fixture() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/v1_draft/checkpoints/cases.json"
    ))
    .unwrap_or_default();
    assert_eq!(
        fixture["refusals"]["checkpoint.pending_control.signed"]["status"].as_str(),
        Some("pending_control")
    );
    validated_checkpoint_descriptor_carrier_enters_corpus();
}

#[test]
#[allow(clippy::expect_used)]
fn checkpoint_pending_requires_missing_or_statefully_pending_control() {
    let controller = TestSigner::from_byte(122);
    let author = TestSigner::from_byte(123);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "7a".repeat(32)
    )
    .parse()
    .expect("fixed coordinate");
    let pending_control = signed_acl_control(
        &controller,
        coordinate,
        1,
        Some(EventId::from_bytes([0x7b; 32])),
        1,
        vec![(author.public_key().to_hex(), vec!["checkpoint"])],
    );
    let pending_control_id = VerifiedNip01Event::verify(pending_control.clone())
        .expect("signed pending control")
        .event_id();
    let descriptor = author.sign(
        &UnsignedEventDraft::new(
            2,
            1_626,
            vec![
                vec!["a".to_owned(), coordinate.to_address()],
                vec!["e".to_owned(), pending_control_id.to_hex()],
                vec!["x".to_owned(), "7c".repeat(32)],
            ],
            format!(
                r#"{{"change_count":1,"change_set_hash":"{}","chunk_count":1,"chunk_root":"{}","chunk_size":1,"dependency_edges":0,"encoding":"automerge-save-v1","heads":["{}"],"raw_size":1,"total_ops":1,"v":1}}"#,
                "7d".repeat(32),
                "7e".repeat(32),
                "7f".repeat(32),
            ),
        )
        .expect("descriptor draft")
        .prepare(author.public_key())
        .expect("descriptor preimage"),
    );
    let descriptor_id = VerifiedNip01Event::verify(descriptor.clone())
        .expect("signed descriptor")
        .event_id();
    let mut builder = CorpusBuilder::new();
    for event in [descriptor, pending_control] {
        assert!(matches!(
            builder.ingest(event),
            IngestOutcome::Accepted { .. }
        ));
    }
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &builder.finish(),
        coordinate,
        &mut WorkBudget::new(1_000_000, 1_000_000),
        &NeverCancelled,
    );
    assert_eq!(
        report
            .control_dispositions()
            .iter()
            .find(|(event_id, _)| *event_id == pending_control_id)
            .map(|(_, disposition)| *disposition),
        Some(ProtocolDisposition::Pending)
    );
    assert_eq!(
        report.checkpoints().first().map(|result| result.status()),
        Some(CheckpointVerificationStatus::PendingControl)
    );
    assert_eq!(
        event_disposition(&report, descriptor_id),
        Some(ProtocolDisposition::Pending)
    );
    assert_eq!(event_diagnostic(&report, descriptor_id), None);
}

#[derive(Clone, Copy, Debug)]
enum KnownUnusableDescriptorControl {
    Noncanonical,
    WrongKind,
    WrongCoordinate,
    StaticInvalid,
    DynamicInvalid,
    Unsupported,
}

#[derive(Clone, Copy, Debug)]
enum ReferencedEvidenceIngress {
    Accepted,
    Irrelevant,
    InvalidCarrier,
    Unsupported,
}

#[allow(clippy::expect_used)]
fn signed_checkpoint_for_control(
    author: &TestSigner,
    coordinate: DocumentCoordinate,
    control_id: EventId,
) -> (RawEventBytes, EventId, RawEventBytes, EventId) {
    let data = b"a";
    let chunk_hash: [u8; 32] = Sha256::digest(data).into();
    let descriptor = author.sign(
        &UnsignedEventDraft::new(
            20,
            1_626,
            vec![
                vec!["a".to_owned(), coordinate.to_address()],
                vec!["e".to_owned(), control_id.to_hex()],
                vec!["x".to_owned(), "a1".repeat(32)],
            ],
            format!(
                r#"{{"change_count":1,"change_set_hash":"{}","chunk_count":1,"chunk_root":"{}","chunk_size":1,"dependency_edges":0,"encoding":"automerge-save-v1","heads":["{}"],"raw_size":1,"total_ops":1,"v":1}}"#,
                "b1".repeat(32),
                "c1".repeat(32),
                "d1".repeat(32),
            ),
        )
        .expect("descriptor draft")
        .prepare(author.public_key())
        .expect("descriptor preimage"),
    );
    let descriptor_id = VerifiedNip01Event::verify(descriptor.clone())
        .expect("signed descriptor")
        .event_id();
    let chunk = author.sign(
        &UnsignedEventDraft::new(
            21,
            1_627,
            vec![
                vec!["a".to_owned(), coordinate.to_address()],
                vec!["e".to_owned(), descriptor_id.to_hex()],
                vec!["x".to_owned(), hex32(chunk_hash)],
                vec!["part".to_owned(), "0".to_owned(), "1".to_owned()],
            ],
            r#"{"data":"YQ==","proof":[],"v":1}"#.to_owned(),
        )
        .expect("chunk draft")
        .prepare(author.public_key())
        .expect("chunk preimage"),
    );
    let chunk_id = VerifiedNip01Event::verify(chunk.clone())
        .expect("signed chunk")
        .event_id();
    (descriptor, descriptor_id, chunk, chunk_id)
}

#[allow(clippy::expect_used)]
fn evaluate_known_unusable_descriptor_control(
    family: KnownUnusableDescriptorControl,
) -> (EvaluationReport, EventId, EventId, WorkCounters) {
    let controller = TestSigner::from_byte(124);
    let author = TestSigner::from_byte(125);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "81".repeat(32)
    )
    .parse()
    .expect("fixed coordinate");
    let grants = vec![(author.public_key().to_hex(), vec!["checkpoint"])];
    let mut events = Vec::new();
    let (control, expected_ingest) = match family {
        KnownUnusableDescriptorControl::Noncanonical => {
            let left = signed_acl_control(&controller, coordinate, 1, None, 0, grants.clone());
            let right = signed_acl_control(&controller, coordinate, 2, None, 0, grants.clone());
            let left_id = VerifiedNip01Event::verify(left.clone())
                .expect("signed left control")
                .event_id();
            let right_id = VerifiedNip01Event::verify(right.clone())
                .expect("signed right control")
                .event_id();
            events.extend([left, right]);
            (left_id.max(right_id), ReferencedEvidenceIngress::Accepted)
        }
        KnownUnusableDescriptorControl::WrongKind => {
            let event = controller.sign(
                &UnsignedEventDraft::new(1, 1, Vec::new(), "known non-carrier".to_owned())
                    .expect("non-carrier draft")
                    .prepare(controller.public_key())
                    .expect("non-carrier preimage"),
            );
            let event_id = VerifiedNip01Event::verify(event.clone())
                .expect("signed non-carrier")
                .event_id();
            events.push(event);
            (event_id, ReferencedEvidenceIngress::Irrelevant)
        }
        KnownUnusableDescriptorControl::WrongCoordinate => {
            let other_coordinate: DocumentCoordinate = format!(
                "31624:{}:{}",
                controller.public_key().to_hex(),
                "82".repeat(32)
            )
            .parse()
            .expect("other coordinate");
            let event =
                signed_acl_control(&controller, other_coordinate, 1, None, 0, grants.clone());
            let event_id = VerifiedNip01Event::verify(event.clone())
                .expect("signed other-coordinate control")
                .event_id();
            events.push(event);
            (event_id, ReferencedEvidenceIngress::Accepted)
        }
        KnownUnusableDescriptorControl::StaticInvalid => {
            let event = controller.sign(
                &UnsignedEventDraft::new(
                    1,
                    1_625,
                    vec![vec!["a".to_owned(), coordinate.to_address()]],
                    r#"{"v":1}"#.to_owned(),
                )
                .expect("invalid control draft")
                .prepare(controller.public_key())
                .expect("invalid control preimage"),
            );
            let event_id = VerifiedNip01Event::verify(event.clone())
                .expect("signed invalid control")
                .event_id();
            events.push(event);
            (event_id, ReferencedEvidenceIngress::InvalidCarrier)
        }
        KnownUnusableDescriptorControl::DynamicInvalid => {
            let genesis = signed_acl_control(&controller, coordinate, 1, None, 0, grants.clone());
            let genesis_id = VerifiedNip01Event::verify(genesis.clone())
                .expect("signed genesis")
                .event_id();
            let child = signed_acl_control(&controller, coordinate, 2, Some(genesis_id), 2, grants);
            let child_id = VerifiedNip01Event::verify(child.clone())
                .expect("signed invalid child")
                .event_id();
            events.extend([genesis, child]);
            (child_id, ReferencedEvidenceIngress::Accepted)
        }
        KnownUnusableDescriptorControl::Unsupported => {
            let event = controller.sign(
                &UnsignedEventDraft::new(
                    1,
                    1_625,
                    vec![vec!["a".to_owned(), coordinate.to_address()]],
                    r#"{"v":2}"#.to_owned(),
                )
                .expect("unsupported control draft")
                .prepare(controller.public_key())
                .expect("unsupported control preimage"),
            );
            let event_id = VerifiedNip01Event::verify(event.clone())
                .expect("signed unsupported control")
                .event_id();
            events.push(event);
            (event_id, ReferencedEvidenceIngress::Unsupported)
        }
    };
    let (descriptor, descriptor_id, chunk, chunk_id) =
        signed_checkpoint_for_control(&author, coordinate, control);
    events.extend([descriptor, chunk]);

    let mut builder = CorpusBuilder::new();
    for event in events {
        let event_id = VerifiedNip01Event::verify(event.clone())
            .expect("signed family event")
            .event_id();
        let outcome = builder.ingest(event);
        if event_id == control {
            match expected_ingest {
                ReferencedEvidenceIngress::Accepted => {
                    assert!(matches!(outcome, IngestOutcome::Accepted { .. }))
                }
                ReferencedEvidenceIngress::Irrelevant => {
                    assert!(matches!(outcome, IngestOutcome::Irrelevant { .. }))
                }
                ReferencedEvidenceIngress::InvalidCarrier => {
                    assert!(matches!(outcome, IngestOutcome::InvalidCarrier { .. }))
                }
                ReferencedEvidenceIngress::Unsupported => {
                    assert!(matches!(outcome, IngestOutcome::UnsupportedRevision { .. }))
                }
            }
        } else {
            assert!(matches!(outcome, IngestOutcome::Accepted { .. }));
        }
    }
    let mut budget = WorkBudget::new(1_000_000, 1_000_000);
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &builder.finish(),
        coordinate,
        &mut budget,
        &NeverCancelled,
    );
    (report, descriptor_id, chunk_id, budget.consumed())
}

#[test]
#[allow(clippy::expect_used)]
fn six_known_unusable_descriptor_controls_are_invalid_before_history_work() {
    for family in [
        KnownUnusableDescriptorControl::Noncanonical,
        KnownUnusableDescriptorControl::WrongKind,
        KnownUnusableDescriptorControl::WrongCoordinate,
        KnownUnusableDescriptorControl::StaticInvalid,
        KnownUnusableDescriptorControl::DynamicInvalid,
        KnownUnusableDescriptorControl::Unsupported,
    ] {
        let (report, descriptor_id, chunk_id, work) =
            evaluate_known_unusable_descriptor_control(family);
        let checkpoint = report
            .checkpoints()
            .iter()
            .find(|checkpoint| checkpoint.descriptor_event() == descriptor_id)
            .expect("known-unusable checkpoint result");
        assert_eq!(report.completion(), Completion::Complete, "{family:?}");
        assert_eq!(
            checkpoint.status(),
            CheckpointVerificationStatus::Unauthorized,
            "{family:?}"
        );
        assert_eq!(checkpoint.chunk_events(), [chunk_id], "{family:?}");
        for event_id in [descriptor_id, chunk_id] {
            assert_eq!(
                event_disposition(&report, event_id),
                Some(ProtocolDisposition::Invalid),
                "{family:?}"
            );
            assert_eq!(
                event_diagnostic(&report, event_id),
                Some("checkpoint.history"),
                "{family:?}"
            );
        }
        assert_eq!(checkpoint.completion(), Completion::Complete, "{family:?}");
        assert!(
            work.get(WorkCounter::CheckpointByte) == 0,
            "{family:?} performed snapshot byte work"
        );
    }
}

#[test]
fn known_invalid_checkpoint_control_is_never_pending() {
    let report = evaluate_signed_reproduction_fixture(include_str!(
        "../../../fixtures/v1_draft/scenarios/checkpoint/checkpoint_descriptor_references_invalid_control.input.json"
    ));
    assert_eq!(report.checkpoints().len(), 1);
    let Some(checkpoint) = report.checkpoints().first() else {
        return;
    };
    assert_eq!(
        checkpoint.status(),
        CheckpointVerificationStatus::Unauthorized
    );
    assert_eq!(
        event_disposition(&report, checkpoint.descriptor_event()),
        Some(ProtocolDisposition::Invalid)
    );
    assert_eq!(
        event_diagnostic(&report, checkpoint.descriptor_event()),
        Some("checkpoint.history")
    );
    assert_eq!(checkpoint.chunk_events().len(), 1);
    for chunk_id in checkpoint.chunk_events() {
        assert_eq!(
            event_disposition(&report, *chunk_id),
            Some(ProtocolDisposition::Invalid)
        );
        assert_eq!(
            event_diagnostic(&report, *chunk_id),
            Some("checkpoint.history")
        );
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
    let mut out_of_range = valid_tags();
    out_of_range[3][1] = "1".to_owned();
    for (created_at, tags, content, expected) in [
        (
            2,
            [valid_tags(), vec![vec!["-".to_owned()]]].concat(),
            canonical_content,
            "tag.forbidden",
        ),
        (3, leading_zero, canonical_content, "checkpoint.chunk"),
        (4, upper_hash, canonical_content, "checkpoint.chunk"),
        (5, out_of_range, canonical_content, "checkpoint.chunk"),
        (
            6,
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
    let oversized = vec![0_u8; 32_769];
    let oversized_hash: [u8; 32] = Sha256::digest(&oversized).into();
    let oversized_tags = vec![
        vec!["a".to_owned(), coordinate.to_address()],
        vec!["e".to_owned(), "04".repeat(32)],
        vec![
            "x".to_owned(),
            ChunkHash::from_bytes(oversized_hash).to_hex(),
        ],
        vec!["part".to_owned(), "0".to_owned(), "1".to_owned()],
    ];
    let oversized_content = format!(
        r#"{{"data":"{}","proof":[],"v":1}}"#,
        base64::engine::general_purpose::STANDARD.encode(oversized)
    );
    let mut invalid = CorpusBuilder::new();
    assert!(matches!(
        invalid.ingest(sign(7, oversized_tags, &oversized_content)),
        IngestOutcome::InvalidCarrier { diagnostic, .. }
            if diagnostic.as_str() == "checkpoint.chunk"
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
    let pending = pending.finish();
    assert_eq!(
        pending.pending_control_ids().collect::<Vec<_>>(),
        vec![child_id]
    );
    let parsed_coordinate: DocumentCoordinate = coordinate.parse().expect("fixed coordinate");
    let pending_report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &pending,
        parsed_coordinate,
        &mut WorkBudget::new(1_000_000, 1_000),
        &NeverCancelled,
    );
    assert_eq!(
        pending_report.control_dispositions(),
        [(child_id, ProtocolDisposition::Pending)]
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

#[allow(clippy::expect_used)]
fn evaluate_signed_reproduction_fixture(source: &str) -> EvaluationReport {
    let fixture: serde_json::Value =
        serde_json::from_str(source).expect("signed reproduction fixture");
    let coordinate = fixture["coordinate"]
        .as_str()
        .expect("fixture coordinate")
        .parse()
        .expect("valid fixture coordinate");
    let mut builder = CorpusBuilder::new();
    for entry in fixture["raw_events"]
        .as_array()
        .expect("fixture raw events")
    {
        let data = entry["data"].as_str().expect("signed event bytes");
        let outcome = builder.ingest_bytes(data.as_bytes());
        assert!(
            matches!(
                outcome,
                IngestOutcome::Accepted { .. }
                    | IngestOutcome::InvalidCarrier { .. }
                    | IngestOutcome::UnsupportedRevision { .. }
            ),
            "fixture event must enter the evidence corpus"
        );
    }
    ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &builder.finish(),
        coordinate,
        &mut WorkBudget::new(2_000_000, 2_000_000),
        &NeverCancelled,
    )
}

#[test]
fn finding_073_checkpoint_authorization_precedes_history() {
    let report = evaluate_signed_reproduction_fixture(include_str!(
        "../../../fixtures/v1_draft/scenarios/checkpoint/checkpoint_descriptor_references_invalid_control.input.json"
    ));
    assert_eq!(
        report.checkpoints().first().map(|result| result.status()),
        Some(CheckpointVerificationStatus::Unauthorized),
        "FINDING_073 regression: known-invalid checkpoint control must be rejected before history work"
    );
}

#[test]
#[ignore = "expected to fail until FINDING_074 closes"]
#[allow(clippy::expect_used)]
fn finding_074_invalid_carrier_is_independent_of_excluded_hash() {
    let scenario = signed_engine_scenario();
    let controller = TestSigner::from_byte(20);
    let writer = TestSigner::from_byte(21);
    let members = vec![(controller.public_key().to_hex(), vec!["write"])];
    let canonical_child = signed_acl_control(
        &controller,
        scenario.coordinate,
        3,
        Some(scenario.control_id),
        1,
        members.clone(),
    );
    let invalid_child = signed_acl_control(
        &controller,
        scenario.coordinate,
        4,
        Some(scenario.control_id),
        2,
        members,
    );
    let invalid_child_id = VerifiedNip01Event::verify(invalid_child.clone())
        .expect("signed invalid child")
        .event_id();
    let original =
        VerifiedNip01Event::verify(scenario.change.clone()).expect("signed original change");
    let invalid_claim = writer.sign(
        &UnsignedEventDraft::new(
            5,
            1_624,
            vec![
                vec!["a".to_owned(), scenario.coordinate.to_address()],
                vec!["e".to_owned(), invalid_child_id.to_hex()],
                vec!["x".to_owned(), scenario.change_hash.to_hex()],
            ],
            original.content().to_owned(),
        )
        .expect("duplicate claim draft")
        .prepare(writer.public_key())
        .expect("duplicate claim preimage"),
    );
    let invalid_claim_id = VerifiedNip01Event::verify(invalid_claim.clone())
        .expect("signed invalid claim")
        .event_id();
    let mut builder = CorpusBuilder::new();
    for event in [
        scenario.control,
        scenario.change,
        canonical_child,
        invalid_child,
        invalid_claim,
    ] {
        assert!(matches!(
            builder.ingest(event),
            IngestOutcome::Accepted { .. }
        ));
    }
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &builder.finish(),
        scenario.coordinate,
        &mut WorkBudget::new(2_000_000, 2_000_000),
        &NeverCancelled,
    );
    assert_eq!(report.excluded_changes(), [scenario.change_hash]);
    assert_eq!(
        event_disposition(&report, invalid_claim_id),
        Some(ProtocolDisposition::Invalid),
        "FINDING_074 reproduced: known-invalid carrier inherits the excluded semantic-hash outcome"
    );
}

#[test]
#[allow(clippy::expect_used)]
fn finding_083_budget_stop_is_not_relabelled_by_cancellation_requery() {
    let controller = TestSigner::from_byte(120);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "c0".repeat(32)
    )
    .parse()
    .expect("fixed coordinate");
    let observations = Cell::new(0_u64);
    let cancellation = || {
        let observation = observations.get().saturating_add(1);
        observations.set(observation);
        observation > 1
    };
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1()).evaluate_report(
        &CorpusBuilder::new().finish(),
        coordinate,
        &mut WorkBudget::new(0, 0),
        &cancellation,
    );
    assert_eq!(
        (report.completion(), observations.get()),
        (Completion::BudgetExhausted, 1),
        "FINDING_083 regression: budget exhaustion must not be relabelled by a repeated cancellation observation"
    );
}

#[test]
#[ignore = "expected to fail until FINDING_082 closes"]
#[allow(clippy::expect_used)]
fn finding_082_reevaluation_stops_before_post_incomplete_alert_work() {
    let controller = TestSigner::from_byte(121);
    let writer = TestSigner::from_byte(122);
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key().to_hex(),
        "c1".repeat(32)
    )
    .parse()
    .expect("fixed coordinate");
    let members = vec![(writer.public_key().to_hex(), vec!["write"])];
    let old_control = signed_acl_control(&controller, coordinate, 1, None, 0, members.clone());
    let new_control = signed_acl_control(&controller, coordinate, 2, None, 0, members);
    let old_corpus = {
        let mut builder = CorpusBuilder::new();
        assert!(matches!(
            builder.ingest(old_control),
            IngestOutcome::Accepted { .. }
        ));
        builder.finish()
    };
    let new_corpus = {
        let mut builder = CorpusBuilder::new();
        assert!(matches!(
            builder.ingest(new_control),
            IngestOutcome::Accepted { .. }
        ));
        builder.finish()
    };
    let evaluator = ReferenceEvaluator::new(ProtocolRevision::draft_v1());
    let previous = evaluator.evaluate_report(
        &old_corpus,
        coordinate,
        &mut WorkBudget::new(2_000_000, 2_000_000),
        &NeverCancelled,
    );
    assert_eq!(previous.completion(), Completion::Complete);
    let mut calibration = WorkBudget::new(2_000_000, 2_000);
    let complete_current =
        evaluator.evaluate_report(&new_corpus, coordinate, &mut calibration, &NeverCancelled);
    assert_eq!(complete_current.completion(), Completion::Complete);
    let consumed_items = 2_000_u64.saturating_sub(calibration.remaining().1);
    let current = evaluator.reevaluate_report(
        &new_corpus,
        coordinate,
        &previous,
        &mut WorkBudget::new(2_000_000, consumed_items.saturating_sub(1)),
        &NeverCancelled,
    );
    assert_eq!(current.completion(), Completion::BudgetExhausted);
    assert_eq!(
        current.integrity_alerts().len(),
        0,
        "FINDING_082 reproduced: reevaluation adds an integrity alert after incomplete finalization"
    );
}
