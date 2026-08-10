use std::collections::{BTreeMap, BTreeSet};

use crate::automerge_adapter::document::{AppliedDocument, materialize_history};
use crate::automerge_adapter::materialized_view::MaterializedDocumentView;
use crate::control::candidate::{CandidateResult, evaluate_child};
use crate::control::candidate_outcome::ControlCandidateOutcome;
use crate::control::epoch_state::AcceptedEpochState;
use crate::control::frontier::accepted_frontier_closure;
use crate::control::parent_view::ParentEpochView;
use crate::control::select::select_valid_outcomes_with_alert;
use crate::control::validate::ControlEnvelope;
use crate::graph::change_candidate::ChangeCandidate;
use crate::graph::dependency_graph::build_graph;
use crate::graph::equivocation::{QuarantineError, quarantine_equivocation_descendants};
use crate::graph::schedule::{ScheduleError, schedule_candidates};
use crate::reference::epoch::{EpochCandidate, resolve_epoch};
use crate::reference::epoch_engine::{
    AcceptedAtControl, EpochEvaluationError, EpochEvaluationInput, EpochEvaluationResult,
    evaluate_epoch,
};
use crate::{
    CancellationCheck, ChangeHash, Completion, EvaluationFailure, EventId, IntegrityAlert,
    ProtocolDisposition, WorkBudget, WorkCounter,
};

#[derive(Clone, Debug)]
pub(crate) struct BatchChange {
    pub(crate) candidate: ChangeCandidate,
    pub(crate) semantically_valid: bool,
    pub(crate) raw_change: Option<Vec<u8>>,
}

#[derive(Clone, Debug)]
pub(crate) struct BatchControl {
    pub(crate) event_id: EventId,
    pub(crate) parent: Option<EventId>,
    pub(crate) accepted_base: BTreeSet<ChangeHash>,
    pub(crate) frozen: bool,
    pub(crate) changes: Vec<BatchChange>,
    pub(crate) envelope: Option<ControlEnvelope>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BatchEvaluationReport {
    pub(crate) canonical_controls: Vec<EventId>,
    pub(crate) control_dispositions: BTreeMap<EventId, ProtocolDisposition>,
    pub(crate) accepted_at_control: BTreeMap<EventId, AcceptedAtControl>,
    pub(crate) dispositions: BTreeMap<ChangeHash, ProtocolDisposition>,
    pub(crate) accepted_changes: BTreeSet<ChangeHash>,
    pub(crate) heads: BTreeSet<ChangeHash>,
    pub(crate) materialized_document: Option<Vec<u8>>,
    pub(crate) integrity_alerts: Vec<IntegrityAlert>,
    pub(crate) completion: Completion,
    pub(crate) failure: Option<EvaluationFailure>,
}

pub(crate) fn evaluate_batch(
    controls: impl IntoIterator<Item = BatchControl>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> BatchEvaluationReport {
    if cancellation.is_cancelled() {
        return incomplete_report(
            Vec::new(),
            BTreeMap::new(),
            BTreeSet::new(),
            Vec::new(),
            Completion::Cancelled,
        );
    }
    let mut collected = Vec::new();
    for control in controls {
        if cancellation.is_cancelled() {
            return incomplete_report(
                Vec::new(),
                BTreeMap::new(),
                BTreeSet::new(),
                Vec::new(),
                Completion::Cancelled,
            );
        }
        if budget.charge(WorkCounter::Control, 1).is_err() {
            return incomplete_report(
                Vec::new(),
                BTreeMap::new(),
                BTreeSet::new(),
                Vec::new(),
                Completion::BudgetExhausted,
            );
        }
        collected.push(control);
    }
    let controls = collected
        .into_iter()
        .map(|control| (control.event_id, control))
        .collect::<BTreeMap<_, _>>();
    let mut by_parent = BTreeMap::<Option<EventId>, BTreeSet<EventId>>::new();
    for control in controls.values() {
        by_parent
            .entry(control.parent)
            .or_default()
            .insert(control.event_id);
    }
    let mut control_dispositions = controls
        .keys()
        .copied()
        .map(|event_id| (event_id, ProtocolDisposition::Excluded))
        .collect::<BTreeMap<_, _>>();
    let mut dispositions = controls
        .values()
        .flat_map(|control| control.changes.iter())
        .map(|change| (change.candidate.change_hash, ProtocolDisposition::Excluded))
        .collect::<BTreeMap<_, _>>();
    let mut canonical_controls = Vec::new();
    let mut integrity_alerts = Vec::new();
    let mut completion = Completion::Complete;
    let mut failure = None;
    let mut accepted_changes = BTreeSet::new();
    let mut accepted_at_control = BTreeMap::new();
    let mut parent_epoch_result: Option<EpochEvaluationResult> = None;
    let mut parent_id = None;
    while let Some(children) = by_parent.get(&parent_id) {
        if cancellation.is_cancelled() {
            completion = Completion::Cancelled;
            break;
        }
        let candidate_count = u64::try_from(children.len()).unwrap_or(u64::MAX);
        if budget
            .charge(WorkCounter::Control, candidate_count)
            .is_err()
        {
            completion = Completion::BudgetExhausted;
            break;
        }
        let parent_view = parent_epoch_result
            .as_ref()
            .map(|result| ParentEpochView::from_accepted_state(result.accepted_state()));
        let ancestry = canonical_controls
            .iter()
            .filter_map(|event_id| {
                controls
                    .get(event_id)
                    .and_then(|control| control.envelope.as_ref())
                    .map(ControlEnvelope::content)
            })
            .collect::<Vec<_>>();
        let parent_envelope = parent_id.and_then(|event_id| {
            controls
                .get(&event_id)
                .and_then(|control| control.envelope.as_ref())
        });
        let outcomes = children
            .iter()
            .filter_map(|event_id| {
                let control = controls.get(event_id)?;
                let sequence = control
                    .envelope
                    .as_ref()
                    .map_or(0, ControlEnvelope::sequence);
                let Some(child) = control.envelope.as_ref() else {
                    return Some(ControlCandidateOutcome::valid(
                        *event_id,
                        control.parent,
                        sequence,
                        control.accepted_base.clone(),
                    ));
                };
                let Some(parent) = parent_envelope else {
                    return Some(ControlCandidateOutcome::valid(
                        *event_id,
                        None,
                        sequence,
                        BTreeSet::new(),
                    ));
                };
                let Some(view) = parent_view.as_ref() else {
                    return Some(ControlCandidateOutcome::invalid(
                        *event_id,
                        control.parent,
                        sequence,
                        crate::DiagnosticCode::registered("control.state"),
                        None,
                    ));
                };
                Some(match evaluate_child(parent, child, &ancestry, view) {
                    CandidateResult::Valid => {
                        let closure = accepted_frontier_closure(
                            child.base_heads(),
                            view.accepted(),
                            view.dependency_index(),
                        );
                        ControlCandidateOutcome::valid(
                            *event_id,
                            control.parent,
                            sequence,
                            closure.accepted,
                        )
                    }
                    CandidateResult::Pending(diagnostic) => ControlCandidateOutcome::pending(
                        *event_id,
                        control.parent,
                        sequence,
                        diagnostic,
                        None,
                    ),
                    CandidateResult::Invalid(diagnostic) => ControlCandidateOutcome::invalid(
                        *event_id,
                        control.parent,
                        sequence,
                        diagnostic,
                        None,
                    ),
                })
            })
            .collect::<Vec<_>>();
        for outcome in &outcomes {
            if outcome.disposition() != ProtocolDisposition::Accepted {
                control_dispositions.insert(outcome.event_id(), outcome.disposition());
            }
        }
        let (selection, alert) =
            select_valid_outcomes_with_alert(parent_id, outcomes.iter().cloned());
        let Some(selected) = selection.selected else {
            break;
        };
        let selected_base = outcomes
            .iter()
            .find(|outcome| outcome.event_id() == selected)
            .and_then(ControlCandidateOutcome::validated_base_closure)
            .cloned()
            .unwrap_or_default();
        let Some(control) = controls.get(&selected) else {
            failure = Some(EvaluationFailure::InvariantViolation);
            break;
        };
        let Some(selected_state) =
            accepted_state_for_closure(&selected_base, &controls, parent_epoch_result.as_ref())
        else {
            failure = Some(EvaluationFailure::InvariantViolation);
            break;
        };
        canonical_controls.push(selected);
        control_dispositions.insert(selected, ProtocolDisposition::Accepted);
        if let Some(alert) = alert {
            integrity_alerts.push(alert);
        }
        for change in &control.changes {
            dispositions.remove(&change.candidate.change_hash);
        }
        if budget.charge(WorkCounter::Control, 1).is_err() {
            completion = Completion::BudgetExhausted;
            break;
        }
        for hash in accepted_changes.difference(&selected_base) {
            dispositions.insert(*hash, ProtocolDisposition::Excluded);
        }
        accepted_changes = selected_base;
        let control_index = canonical_controls.len() - 1;
        let epoch = if control.envelope.is_some() {
            resolve_authoritative_epoch(
                control,
                selected_state,
                &canonical_controls[..control_index],
                &controls,
                budget,
                cancellation,
            )
        } else {
            let epoch_inputs = control.changes.iter().map(|change| EpochCandidate {
                candidate: change.candidate.clone(),
                semantically_valid: change.semantically_valid,
                canonical_control: !control.frozen,
            });
            resolve_epoch(epoch_inputs, accepted_changes.clone(), budget, cancellation)
                .map_err(EpochResolutionError::Schedule)
        };
        let resolved = match epoch {
            Ok(resolved) => resolved,
            Err(EpochResolutionError::Schedule(ScheduleError::BudgetExhausted)) => {
                completion = Completion::BudgetExhausted;
                break;
            }
            Err(EpochResolutionError::Schedule(ScheduleError::Cancelled)) => {
                completion = Completion::Cancelled;
                break;
            }
            Err(EpochResolutionError::InvalidState) => {
                failure = Some(EvaluationFailure::Graph);
                break;
            }
        };
        dispositions.extend(resolved);
        let mut eligible = controls
            .values()
            .flat_map(|candidate_control| candidate_control.changes.iter())
            .filter(|change| {
                accepted_changes.contains(&change.candidate.change_hash)
                    || (change.candidate.control_id == selected
                        && change.semantically_valid
                        && !control.frozen)
            })
            .map(|change| (change.candidate.change_hash, change.candidate.clone()))
            .collect::<BTreeMap<_, _>>();
        for change in &control.changes {
            if change.semantically_valid && !control.frozen {
                eligible
                    .entry(change.candidate.change_hash)
                    .or_insert_with(|| change.candidate.clone());
            }
        }
        let eligible = eligible.into_values().collect::<Vec<_>>();
        if let Ok(graph) = build_graph(eligible.clone(), accepted_changes.clone()) {
            match quarantine_equivocation_descendants(eligible, &graph, budget, cancellation) {
                Ok(quarantine) => {
                    for hash in &quarantine.quarantined {
                        dispositions.insert(*hash, ProtocolDisposition::Excluded);
                    }
                    integrity_alerts.extend(quarantine.alerts);
                }
                Err(QuarantineError::BudgetExhausted) => {
                    completion = Completion::BudgetExhausted;
                    break;
                }
                Err(QuarantineError::Cancelled) => {
                    completion = Completion::Cancelled;
                    break;
                }
                Err(QuarantineError::Alert(_)) => {
                    failure = Some(EvaluationFailure::InvariantViolation);
                    break;
                }
            }
        }
        accepted_changes.extend(dispositions.iter().filter_map(|(hash, disposition)| {
            (*disposition == ProtocolDisposition::Accepted).then_some(*hash)
        }));
        accepted_changes
            .retain(|hash| dispositions.get(hash) != Some(&ProtocolDisposition::Excluded));
        let Some(epoch_result) = epoch_result_from_accepted(
            &accepted_changes,
            &controls,
            dispositions.clone(),
            Vec::new(),
        ) else {
            failure = Some(EvaluationFailure::InvariantViolation);
            break;
        };
        accepted_at_control.insert(selected, AcceptedAtControl::from_result(&epoch_result));
        parent_epoch_result = Some(epoch_result);
        if control.frozen {
            break;
        }
        parent_id = Some(selected);
    }

    if completion != Completion::Complete || failure.is_some() {
        let heads = derive_heads(&accepted_changes, &controls);
        let report = incomplete_report(
            canonical_controls,
            dispositions,
            accepted_changes,
            integrity_alerts,
            completion,
        )
        .with_heads(heads);
        return match failure {
            Some(failure) => report.with_failure(failure),
            None => report,
        };
    }

    let candidates = controls
        .values()
        .flat_map(|control| control.changes.iter())
        .filter(|change| accepted_changes.contains(&change.candidate.change_hash))
        .map(|change| change.candidate.clone())
        .collect::<Vec<_>>();
    let ordered = match schedule_candidates(candidates, BTreeSet::new(), budget, cancellation) {
        Ok(schedule) => schedule.ordered,
        Err(error) => {
            let completion = match error {
                ScheduleError::BudgetExhausted => Completion::BudgetExhausted,
                ScheduleError::Cancelled => Completion::Cancelled,
            };
            let heads = derive_heads(&accepted_changes, &controls);
            return incomplete_report(
                canonical_controls,
                dispositions,
                accepted_changes,
                integrity_alerts,
                completion,
            )
            .with_heads(heads);
        }
    };
    let raw_changes = controls
        .values()
        .flat_map(|control| control.changes.iter())
        .filter_map(|change| {
            accepted_changes
                .contains(&change.candidate.change_hash)
                .then(|| {
                    change
                        .raw_change
                        .clone()
                        .map(|raw| (change.candidate.change_hash, raw))
                })
                .flatten()
        })
        .collect::<BTreeMap<_, _>>();
    let can_materialize = raw_changes.len() == accepted_changes.len();
    if can_materialize
        && let Err(completion) = charge_application_work(&ordered, budget, cancellation)
    {
        let heads = derive_heads(&accepted_changes, &controls);
        return incomplete_report(
            canonical_controls,
            dispositions,
            accepted_changes,
            integrity_alerts,
            completion,
        )
        .with_heads(heads);
    }
    let materialized = if can_materialize {
        match materialize_history(&raw_changes, &ordered) {
            Ok(document) => Some(document),
            Err(_) => {
                let heads = derive_heads(&accepted_changes, &controls);
                return incomplete_report(
                    canonical_controls,
                    dispositions,
                    accepted_changes,
                    integrity_alerts,
                    Completion::Complete,
                )
                .with_failure(EvaluationFailure::Apply)
                .with_heads(heads);
            }
        }
    } else {
        None
    };
    let derived_heads = derive_heads(&accepted_changes, &controls);
    let (heads, materialized_document) = match materialized {
        Some(AppliedDocument {
            heads,
            canonical_bytes,
        }) if applied_heads_agree(&derived_heads, &heads) => (heads, Some(canonical_bytes)),
        Some(_) => {
            return incomplete_report(
                canonical_controls,
                dispositions,
                accepted_changes,
                integrity_alerts,
                Completion::Complete,
            )
            .with_failure(EvaluationFailure::InvariantViolation)
            .with_heads(derived_heads);
        }
        None => (derived_heads, None),
    };
    BatchEvaluationReport {
        canonical_controls,
        control_dispositions,
        accepted_at_control,
        dispositions,
        accepted_changes,
        heads,
        materialized_document,
        integrity_alerts,
        completion,
        failure: None,
    }
}

enum EpochResolutionError {
    Schedule(ScheduleError),
    InvalidState,
}

fn resolve_authoritative_epoch(
    control: &BatchControl,
    accepted_base: AcceptedEpochState,
    ancestry: &[EventId],
    controls: &BTreeMap<EventId, BatchControl>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<BTreeMap<ChangeHash, ProtocolDisposition>, EpochResolutionError> {
    let Some(selected) = control.envelope.clone() else {
        let epoch_inputs = control.changes.iter().map(|change| EpochCandidate {
            candidate: change.candidate.clone(),
            semantically_valid: change.semantically_valid,
            canonical_control: !control.frozen,
        });
        return resolve_epoch(
            epoch_inputs,
            accepted_base.accepted_closure().clone(),
            budget,
            cancellation,
        )
        .map_err(EpochResolutionError::Schedule);
    };
    let canonical_ancestry = ancestry
        .iter()
        .filter_map(|event_id| {
            controls
                .get(event_id)
                .and_then(|control| control.envelope.clone())
        })
        .collect();
    let raw_changes = controls
        .values()
        .flat_map(|control| control.changes.iter())
        .filter_map(|change| {
            change
                .raw_change
                .clone()
                .map(|raw| (change.candidate.change_hash, raw))
        })
        .collect();
    let input = EpochEvaluationInput::new_with_raw(
        selected,
        accepted_base,
        control
            .changes
            .iter()
            .map(|change| (change.candidate.clone(), change.raw_change.clone())),
        raw_changes,
        canonical_ancestry,
    )
    .map_err(|_| EpochResolutionError::InvalidState)?;
    evaluate_epoch(&input, budget, cancellation)
        .map(|result| result.dispositions().clone())
        .map_err(|error| match error {
            EpochEvaluationError::Schedule(error) => EpochResolutionError::Schedule(error),
            EpochEvaluationError::Quarantine(QuarantineError::BudgetExhausted) => {
                EpochResolutionError::Schedule(ScheduleError::BudgetExhausted)
            }
            EpochEvaluationError::Quarantine(QuarantineError::Cancelled) => {
                EpochResolutionError::Schedule(ScheduleError::Cancelled)
            }
            EpochEvaluationError::Quarantine(QuarantineError::Alert(_))
            | EpochEvaluationError::Graph(_)
            | EpochEvaluationError::State(_) => EpochResolutionError::InvalidState,
        })
}

fn accepted_state_for_closure(
    accepted: &BTreeSet<ChangeHash>,
    controls: &BTreeMap<EventId, BatchControl>,
    parent: Option<&EpochEvaluationResult>,
) -> Option<AcceptedEpochState> {
    let candidates = controls
        .values()
        .flat_map(|control| control.changes.iter())
        .filter(|change| accepted.contains(&change.candidate.change_hash))
        .map(|change| (change.candidate.change_hash, change.candidate.clone()))
        .collect::<BTreeMap<_, _>>();
    if candidates.len() != accepted.len() {
        return None;
    }
    let depended_on = candidates
        .values()
        .flat_map(|candidate| candidate.dependencies.iter().copied())
        .filter(|hash| accepted.contains(hash))
        .collect::<BTreeSet<_>>();
    let heads = accepted.difference(&depended_on).copied().collect();
    let materialized = parent.and_then(|result| {
        (result.accepted_state().accepted_closure() == accepted)
            .then(|| result.accepted_state().materialized().cloned())
            .flatten()
    });
    let materialized = if accepted.is_empty() && materialized.is_none() {
        MaterializedDocumentView::empty().ok()
    } else {
        materialized
    };
    AcceptedEpochState::new(accepted.clone(), heads, candidates, materialized).ok()
}

fn epoch_result_from_accepted(
    accepted: &BTreeSet<ChangeHash>,
    controls: &BTreeMap<EventId, BatchControl>,
    dispositions: BTreeMap<ChangeHash, ProtocolDisposition>,
    integrity_alerts: Vec<IntegrityAlert>,
) -> Option<EpochEvaluationResult> {
    let candidates = controls
        .values()
        .flat_map(|control| control.changes.iter())
        .filter(|change| accepted.contains(&change.candidate.change_hash))
        .map(|change| (change.candidate.change_hash, change.candidate.clone()))
        .collect::<BTreeMap<_, _>>();
    if candidates.len() != accepted.len() {
        return None;
    }
    let depended_on = candidates
        .values()
        .flat_map(|candidate| candidate.dependencies.iter().copied())
        .filter(|hash| accepted.contains(hash))
        .collect::<BTreeSet<_>>();
    let heads = accepted.difference(&depended_on).copied().collect();
    EpochEvaluationResult::new(
        accepted.clone(),
        heads,
        candidates,
        dispositions,
        integrity_alerts,
        None,
    )
    .ok()
}

fn applied_heads_agree(
    derived_heads: &BTreeSet<ChangeHash>,
    applied_heads: &BTreeSet<ChangeHash>,
) -> bool {
    derived_heads == applied_heads
}

fn charge_application_work(
    ordered: &[ChangeHash],
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<(), Completion> {
    for _ in ordered {
        if cancellation.is_cancelled() {
            return Err(Completion::Cancelled);
        }
        budget
            .charge(WorkCounter::ApplyChange, 1)
            .map_err(|_| Completion::BudgetExhausted)?;
    }
    if cancellation.is_cancelled() {
        return Err(Completion::Cancelled);
    }
    budget
        .charge(WorkCounter::ApplyChange, 1)
        .map_err(|_| Completion::BudgetExhausted)
}

impl BatchEvaluationReport {
    fn with_heads(mut self, heads: BTreeSet<ChangeHash>) -> Self {
        self.heads = heads;
        self
    }

    fn with_failure(mut self, failure: EvaluationFailure) -> Self {
        self.failure = Some(failure);
        self
    }
}

fn incomplete_report(
    canonical_controls: Vec<EventId>,
    dispositions: BTreeMap<ChangeHash, ProtocolDisposition>,
    accepted_changes: BTreeSet<ChangeHash>,
    integrity_alerts: Vec<IntegrityAlert>,
    completion: Completion,
) -> BatchEvaluationReport {
    BatchEvaluationReport {
        canonical_controls,
        control_dispositions: BTreeMap::new(),
        accepted_at_control: BTreeMap::new(),
        dispositions,
        accepted_changes,
        heads: BTreeSet::new(),
        materialized_document: None,
        integrity_alerts,
        completion,
        failure: match completion {
            Completion::Complete => None,
            Completion::BudgetExhausted => Some(EvaluationFailure::BudgetExhausted),
            Completion::Cancelled => Some(EvaluationFailure::Cancelled),
        },
    }
}

fn derive_heads(
    accepted: &BTreeSet<ChangeHash>,
    controls: &BTreeMap<EventId, BatchControl>,
) -> BTreeSet<ChangeHash> {
    let dependencies = controls
        .values()
        .flat_map(|control| control.changes.iter())
        .filter(|change| accepted.contains(&change.candidate.change_hash))
        .flat_map(|change| change.candidate.dependencies.iter().copied())
        .collect::<BTreeSet<_>>();
    accepted.difference(&dependencies).copied().collect()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{AcceptedAtControl, BatchChange, BatchControl, evaluate_batch};
    use crate::automerge_adapter::decode::decode_change;
    use crate::graph::actor_state::tests::candidate;
    use crate::{
        ChangeHash, Completion, EvaluationFailure, EventId, NeverCancelled, ProtocolDisposition,
        ProtocolRevision, WorkBudget,
    };

    fn control(id: u8, parent: Option<u8>, changes: Vec<BatchChange>) -> BatchControl {
        BatchControl {
            event_id: EventId::from_bytes([id; 32]),
            parent: parent.map(|byte| EventId::from_bytes([byte; 32])),
            accepted_base: BTreeSet::new(),
            frozen: false,
            changes,
            envelope: None,
        }
    }

    fn change(hash: u8, actor: u8, sequence: u64) -> BatchChange {
        let mut value = candidate(actor, sequence, sequence, 1);
        value.change_hash = ChangeHash::from_bytes([hash; 32]);
        BatchChange {
            candidate: value,
            semantically_valid: true,
            raw_change: None,
        }
    }

    #[test]
    fn implement_full_batch_reference_evaluator() {
        let raw = include_str!("../../../../fixtures/v1_draft/automerge_changes/basic/change.hex")
            .trim()
            .as_bytes()
            .chunks_exact(2)
            .filter_map(|pair| {
                core::str::from_utf8(pair)
                    .ok()
                    .and_then(|text| u8::from_str_radix(text, 16).ok())
            })
            .collect::<Vec<_>>();
        let decoded = decode_change(&raw, ProtocolRevision::draft_v1());
        assert!(decoded.is_ok());
        let Ok(decoded) = decoded else { return };
        let mut basic = change(1, 1, 1);
        basic.candidate.change_hash = ChangeHash::from_bytes(*decoded.hash.as_bytes());
        basic.candidate.actor = crate::ActorId::from_bytes(*decoded.actor.as_bytes());
        basic.candidate.sequence = decoded.sequence;
        basic.candidate.start_op = decoded.start_op;
        basic.candidate.operation_count = u64::try_from(decoded.operations.len()).unwrap_or(0);
        basic.raw_change = Some(raw);
        let basic_report = evaluate_batch(
            [control(1, None, vec![basic.clone()])],
            &mut WorkBudget::new(0, 20),
            &NeverCancelled,
        );
        assert_eq!(basic_report.completion, Completion::Complete);
        assert_eq!(
            basic_report
                .accepted_at_control
                .get(&EventId::from_bytes([1; 32]))
                .map(AcceptedAtControl::accepted_closure),
            Some(&basic_report.accepted_changes),
        );
        assert!(basic_report.materialized_document.is_some());
        let final_schedule_exhausted = evaluate_batch(
            [control(1, None, vec![basic.clone()])],
            &mut WorkBudget::new(0, 6),
            &NeverCancelled,
        );
        assert_eq!(
            final_schedule_exhausted.completion,
            Completion::BudgetExhausted
        );
        assert_eq!(final_schedule_exhausted.accepted_changes.len(), 1);
        assert!(final_schedule_exhausted.materialized_document.is_none());

        let mut malformed = basic.clone();
        malformed.raw_change = Some(vec![0xff]);
        let materialization_failed = evaluate_batch(
            [control(1, None, vec![malformed])],
            &mut WorkBudget::new(0, 20),
            &NeverCancelled,
        );
        assert_eq!(materialization_failed.completion, Completion::Complete);
        assert_eq!(
            materialization_failed.failure,
            Some(EvaluationFailure::Apply)
        );
        assert!(materialization_failed.materialized_document.is_none());

        let concurrent = evaluate_batch(
            [control(1, None, vec![change(1, 1, 1), change(2, 2, 1)])],
            &mut WorkBudget::new(0, 20),
            &NeverCancelled,
        );
        assert_eq!(concurrent.accepted_changes.len(), 2);

        let mut invalid = change(3, 3, 1);
        invalid.semantically_valid = false;
        let revoked = evaluate_batch(
            [control(1, None, vec![invalid.clone()])],
            &mut WorkBudget::new(0, 20),
            &NeverCancelled,
        );
        assert_eq!(
            revoked.dispositions[&invalid.candidate.change_hash],
            ProtocolDisposition::Invalid
        );

        let forked = evaluate_batch(
            [control(2, None, vec![]), control(1, None, vec![])],
            &mut WorkBudget::new(0, 20),
            &NeverCancelled,
        );
        assert_eq!(
            forked.canonical_controls,
            vec![EventId::from_bytes([1; 32])]
        );
        assert_eq!(forked.integrity_alerts.len(), 1);

        let equivocated = evaluate_batch(
            [control(1, None, vec![change(1, 1, 1), change(2, 1, 1)])],
            &mut WorkBudget::new(0, 20),
            &NeverCancelled,
        );
        assert!(equivocated.accepted_changes.is_empty());
        assert_eq!(equivocated.integrity_alerts.len(), 1);

        let mut frozen = control(1, None, vec![change(1, 1, 1)]);
        frozen.frozen = true;
        let frozen = evaluate_batch([frozen], &mut WorkBudget::new(0, 20), &NeverCancelled);
        assert!(frozen.accepted_changes.is_empty());
        assert_eq!(
            frozen.dispositions.values().next(),
            Some(&ProtocolDisposition::Excluded)
        );

        let mut reversed = vec![control(2, None, vec![]), control(1, None, vec![basic])];
        let first = evaluate_batch(
            reversed.clone(),
            &mut WorkBudget::new(0, 20),
            &NeverCancelled,
        );
        reversed.reverse();
        let second = evaluate_batch(reversed, &mut WorkBudget::new(0, 20), &NeverCancelled);
        assert_eq!(first, second);
    }

    #[test]
    fn applied_head_agreement_is_required() {
        let accepted = BTreeSet::from([ChangeHash::from_bytes([1; 32])]);
        let controls = std::collections::BTreeMap::from([(
            EventId::from_bytes([2; 32]),
            control(2, None, vec![change(1, 1, 1)]),
        )]);
        let derived = super::derive_heads(&accepted, &controls);
        assert!(super::applied_heads_agree(&derived, &accepted));
        assert!(!super::applied_heads_agree(
            &derived,
            &BTreeSet::from([ChangeHash::from_bytes([9; 32])])
        ));
    }
}
