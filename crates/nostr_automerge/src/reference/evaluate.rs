use std::collections::{BTreeMap, BTreeSet};

use crate::automerge_adapter::document::{AppliedDocument, materialize_history};
use crate::control::select::select_with_alert;
use crate::graph::change_candidate::ChangeCandidate;
use crate::graph::dependency_graph::build_graph;
use crate::graph::equivocation::quarantine_equivocation_descendants;
use crate::graph::schedule::{ScheduleError, schedule_candidates};
use crate::reference::epoch::{EpochCandidate, resolve_epoch};
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
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BatchEvaluationReport {
    pub(crate) canonical_controls: Vec<EventId>,
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
    let controls = controls
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
    let mut canonical_controls = Vec::new();
    let mut integrity_alerts = Vec::new();
    let mut parent = None;
    while let Some(children) = by_parent.get(&parent) {
        if cancellation.is_cancelled() {
            return incomplete_report(
                canonical_controls,
                BTreeMap::new(),
                BTreeSet::new(),
                integrity_alerts,
                Completion::Cancelled,
            );
        }
        let candidate_count = u64::try_from(children.len()).unwrap_or(u64::MAX);
        if budget
            .charge(WorkCounter::Control, candidate_count)
            .is_err()
        {
            return incomplete_report(
                canonical_controls,
                BTreeMap::new(),
                BTreeSet::new(),
                integrity_alerts,
                Completion::BudgetExhausted,
            );
        }
        let (selection, alert) = select_with_alert(parent, children.iter().copied());
        let Some(selected) = selection.selected else {
            break;
        };
        canonical_controls.push(selected);
        if let Some(alert) = alert {
            integrity_alerts.push(alert);
        }
        parent = Some(selected);
    }
    let canonical_set = canonical_controls.iter().copied().collect::<BTreeSet<_>>();
    let mut dispositions = BTreeMap::new();
    for control in controls
        .values()
        .filter(|control| !canonical_set.contains(&control.event_id))
    {
        for change in &control.changes {
            dispositions.insert(change.candidate.change_hash, ProtocolDisposition::Excluded);
        }
    }
    let mut completion = Completion::Complete;
    let mut accepted_changes = BTreeSet::new();
    for control_id in &canonical_controls {
        let Some(control) = controls.get(control_id) else {
            continue;
        };
        if cancellation.is_cancelled() {
            completion = Completion::Cancelled;
            break;
        }
        if budget.charge(WorkCounter::Control, 1).is_err() {
            completion = Completion::BudgetExhausted;
            break;
        }
        for hash in accepted_changes.difference(&control.accepted_base) {
            dispositions.insert(*hash, ProtocolDisposition::Excluded);
        }
        accepted_changes = control.accepted_base.clone();
        let epoch_inputs = control.changes.iter().map(|change| EpochCandidate {
            candidate: change.candidate.clone(),
            semantically_valid: change.semantically_valid,
            canonical_control: !control.frozen,
        });
        let epoch = resolve_epoch(epoch_inputs, accepted_changes.clone(), budget, cancellation);
        let resolved = match epoch {
            Ok(resolved) => resolved,
            Err(ScheduleError::BudgetExhausted) => {
                completion = Completion::BudgetExhausted;
                break;
            }
            Err(ScheduleError::Cancelled) => {
                completion = Completion::Cancelled;
                break;
            }
        };
        dispositions.extend(resolved);
        let eligible = control
            .changes
            .iter()
            .filter(|change| change.semantically_valid && !control.frozen)
            .map(|change| change.candidate.clone())
            .collect::<Vec<_>>();
        if let Ok(graph) = build_graph(eligible.clone(), accepted_changes.clone())
            && let Ok(quarantine) = quarantine_equivocation_descendants(eligible, &graph)
        {
            for hash in &quarantine.quarantined {
                dispositions.insert(*hash, ProtocolDisposition::Excluded);
            }
            integrity_alerts.extend(quarantine.alerts);
        }
        accepted_changes.extend(dispositions.iter().filter_map(|(hash, disposition)| {
            (*disposition == ProtocolDisposition::Accepted).then_some(*hash)
        }));
        accepted_changes
            .retain(|hash| dispositions.get(hash) != Some(&ProtocolDisposition::Excluded));
    }

    if completion != Completion::Complete {
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
                    Completion::Failed,
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
                Completion::Failed,
            )
            .with_failure(EvaluationFailure::InvariantViolation)
            .with_heads(derived_heads);
        }
        None => (derived_heads, None),
    };
    BatchEvaluationReport {
        canonical_controls,
        dispositions,
        accepted_changes,
        heads,
        materialized_document,
        integrity_alerts,
        completion,
        failure: None,
    }
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
            Completion::Failed => Some(EvaluationFailure::InvariantViolation),
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

    use super::{BatchChange, BatchControl, evaluate_batch};
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
        assert!(basic_report.materialized_document.is_some());
        let final_schedule_exhausted = evaluate_batch(
            [control(1, None, vec![basic.clone()])],
            &mut WorkBudget::new(0, 4),
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
        assert_eq!(materialization_failed.completion, Completion::Failed);
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
