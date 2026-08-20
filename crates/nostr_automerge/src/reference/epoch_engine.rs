use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};

use crate::ChangeHash;
use crate::automerge_adapter::materialized_view::MaterializedDocumentView;
use crate::control::epoch_state::AcceptedEpochState;
use crate::control::validate::ControlEnvelope;
use crate::graph::actor_state::{
    EpochActorState, apply_empty_counter, apply_nonempty_counter, initialize_actor_states,
    validate_actor_predecessor,
};
use crate::graph::change_candidate::ChangeCandidate;
use crate::graph::closure::{CandidateClosureError, candidate_dependency_closure};
use crate::graph::dependency_graph::{GraphBuildError, build_graph};
use crate::graph::epoch::{EpochAncestry, validate_epoch_ancestry};
use crate::graph::equivocation::{QuarantineError, quarantine_equivocation_descendants};
use crate::graph::schedule::ScheduleError;
use crate::reference::apply::apply_exact_closure;
use crate::reference::epoch::{EpochCandidate, resolve_epoch};
use crate::types::role::Role;
use crate::{ActorId, IntegrityAlert, ProtocolDisposition};
use crate::{CancellationCheck, WorkBudget, WorkCounter};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EpochEvaluationInputError {
    BaseFrontierMismatch,
    AncestryMismatch,
    CandidateControlMismatch,
    DuplicateCandidate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EpochEvaluationError {
    Schedule(ScheduleError),
    Graph(GraphBuildError),
    Quarantine(QuarantineError),
    State(crate::control::epoch_state::AcceptedEpochStateError),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PriorChangeKnowledge {
    AcceptedInBase,
    SameEpochCandidate,
    PrunedCanonicalAncestor,
    KnownOtherControl,
    KnownInvalid,
    KnownUnsupported,
    PriorEquivocationExcluded,
    Unknown,
}

impl PriorChangeKnowledge {
    pub(crate) const fn is_known_impossible(self) -> bool {
        matches!(
            self,
            Self::PrunedCanonicalAncestor
                | Self::KnownOtherControl
                | Self::KnownInvalid
                | Self::KnownUnsupported
                | Self::PriorEquivocationExcluded
        )
    }
}

/// Complete trusted input for evaluating one selected control epoch.
///
/// The accepted base is carried as one invariant-checked state object. Change
/// semantic validity is deliberately absent and must be derived by evaluation.
#[derive(Clone)]
pub(crate) struct EpochEvaluationInput<'a> {
    selected_control: ControlEnvelope,
    accepted_base: AcceptedEpochState,
    candidate_changes: BTreeMap<ChangeHash, ChangeCandidate>,
    raw_changes: Cow<'a, BTreeMap<ChangeHash, Vec<u8>>>,
    canonical_ancestry: Vec<ControlEnvelope>,
    prior_change_knowledge: BTreeMap<ChangeHash, PriorChangeKnowledge>,
}

impl EpochEvaluationInput<'static> {
    pub(crate) fn new(
        selected_control: ControlEnvelope,
        accepted_base: AcceptedEpochState,
        candidate_changes: impl IntoIterator<Item = ChangeCandidate>,
        canonical_ancestry: Vec<ControlEnvelope>,
    ) -> Result<Self, EpochEvaluationInputError> {
        Self::new_with_raw(
            selected_control,
            accepted_base,
            candidate_changes
                .into_iter()
                .map(|candidate| (candidate, None)),
            BTreeMap::new(),
            canonical_ancestry,
        )
    }

    pub(crate) fn new_with_raw(
        selected_control: ControlEnvelope,
        accepted_base: AcceptedEpochState,
        candidate_changes: impl IntoIterator<Item = (ChangeCandidate, Option<Vec<u8>>)>,
        raw_changes: BTreeMap<ChangeHash, Vec<u8>>,
        canonical_ancestry: Vec<ControlEnvelope>,
    ) -> Result<Self, EpochEvaluationInputError> {
        Self::new_with_raw_and_prior(
            selected_control,
            accepted_base,
            candidate_changes,
            raw_changes,
            canonical_ancestry,
            BTreeMap::new(),
        )
    }

    pub(crate) fn new_with_raw_and_prior(
        selected_control: ControlEnvelope,
        accepted_base: AcceptedEpochState,
        candidate_changes: impl IntoIterator<Item = (ChangeCandidate, Option<Vec<u8>>)>,
        mut raw_changes: BTreeMap<ChangeHash, Vec<u8>>,
        canonical_ancestry: Vec<ControlEnvelope>,
        prior_change_knowledge: BTreeMap<ChangeHash, PriorChangeKnowledge>,
    ) -> Result<Self, EpochEvaluationInputError> {
        let declared_heads = selected_control.base_heads().collect::<BTreeSet<_>>();
        if declared_heads != *accepted_base.frontier_heads() {
            return Err(EpochEvaluationInputError::BaseFrontierMismatch);
        }
        let expected_parent = canonical_ancestry.last().map(ControlEnvelope::event_id);
        if selected_control.parent() != expected_parent
            || canonical_ancestry.windows(2).any(|pair| {
                pair[1].parent() != Some(pair[0].event_id())
                    || pair[0]
                        .sequence()
                        .checked_add(1)
                        .is_none_or(|sequence| pair[1].sequence() != sequence)
            })
        {
            return Err(EpochEvaluationInputError::AncestryMismatch);
        }
        let selected_id = selected_control.event_id();
        let mut candidates = BTreeMap::new();
        for (candidate, raw) in candidate_changes {
            if candidate.control_id != selected_id {
                return Err(EpochEvaluationInputError::CandidateControlMismatch);
            }
            let hash = candidate.change_hash;
            if candidates.insert(hash, candidate).is_some() {
                return Err(EpochEvaluationInputError::DuplicateCandidate);
            }
            if let Some(raw) = raw {
                raw_changes.insert(hash, raw);
            }
        }
        Ok(Self {
            selected_control,
            accepted_base,
            candidate_changes: candidates,
            raw_changes: Cow::Owned(raw_changes),
            canonical_ancestry,
            prior_change_knowledge,
        })
    }
}

impl<'a> EpochEvaluationInput<'a> {
    pub(crate) fn new_with_borrowed_raw_and_prior(
        selected_control: ControlEnvelope,
        accepted_base: AcceptedEpochState,
        candidate_changes: impl IntoIterator<Item = ChangeCandidate>,
        raw_changes: &'a BTreeMap<ChangeHash, Vec<u8>>,
        canonical_ancestry: Vec<ControlEnvelope>,
        prior_change_knowledge: BTreeMap<ChangeHash, PriorChangeKnowledge>,
    ) -> Result<Self, EpochEvaluationInputError> {
        let mut input = EpochEvaluationInput::new_with_raw_and_prior(
            selected_control,
            accepted_base,
            candidate_changes
                .into_iter()
                .map(|candidate| (candidate, None)),
            BTreeMap::new(),
            canonical_ancestry,
            prior_change_knowledge,
        )?;
        input.raw_changes = Cow::Borrowed(raw_changes);
        Ok(input)
    }

    pub(crate) const fn selected_control(&self) -> &ControlEnvelope {
        &self.selected_control
    }

    pub(crate) const fn accepted_base(&self) -> &AcceptedEpochState {
        &self.accepted_base
    }

    pub(crate) const fn candidate_changes(&self) -> &BTreeMap<ChangeHash, ChangeCandidate> {
        &self.candidate_changes
    }

    pub(crate) fn raw_changes(&self) -> &BTreeMap<ChangeHash, Vec<u8>> {
        &self.raw_changes
    }

    pub(crate) fn canonical_ancestry(&self) -> &[ControlEnvelope] {
        &self.canonical_ancestry
    }

    pub(crate) const fn prior_change_knowledge(
        &self,
    ) -> &BTreeMap<ChangeHash, PriorChangeKnowledge> {
        &self.prior_change_knowledge
    }
}

/// Complete authoritative output of evaluating one selected control epoch.
#[derive(Clone)]
pub(crate) struct EpochEvaluationResult {
    accepted_state: AcceptedEpochState,
    dispositions: BTreeMap<ChangeHash, ProtocolDisposition>,
    integrity_alerts: Vec<IntegrityAlert>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AcceptedAtControl {
    accepted_closure: BTreeSet<ChangeHash>,
    frontier_heads: BTreeSet<ChangeHash>,
    actor_states: BTreeMap<ActorId, EpochActorState>,
}

impl AcceptedAtControl {
    pub(crate) fn from_result(result: &EpochEvaluationResult) -> Self {
        Self {
            accepted_closure: result.accepted_state().accepted_closure().clone(),
            frontier_heads: result.accepted_state().frontier_heads().clone(),
            actor_states: result.accepted_state().actor_states().clone(),
        }
    }

    pub(crate) const fn accepted_closure(&self) -> &BTreeSet<ChangeHash> {
        &self.accepted_closure
    }

    pub(crate) const fn frontier_heads(&self) -> &BTreeSet<ChangeHash> {
        &self.frontier_heads
    }

    pub(crate) const fn actor_states(&self) -> &BTreeMap<ActorId, EpochActorState> {
        &self.actor_states
    }

    #[cfg(test)]
    pub(crate) fn for_test(accepted_closure: BTreeSet<ChangeHash>) -> Self {
        Self {
            frontier_heads: accepted_closure.clone(),
            accepted_closure,
            actor_states: BTreeMap::new(),
        }
    }
}

impl EpochEvaluationResult {
    pub(crate) fn new(
        accepted_closure: BTreeSet<ChangeHash>,
        frontier_heads: BTreeSet<ChangeHash>,
        accepted_candidates: BTreeMap<ChangeHash, ChangeCandidate>,
        dispositions: BTreeMap<ChangeHash, ProtocolDisposition>,
        integrity_alerts: Vec<IntegrityAlert>,
        materialized: Option<MaterializedDocumentView>,
    ) -> Result<Self, crate::control::epoch_state::AcceptedEpochStateError> {
        let accepted_state = AcceptedEpochState::new(
            accepted_closure,
            frontier_heads,
            accepted_candidates,
            materialized,
        )?;
        Ok(Self {
            accepted_state,
            dispositions,
            integrity_alerts,
        })
    }

    pub(crate) const fn accepted_state(&self) -> &AcceptedEpochState {
        &self.accepted_state
    }

    pub(crate) const fn dispositions(&self) -> &BTreeMap<ChangeHash, ProtocolDisposition> {
        &self.dispositions
    }

    pub(crate) fn integrity_alerts(&self) -> &[IntegrityAlert] {
        &self.integrity_alerts
    }
}

pub(crate) fn evaluate_epoch(
    input: &EpochEvaluationInput,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<EpochEvaluationResult, EpochEvaluationError> {
    let selected = input.selected_control();
    let terminal = selected.content().terminal;
    let mut all_candidates = input.accepted_base().accepted_candidates().clone();
    all_candidates.extend(input.candidate_changes().clone());
    let mut epoch_candidates = Vec::with_capacity(input.candidate_changes().len());
    for candidate in input.candidate_changes().values().cloned() {
        let authorized = selected.content().members.iter().any(|member| {
            member.actor == candidate.actor
                && member.device == candidate.author
                && member.roles.contains(&Role::Write)
        });
        let closure =
            candidate_dependency_closure(&candidate, &all_candidates, budget, cancellation)
                .map_err(|error| EpochEvaluationError::Schedule(closure_schedule_error(error)))?;
        let complete_closure = closure.missing.is_empty().then_some(&closure.known);
        let actor_sequence_valid = complete_closure.is_none_or(|known| {
            validate_actor_predecessor(&candidate, known, &all_candidates).is_ok()
        });
        let actor_counter_valid = if let Some(known) = complete_closure {
            let base = known
                .iter()
                .filter_map(|hash| all_candidates.get(hash).cloned())
                .collect::<Vec<_>>();
            charge_actor_reconstruction(&base, budget, cancellation)
                .map_err(EpochEvaluationError::Schedule)?;
            initialize_actor_states(base).is_ok_and(|mut states| {
                if candidate.operation_count == 0 {
                    let depended_on = known
                        .iter()
                        .filter_map(|hash| all_candidates.get(hash))
                        .flat_map(|ancestor| ancestor.dependencies.iter().copied())
                        .filter(|hash| known.contains(hash))
                        .collect::<BTreeSet<_>>();
                    let mut current_heads = known
                        .difference(&depended_on)
                        .copied()
                        .collect::<BTreeSet<_>>();
                    current_heads.extend(
                        input
                            .accepted_base()
                            .frontier_heads()
                            .difference(known)
                            .copied(),
                    );
                    apply_empty_counter(&mut states, &candidate, &current_heads).is_ok()
                } else {
                    apply_nonempty_counter(&mut states, &candidate).is_ok()
                }
            })
        } else {
            true
        };
        let ancestry_valid = !matches!(
            validate_epoch_ancestry(
                input.accepted_base().frontier_heads(),
                &closure.known,
                &closure.missing,
            ),
            EpochAncestry::InvalidOmission(_)
        );
        let prior_dependencies_valid =
            !candidate
                .dependencies
                .iter()
                .chain(&closure.missing)
                .any(|dependency| {
                    input
                        .prior_change_knowledge()
                        .get(dependency)
                        .is_some_and(|knowledge| knowledge.is_known_impossible())
                });
        let prior_semantics_valid = authorized
            && actor_sequence_valid
            && actor_counter_valid
            && ancestry_valid
            && prior_dependencies_valid;
        let application_valid = if !prior_semantics_valid {
            false
        } else if !closure.missing.is_empty() {
            true
        } else if !closure.cyclic.is_empty() {
            false
        } else {
            input
                .raw_changes()
                .get(&candidate.change_hash)
                .is_some_and(|raw| {
                    let closure_raw = closure
                        .known
                        .iter()
                        .filter_map(|hash| {
                            input
                                .raw_changes()
                                .get(hash)
                                .cloned()
                                .map(|raw| (*hash, raw))
                        })
                        .collect::<BTreeMap<_, _>>();
                    closure_raw.len() == closure.known.len()
                        && apply_exact_closure(
                            &closure_raw,
                            &closure.ordered,
                            candidate.change_hash,
                            raw,
                            &candidate.dependencies.iter().copied().collect(),
                        )
                        .is_ok()
                })
        };
        let semantically_valid = prior_semantics_valid && application_valid;
        epoch_candidates.push(EpochCandidate {
            candidate,
            semantically_valid,
            canonical_control: !terminal,
        });
    }
    let mut dispositions = resolve_epoch(
        epoch_candidates,
        input.accepted_base().accepted_closure().clone(),
        budget,
        cancellation,
    )
    .map_err(EpochEvaluationError::Schedule)?;
    let eligible = all_candidates
        .values()
        .filter(|candidate| {
            input
                .accepted_base()
                .accepted_closure()
                .contains(&candidate.change_hash)
                || dispositions.get(&candidate.change_hash) == Some(&ProtocolDisposition::Accepted)
        })
        .cloned()
        .collect::<Vec<_>>();
    let graph = build_graph(
        eligible.clone(),
        input.accepted_base().accepted_closure().clone(),
    )
    .map_err(EpochEvaluationError::Graph)?;
    let quarantine = quarantine_equivocation_descendants(eligible, &graph, budget, cancellation)
        .map_err(EpochEvaluationError::Quarantine)?;
    for hash in &quarantine.quarantined {
        dispositions.insert(*hash, ProtocolDisposition::Excluded);
    }
    let mut accepted_candidates = input.accepted_base().accepted_candidates().clone();
    accepted_candidates.extend(
        input
            .candidate_changes()
            .iter()
            .filter_map(|(hash, candidate)| {
                (dispositions.get(hash) == Some(&ProtocolDisposition::Accepted))
                    .then_some((*hash, candidate.clone()))
            }),
    );
    accepted_candidates
        .retain(|hash, _| dispositions.get(hash) != Some(&ProtocolDisposition::Excluded));
    let accepted_closure = accepted_candidates.keys().copied().collect::<BTreeSet<_>>();
    let depended_on = accepted_candidates
        .values()
        .flat_map(|candidate| candidate.dependencies.iter().copied())
        .filter(|hash| accepted_closure.contains(hash))
        .collect::<BTreeSet<_>>();
    let frontier_heads = accepted_closure.difference(&depended_on).copied().collect();
    let materialized = if accepted_closure == *input.accepted_base().accepted_closure() {
        input.accepted_base().materialized().cloned()
    } else {
        None
    };
    charge_actor_reconstruction(
        &accepted_candidates.values().cloned().collect::<Vec<_>>(),
        budget,
        cancellation,
    )
    .map_err(EpochEvaluationError::Schedule)?;
    EpochEvaluationResult::new(
        accepted_closure,
        frontier_heads,
        accepted_candidates,
        dispositions,
        quarantine.alerts,
        materialized,
    )
    .map_err(EpochEvaluationError::State)
}

fn charge_actor_reconstruction(
    candidates: &[ChangeCandidate],
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<(), ScheduleError> {
    if cancellation.is_cancelled() {
        return Err(ScheduleError::Cancelled);
    }
    let nodes = u64::try_from(candidates.len())
        .ok()
        .and_then(|count| count.checked_mul(2))
        .ok_or(ScheduleError::BudgetExhausted)?;
    let edges = candidates.iter().try_fold(0_u64, |total, candidate| {
        u64::try_from(candidate.dependencies.len())
            .ok()
            .and_then(|count| count.checked_mul(2))
            .and_then(|count| total.checked_add(count))
    });
    let edges = edges.ok_or(ScheduleError::BudgetExhausted)?;
    budget
        .charge(WorkCounter::GraphNode, nodes)
        .map_err(|_| ScheduleError::BudgetExhausted)?;
    if cancellation.is_cancelled() {
        return Err(ScheduleError::Cancelled);
    }
    budget
        .charge(WorkCounter::GraphEdge, edges)
        .map_err(|_| ScheduleError::BudgetExhausted)
}

const fn closure_schedule_error(error: CandidateClosureError) -> ScheduleError {
    match error {
        CandidateClosureError::BudgetExhausted => ScheduleError::BudgetExhausted,
        CandidateClosureError::Cancelled => ScheduleError::Cancelled,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::{
        AcceptedAtControl, EpochEvaluationInput, EpochEvaluationInputError, EpochEvaluationResult,
        PriorChangeKnowledge, charge_actor_reconstruction, evaluate_epoch,
    };
    use crate::automerge_adapter::materialized_view::MaterializedDocumentView;
    use crate::carrier::control::{ValidatedControlCarrier, ValidatedControlContent};
    use crate::control::epoch_state::{AcceptedEpochState, AcceptedEpochStateError};
    use crate::control::validate::ControlEnvelope;
    use crate::graph::actor_state::EpochActorState;
    use crate::graph::change_candidate::ChangeCandidate;
    use crate::{
        ActorId, ChangeHash, ControllerPublicKey, DevicePublicKey, DocumentCoordinate, DocumentId,
        EventId, NeverCancelled, ProtocolDisposition, WorkBudget, WorkCounter,
    };

    fn control(base_heads: Vec<ChangeHash>) -> ControlEnvelope {
        let controller = ControllerPublicKey::from_bytes([1; 32]);
        let coordinate = DocumentCoordinate::new(controller, DocumentId::from_bytes([2; 32]));
        ControlEnvelope::from_validated(ValidatedControlCarrier::for_test(
            EventId::from_bytes([3; 32]),
            controller,
            coordinate,
            None,
            ValidatedControlContent {
                base_heads,
                members: Vec::new(),
                predecessor: None,
                sequence: 0,
                successor: None,
                terminal: true,
            },
        ))
    }

    #[allow(clippy::expect_used)]
    fn empty_state() -> AcceptedEpochState {
        AcceptedEpochState::new(
            BTreeSet::new(),
            BTreeSet::new(),
            BTreeMap::new(),
            MaterializedDocumentView::empty_for_test().ok(),
        )
        .expect("consistent empty accepted state")
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn accepts_only_complete_trusted_epoch_input() {
        let input = EpochEvaluationInput::new(control(Vec::new()), empty_state(), [], Vec::new())
            .expect("complete genesis input");
        assert_eq!(
            input.selected_control().event_id(),
            EventId::from_bytes([3; 32])
        );
        assert!(input.accepted_base().accepted_closure().is_empty());
        assert!(input.candidate_changes().is_empty());
        assert!(input.canonical_ancestry().is_empty());
        let evaluated = evaluate_epoch(&input, &mut WorkBudget::new(1_000, 1_000), &NeverCancelled);
        assert!(evaluated.is_ok());
        assert!(evaluated.is_ok_and(|result| {
            result.accepted_state().accepted_closure().is_empty()
                && result.accepted_state().materialized().is_some()
        }));

        let head = ChangeHash::from_bytes([4; 32]);
        assert!(matches!(
            EpochEvaluationInput::new(control(vec![head]), empty_state(), [], Vec::new()),
            Err(EpochEvaluationInputError::BaseFrontierMismatch)
        ));
    }

    #[test]
    fn only_known_impossible_dependency_states_invalidate() {
        for usable in [
            PriorChangeKnowledge::AcceptedInBase,
            PriorChangeKnowledge::SameEpochCandidate,
            PriorChangeKnowledge::Unknown,
        ] {
            assert!(!usable.is_known_impossible());
        }
        for impossible in [
            PriorChangeKnowledge::PrunedCanonicalAncestor,
            PriorChangeKnowledge::KnownOtherControl,
            PriorChangeKnowledge::KnownInvalid,
            PriorChangeKnowledge::KnownUnsupported,
            PriorChangeKnowledge::PriorEquivocationExcluded,
        ] {
            assert!(impossible.is_known_impossible());
        }
    }

    #[test]
    fn actor_reconstruction_precharge_is_bounded_and_atomic() {
        let candidate = ChangeCandidate {
            change_hash: ChangeHash::from_bytes([5; 32]),
            actor: ActorId::from_bytes([6; 32]),
            sequence: 1,
            start_op: 1,
            operation_count: 1,
            dependencies: Vec::new(),
            control_id: EventId::from_bytes([3; 32]),
            author: DevicePublicKey::from_bytes([7; 32]),
            valid_carriers: BTreeSet::new(),
        };
        let mut exhausted = WorkBudget::new(0, 1);
        assert_eq!(
            charge_actor_reconstruction(
                std::slice::from_ref(&candidate),
                &mut exhausted,
                &NeverCancelled,
            ),
            Err(crate::graph::schedule::ScheduleError::BudgetExhausted)
        );
        assert_eq!(exhausted.consumed().get(WorkCounter::GraphNode), 0);

        let mut exact = WorkBudget::new(0, 2);
        assert!(charge_actor_reconstruction(&[candidate], &mut exact, &NeverCancelled).is_ok());
        assert_eq!(exact.consumed().get(WorkCounter::GraphNode), 2);
    }

    #[test]
    fn derives_actor_state_before_input_construction() {
        let hash = ChangeHash::from_bytes([5; 32]);
        let actor = ActorId::from_bytes([6; 32]);
        let candidate = ChangeCandidate {
            change_hash: hash,
            actor,
            sequence: 1,
            start_op: 1,
            operation_count: 1,
            dependencies: Vec::new(),
            control_id: EventId::from_bytes([3; 32]),
            author: DevicePublicKey::from_bytes([7; 32]),
            valid_carriers: BTreeSet::from([EventId::from_bytes([8; 32])]),
        };
        let accepted = AcceptedEpochState::new(
            BTreeSet::from([hash]),
            BTreeSet::from([hash]),
            BTreeMap::from([(hash, candidate)]),
            MaterializedDocumentView::empty_for_test().ok(),
        );
        assert!(accepted.is_ok_and(|state| {
            state.actor_states().get(&actor).is_some_and(|actor_state| {
                actor_state.last_sequence == 1
                    && actor_state.next_op == 2
                    && actor_state.highest_change == hash
            })
        }));
    }

    #[test]
    fn epoch_result_requires_complete_consistent_accepted_state() {
        let hash = ChangeHash::from_bytes([9; 32]);
        let actor = ActorId::from_bytes([10; 32]);
        let candidate = ChangeCandidate {
            change_hash: hash,
            actor,
            sequence: 1,
            start_op: 1,
            operation_count: 1,
            dependencies: Vec::new(),
            control_id: EventId::from_bytes([3; 32]),
            author: DevicePublicKey::from_bytes([11; 32]),
            valid_carriers: BTreeSet::from([EventId::from_bytes([12; 32])]),
        };
        let actors = BTreeMap::from([(
            actor,
            EpochActorState {
                last_sequence: 1,
                next_op: 2,
                highest_change: hash,
            },
        )]);
        let dispositions = BTreeMap::from([(hash, ProtocolDisposition::Accepted)]);
        let result = EpochEvaluationResult::new(
            BTreeSet::from([hash]),
            BTreeSet::from([hash]),
            BTreeMap::from([(hash, candidate.clone())]),
            dispositions.clone(),
            Vec::new(),
            MaterializedDocumentView::empty_for_test().ok(),
        );
        assert!(result.is_ok());
        let Ok(result) = result else {
            return;
        };
        assert_eq!(
            result.accepted_state().frontier_heads(),
            &BTreeSet::from([hash])
        );
        assert_eq!(result.accepted_state().actor_states(), &actors);
        assert_eq!(result.dispositions(), &dispositions);
        assert!(result.integrity_alerts().is_empty());
        let snapshot = AcceptedAtControl::from_result(&result);
        assert_eq!(snapshot.accepted_closure(), &BTreeSet::from([hash]));
        assert_eq!(snapshot.frontier_heads(), &BTreeSet::from([hash]));
        assert_eq!(snapshot.actor_states(), &actors);

        let bad_head = EpochEvaluationResult::new(
            BTreeSet::from([hash]),
            BTreeSet::from([ChangeHash::from_bytes([13; 32])]),
            BTreeMap::from([(hash, candidate.clone())]),
            dispositions.clone(),
            Vec::new(),
            None,
        );
        assert!(matches!(
            bad_head,
            Err(AcceptedEpochStateError::FrontierMismatch)
        ));

        let derived = EpochEvaluationResult::new(
            BTreeSet::from([hash]),
            BTreeSet::from([hash]),
            BTreeMap::from([(hash, candidate)]),
            dispositions,
            Vec::new(),
            None,
        );
        assert!(derived.is_ok_and(|result| result.accepted_state().actor_states() == &actors));
    }
}
