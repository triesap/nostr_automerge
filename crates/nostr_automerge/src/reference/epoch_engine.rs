use std::collections::{BTreeMap, BTreeSet};

use crate::ChangeHash;
use crate::control::epoch_state::AcceptedEpochState;
use crate::control::validate::ControlEnvelope;
use crate::graph::change_candidate::ChangeCandidate;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EpochEvaluationInputError {
    BaseFrontierMismatch,
    AncestryMismatch,
    CandidateControlMismatch,
    DuplicateCandidate,
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

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::{EpochEvaluationInput, EpochEvaluationInputError};
    use crate::automerge_adapter::materialized_view::MaterializedDocumentView;
    use crate::carrier::control::{ValidatedControlCarrier, ValidatedControlContent};
    use crate::control::epoch_state::{AcceptedEpochState, AcceptedEpochStateError};
    use crate::control::validate::ControlEnvelope;
    use crate::graph::actor_state::EpochActorState;
    use crate::graph::change_candidate::ChangeCandidate;
    use crate::{
        ActorId, ChangeHash, ControllerPublicKey, DevicePublicKey, DocumentCoordinate, DocumentId,
        EventId,
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
}
