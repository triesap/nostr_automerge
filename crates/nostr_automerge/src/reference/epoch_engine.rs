use std::collections::{BTreeMap, BTreeSet};

use crate::ChangeHash;
use crate::automerge_adapter::materialized_view::MaterializedDocumentView;
use crate::control::epoch_state::AcceptedEpochState;
use crate::control::validate::ControlEnvelope;
use crate::graph::actor_state::{ActorStateError, EpochActorState, initialize_actor_states};
use crate::graph::change_candidate::ChangeCandidate;
use crate::graph::schedule::ScheduleError;
use crate::reference::epoch::{EpochCandidate, resolve_epoch};
use crate::types::role::Role;
use crate::{ActorId, IntegrityAlert, ProtocolDisposition};
use crate::{CancellationCheck, WorkBudget};

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
    ActorState(ActorStateError),
    State(crate::control::epoch_state::AcceptedEpochStateError),
}

/// Complete trusted input for evaluating one selected control epoch.
///
/// The accepted base is carried as one invariant-checked state object. Change
/// semantic validity is deliberately absent and must be derived by evaluation.
#[derive(Clone)]
pub(crate) struct EpochEvaluationInput {
    selected_control: ControlEnvelope,
    accepted_base: AcceptedEpochState,
    candidate_changes: BTreeMap<ChangeHash, ChangeCandidate>,
    canonical_ancestry: Vec<ControlEnvelope>,
}

impl EpochEvaluationInput {
    pub(crate) fn new(
        selected_control: ControlEnvelope,
        accepted_base: AcceptedEpochState,
        candidate_changes: impl IntoIterator<Item = ChangeCandidate>,
        canonical_ancestry: Vec<ControlEnvelope>,
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
        for candidate in candidate_changes {
            if candidate.control_id != selected_id {
                return Err(EpochEvaluationInputError::CandidateControlMismatch);
            }
            if candidates
                .insert(candidate.change_hash, candidate)
                .is_some()
            {
                return Err(EpochEvaluationInputError::DuplicateCandidate);
            }
        }
        Ok(Self {
            selected_control,
            accepted_base,
            candidate_changes: candidates,
            canonical_ancestry,
        })
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

    pub(crate) fn canonical_ancestry(&self) -> &[ControlEnvelope] {
        &self.canonical_ancestry
    }
}

/// Complete authoritative output of evaluating one selected control epoch.
#[derive(Clone)]
pub(crate) struct EpochEvaluationResult {
    accepted_state: AcceptedEpochState,
    dispositions: BTreeMap<ChangeHash, ProtocolDisposition>,
    integrity_alerts: Vec<IntegrityAlert>,
}

impl EpochEvaluationResult {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        accepted_closure: BTreeSet<ChangeHash>,
        frontier_heads: BTreeSet<ChangeHash>,
        accepted_candidates: BTreeMap<ChangeHash, ChangeCandidate>,
        actor_states: BTreeMap<ActorId, EpochActorState>,
        writer_contributions: BTreeMap<ActorId, ChangeHash>,
        dispositions: BTreeMap<ChangeHash, ProtocolDisposition>,
        integrity_alerts: Vec<IntegrityAlert>,
        materialized: Option<MaterializedDocumentView>,
    ) -> Result<Self, crate::control::epoch_state::AcceptedEpochStateError> {
        let accepted_state = AcceptedEpochState::new(
            accepted_closure,
            frontier_heads,
            accepted_candidates,
            actor_states,
            writer_contributions,
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
    let epoch_candidates = input
        .candidate_changes()
        .values()
        .cloned()
        .map(|candidate| {
            let authorized = selected.content().members.iter().any(|member| {
                member.actor == candidate.actor
                    && member.device == candidate.author
                    && member.roles.contains(&Role::Write)
            });
            EpochCandidate {
                candidate,
                semantically_valid: authorized && !terminal,
                canonical_control: true,
            }
        });
    let dispositions = resolve_epoch(
        epoch_candidates,
        input.accepted_base().accepted_closure().clone(),
        budget,
        cancellation,
    )
    .map_err(EpochEvaluationError::Schedule)?;
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
    let accepted_closure = accepted_candidates.keys().copied().collect::<BTreeSet<_>>();
    let actor_states = initialize_actor_states(accepted_candidates.values().cloned())
        .map_err(EpochEvaluationError::ActorState)?;
    let writer_contributions = actor_states
        .iter()
        .map(|(actor, state)| (*actor, state.highest_change))
        .collect();
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
    EpochEvaluationResult::new(
        accepted_closure,
        frontier_heads,
        accepted_candidates,
        actor_states,
        writer_contributions,
        dispositions,
        Vec::new(),
        materialized,
    )
    .map_err(EpochEvaluationError::State)
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::{
        EpochEvaluationInput, EpochEvaluationInputError, EpochEvaluationResult, evaluate_epoch,
    };
    use crate::automerge_adapter::materialized_view::MaterializedDocumentView;
    use crate::carrier::control::{ValidatedControlCarrier, ValidatedControlContent};
    use crate::control::epoch_state::{AcceptedEpochState, AcceptedEpochStateError};
    use crate::control::validate::ControlEnvelope;
    use crate::graph::actor_state::EpochActorState;
    use crate::graph::change_candidate::ChangeCandidate;
    use crate::{
        ActorId, ChangeHash, ControllerPublicKey, DevicePublicKey, DocumentCoordinate, DocumentId,
        EventId, NeverCancelled, ProtocolDisposition, WorkBudget,
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
            BTreeMap::new(),
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
    fn rejects_inconsistent_actor_state_before_input_construction() {
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
        let inconsistent = AcceptedEpochState::new(
            BTreeSet::from([hash]),
            BTreeSet::from([hash]),
            BTreeMap::from([(hash, candidate)]),
            BTreeMap::from([(
                actor,
                EpochActorState {
                    last_sequence: 2,
                    next_op: 2,
                    highest_change: hash,
                },
            )]),
            BTreeMap::from([(actor, hash)]),
            MaterializedDocumentView::empty_for_test().ok(),
        );
        assert!(matches!(
            inconsistent,
            Err(AcceptedEpochStateError::ActorStateMismatch)
        ));
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
            actors.clone(),
            BTreeMap::from([(actor, hash)]),
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

        let bad_head = EpochEvaluationResult::new(
            BTreeSet::from([hash]),
            BTreeSet::from([ChangeHash::from_bytes([13; 32])]),
            BTreeMap::from([(hash, candidate.clone())]),
            actors.clone(),
            BTreeMap::from([(actor, hash)]),
            dispositions.clone(),
            Vec::new(),
            None,
        );
        assert!(matches!(
            bad_head,
            Err(AcceptedEpochStateError::FrontierMismatch)
        ));

        let bad_actor = EpochEvaluationResult::new(
            BTreeSet::from([hash]),
            BTreeSet::from([hash]),
            BTreeMap::from([(hash, candidate)]),
            BTreeMap::new(),
            BTreeMap::from([(actor, hash)]),
            dispositions,
            Vec::new(),
            None,
        );
        assert!(matches!(
            bad_actor,
            Err(AcceptedEpochStateError::ActorStateMismatch)
        ));
    }
}
