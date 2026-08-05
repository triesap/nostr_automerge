use crate::carrier::VerifiedCarrier;
use crate::conformance::dispositions_digest::{
    DispositionItem, DispositionNamespace, dispositions_digest,
};
use crate::conformance::history_digest::history_digest;
use crate::evidence::event::EventEvidence;
use crate::graph::change_candidate::{CandidateCarrier, ChangeCandidate};
use crate::reference::evaluate::{BatchChange, BatchControl, evaluate_batch};
use crate::types::role::Role;
use crate::{
    CancellationCheck, ChangeHash, Completion, DocumentCoordinate, EvidenceCorpus,
    ProtocolDisposition, ProtocolRevision, WorkBudget, WorkCounter,
};

use super::evaluation_report::{
    EvaluationFailure, EvaluationReport, EvaluationReportParts, MaterializedDocumentView,
};

/// Stateless deterministic batch evaluator for immutable signed evidence.
///
/// Evaluation performs no networking, storage, clock access, signing, or key
/// custody. Callers supply an immutable evidence corpus plus explicit local
/// work and cancellation policy to the evaluation operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReferenceEvaluator {
    revision: ProtocolRevision,
}

impl ReferenceEvaluator {
    /// Creates an evaluator for the sealed protocol revision.
    #[must_use]
    pub const fn new(revision: ProtocolRevision) -> Self {
        Self { revision }
    }

    /// Returns the sealed revision interpreted by this evaluator.
    #[must_use]
    pub const fn revision(&self) -> ProtocolRevision {
        self.revision
    }

    /// Evaluates one document coordinate from fully retained signed evidence.
    ///
    /// The operation performs no I/O and derives every control and change input
    /// from validated carriers in `corpus`. Local exhaustion or cancellation is
    /// represented by [`crate::Completion`] without changing dispositions.
    #[must_use]
    pub fn evaluate(
        &self,
        corpus: &EvidenceCorpus,
        coordinate: DocumentCoordinate,
        budget: &mut WorkBudget,
        cancellation: &impl CancellationCheck,
    ) -> EvaluationReport {
        let ingress_complete = charge_ingress(corpus, budget);
        let controls = if ingress_complete {
            controls_for_coordinate(corpus, coordinate)
        } else {
            Vec::new()
        };
        let mut batch = evaluate_batch(controls, budget, cancellation);
        if !ingress_complete {
            batch.completion = Completion::BudgetExhausted;
            batch.failure = Some(EvaluationFailure::BudgetExhausted);
            batch.materialized_document = None;
        }
        let canonical_controls = batch.canonical_controls;
        let dispositions = batch.dispositions.into_iter().collect::<Vec<_>>();
        let accepted_changes = batch.accepted_changes.into_iter().collect::<Vec<_>>();
        let heads = batch.heads.into_iter().collect::<Vec<_>>();
        let pending_changes = disposition_hashes(&dispositions, ProtocolDisposition::Pending);
        let excluded_changes = dispositions
            .iter()
            .filter_map(|(hash, disposition)| {
                (*disposition != ProtocolDisposition::Accepted
                    && *disposition != ProtocolDisposition::Pending)
                    .then_some(*hash)
            })
            .collect::<Vec<_>>();
        let history_digest = history_digest(
            self.revision,
            coordinate,
            &canonical_controls,
            &accepted_changes,
            &heads,
        )
        .unwrap_or_else(|_| unreachable!("engine report collections are canonical"));
        let disposition_items = dispositions
            .iter()
            .map(|(hash, disposition)| DispositionItem {
                namespace: DispositionNamespace::ChangeHash,
                identifier: *hash.as_bytes(),
                disposition: *disposition,
            })
            .collect::<Vec<_>>();
        let dispositions_digest =
            dispositions_digest(self.revision, coordinate, &disposition_items)
                .unwrap_or_else(|_| unreachable!("engine dispositions are canonical"));
        EvaluationReport::from_canonical_parts(EvaluationReportParts {
            coordinate,
            canonical_controls,
            dispositions,
            accepted_changes,
            pending_changes,
            excluded_changes,
            heads,
            evidence: corpus.records().collect(),
            history_digest,
            dispositions_digest,
            integrity_alerts: batch.integrity_alerts,
            completion: batch.completion,
            failure: batch.failure,
            document: batch
                .materialized_document
                .map(MaterializedDocumentView::from_canonical_bytes),
        })
    }
}

fn charge_ingress(corpus: &EvidenceCorpus, budget: &mut WorkBudget) -> bool {
    let event_count = u64::try_from(corpus.evaluation_event_count()).unwrap_or(u64::MAX);
    let carrier_count = u64::try_from(corpus.carrier_evidence_count()).unwrap_or(u64::MAX);
    let decode_bytes = corpus.decode_work_bytes().unwrap_or(u64::MAX);
    budget.charge(WorkCounter::Event, event_count).is_ok()
        && budget.charge(WorkCounter::Carrier, carrier_count).is_ok()
        && budget.charge(WorkCounter::DecodeByte, decode_bytes).is_ok()
}

fn disposition_hashes(
    dispositions: &[(ChangeHash, ProtocolDisposition)],
    expected: ProtocolDisposition,
) -> Vec<ChangeHash> {
    dispositions
        .iter()
        .filter_map(|(hash, disposition)| (*disposition == expected).then_some(*hash))
        .collect()
}

fn controls_for_coordinate(
    corpus: &EvidenceCorpus,
    coordinate: DocumentCoordinate,
) -> Vec<BatchControl> {
    corpus
        .events
        .values()
        .filter_map(|evidence| match evidence {
            EventEvidence::VerifiedCarrier {
                carrier: VerifiedCarrier::Control(control),
                ..
            } if control.coordinate() == coordinate
                && !corpus
                    .indexes
                    .controls
                    .pending
                    .contains(&control.event_id())
                && !has_terminal_parent(corpus, control) =>
            {
                Some(BatchControl {
                    event_id: control.event_id(),
                    parent: control.parent(),
                    accepted_base: control.base_heads().collect(),
                    frozen: control.terminal(),
                    changes: changes_for_control(corpus, control),
                })
            }
            _ => None,
        })
        .collect()
}

fn has_terminal_parent(
    corpus: &EvidenceCorpus,
    control: &crate::carrier::control::ValidatedControlCarrier,
) -> bool {
    control.parent().is_some_and(|parent_id| {
        matches!(
            corpus.events.get(&parent_id),
            Some(EventEvidence::VerifiedCarrier {
                carrier: VerifiedCarrier::Control(parent),
                ..
            }) if parent.coordinate() == control.coordinate() && parent.terminal()
        )
    })
}

fn changes_for_control(
    corpus: &EvidenceCorpus,
    control: &crate::carrier::control::ValidatedControlCarrier,
) -> Vec<BatchChange> {
    let hashes = corpus
        .indexes
        .changes
        .hashes_by_control
        .get(&control.event_id())
        .cloned()
        .unwrap_or_default();
    hashes
        .into_iter()
        .filter_map(|hash| change_for_hash(corpus, control, hash))
        .collect()
}

fn change_for_hash(
    corpus: &EvidenceCorpus,
    control: &crate::carrier::control::ValidatedControlCarrier,
    hash: ChangeHash,
) -> Option<BatchChange> {
    let event_ids = corpus.indexes.changes.carriers_by_hash.get(&hash)?;
    let mut raw = None;
    let carriers = event_ids
        .iter()
        .filter_map(|event_id| match corpus.events.get(event_id) {
            Some(EventEvidence::VerifiedCarrier {
                carrier: VerifiedCarrier::Change(change),
                ..
            }) if change.control_id() == control.event_id()
                && change.coordinate() == control.coordinate() =>
            {
                raw.get_or_insert_with(|| change.canonical_raw_bytes().to_vec());
                Some(CandidateCarrier {
                    event_id: change.event_id(),
                    change_hash: change.change_hash(),
                    actor: change.actor(),
                    sequence: change.sequence(),
                    start_op: change.start_op(),
                    operation_count: change.operation_count(),
                    dependencies: change.dependencies().collect(),
                    control_id: change.control_id(),
                    author: change.author_device(),
                })
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    let candidate = ChangeCandidate::from_carriers(carriers).ok()?;
    let authorized = control.members().iter().any(|member| {
        member.actor == candidate.actor
            && member.device == candidate.author
            && member.roles.contains(&Role::Write)
    });
    Some(BatchChange {
        candidate,
        semantically_valid: authorized && !control.terminal(),
        raw_change: raw,
    })
}
