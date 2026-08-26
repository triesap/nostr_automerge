use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::control::epoch_state::AcceptedEpochState;
use crate::control::frontier::ParentFrontierReference;
use crate::graph::actor_state::EpochActorState;
use crate::reference::epoch_engine::EpochEvaluationResult;
use crate::reference::epoch_engine::{PriorChangeKnowledge, PriorKnowledgeState};
use crate::{ActorId, ChangeHash};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ParentEpochViewBuildError<E> {
    Work(E),
    Invariant,
}

#[derive(Clone)]
pub(crate) struct ParentEpochView {
    payload: ParentEpochPayload,
    frontier_knowledge: BTreeMap<ChangeHash, ParentFrontierReference>,
    inherited_prior: PriorKnowledgeState,
    additional_prior: BTreeMap<ChangeHash, PriorChangeKnowledge>,
}

#[derive(Clone)]
enum ParentEpochPayload {
    Empty,
    Shared(Arc<AcceptedEpochState>),
    #[cfg(test)]
    Parts {
        accepted: BTreeSet<ChangeHash>,
        heads: BTreeSet<ChangeHash>,
        dependencies: BTreeMap<ChangeHash, BTreeSet<ChangeHash>>,
        actors: BTreeMap<ActorId, EpochActorState>,
        writer_contributions: BTreeMap<ActorId, ChangeHash>,
    },
}

impl Default for ParentEpochView {
    fn default() -> Self {
        Self {
            payload: ParentEpochPayload::Empty,
            frontier_knowledge: BTreeMap::new(),
            inherited_prior: PriorKnowledgeState::default(),
            additional_prior: BTreeMap::new(),
        }
    }
}

impl ParentEpochView {
    pub(crate) fn from_accepted_state(state: Arc<AcceptedEpochState>) -> Self {
        Self {
            payload: ParentEpochPayload::Shared(state),
            frontier_knowledge: BTreeMap::new(),
            inherited_prior: PriorKnowledgeState::default(),
            additional_prior: BTreeMap::new(),
        }
    }

    pub(crate) fn from_result_metered<E>(
        result: &EpochEvaluationResult,
        mut visit: impl FnMut() -> Result<(), E>,
    ) -> Result<Self, ParentEpochViewBuildError<E>> {
        let mut view = Self::from_accepted_state(result.accepted_state_handle());
        let mut dispositions = result.dispositions().iter();
        for _ in 0..result.dispositions().len() {
            visit().map_err(ParentEpochViewBuildError::Work)?;
            let Some((hash, disposition)) = dispositions.next() else {
                return Err(ParentEpochViewBuildError::Invariant);
            };
            let knowledge = match disposition {
                crate::ProtocolDisposition::Accepted => {
                    ParentFrontierReference::AcceptedUnderParent
                }
                crate::ProtocolDisposition::Pending => ParentFrontierReference::PendingUnderParent,
                crate::ProtocolDisposition::Invalid => ParentFrontierReference::InvalidUnderParent,
                crate::ProtocolDisposition::Excluded => {
                    ParentFrontierReference::ExcludedUnderParent
                }
                crate::ProtocolDisposition::UnsupportedRevision => {
                    ParentFrontierReference::Unsupported
                }
            };
            visit().map_err(ParentEpochViewBuildError::Work)?;
            view.frontier_knowledge.insert(*hash, knowledge);
        }
        Ok(view)
    }

    /// Shares the immutable persistent prior-state tail in constant time.
    pub(crate) fn share_inherited_prior(&mut self, knowledge: &PriorKnowledgeState) {
        self.inherited_prior = knowledge.clone();
    }

    pub(crate) fn set_additional_prior_knowledge_metered<E>(
        &mut self,
        knowledge: &BTreeMap<ChangeHash, PriorChangeKnowledge>,
        mut visit: impl FnMut() -> Result<(), E>,
    ) -> Result<(), ParentEpochViewBuildError<E>> {
        let mut projected = BTreeMap::new();
        let mut items = knowledge.iter();
        for _ in 0..knowledge.len() {
            visit().map_err(ParentEpochViewBuildError::Work)?;
            let Some((hash, item)) = items.next() else {
                return Err(ParentEpochViewBuildError::Invariant);
            };
            visit().map_err(ParentEpochViewBuildError::Work)?;
            projected.insert(*hash, *item);
        }
        self.additional_prior = projected;
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn from_parts_for_test(
        accepted: BTreeSet<ChangeHash>,
        heads: BTreeSet<ChangeHash>,
        dependencies: BTreeMap<ChangeHash, BTreeSet<ChangeHash>>,
        actors: BTreeMap<ActorId, EpochActorState>,
        writer_contributions: BTreeMap<ActorId, ChangeHash>,
    ) -> Self {
        let frontier_knowledge = accepted
            .iter()
            .copied()
            .map(|hash| (hash, ParentFrontierReference::AcceptedUnderParent))
            .collect();
        Self {
            payload: ParentEpochPayload::Parts {
                accepted,
                heads,
                dependencies,
                actors,
                writer_contributions,
            },
            frontier_knowledge,
            inherited_prior: PriorKnowledgeState::default(),
            additional_prior: BTreeMap::new(),
        }
    }

    pub(crate) fn contains(&self, hash: &ChangeHash) -> bool {
        self.accepted().contains(hash)
    }

    pub(crate) fn accepted(&self) -> &BTreeSet<ChangeHash> {
        match &self.payload {
            ParentEpochPayload::Empty => empty_hash_set(),
            ParentEpochPayload::Shared(state) => state.accepted_closure(),
            #[cfg(test)]
            ParentEpochPayload::Parts { accepted, .. } => accepted,
        }
    }

    pub(crate) fn heads(&self) -> &BTreeSet<ChangeHash> {
        match &self.payload {
            ParentEpochPayload::Empty => empty_hash_set(),
            ParentEpochPayload::Shared(state) => state.frontier_heads(),
            #[cfg(test)]
            ParentEpochPayload::Parts { heads, .. } => heads,
        }
    }

    pub(crate) fn dependencies(&self, hash: &ChangeHash) -> Option<&BTreeSet<ChangeHash>> {
        match &self.payload {
            ParentEpochPayload::Empty => None,
            ParentEpochPayload::Shared(state) => state.dependencies().get(hash),
            #[cfg(test)]
            ParentEpochPayload::Parts { dependencies, .. } => dependencies.get(hash),
        }
    }

    pub(crate) fn dependency_index(&self) -> &BTreeMap<ChangeHash, BTreeSet<ChangeHash>> {
        match &self.payload {
            ParentEpochPayload::Empty => empty_dependency_map(),
            ParentEpochPayload::Shared(state) => state.dependencies(),
            #[cfg(test)]
            ParentEpochPayload::Parts { dependencies, .. } => dependencies,
        }
    }

    pub(crate) fn actor_state(&self, actor: &ActorId) -> Option<EpochActorState> {
        match &self.payload {
            ParentEpochPayload::Empty => None,
            ParentEpochPayload::Shared(state) => state.actor_states().get(actor).copied(),
            #[cfg(test)]
            ParentEpochPayload::Parts { actors, .. } => actors.get(actor).copied(),
        }
    }

    pub(crate) fn writer_contribution(&self, actor: &ActorId) -> Option<ChangeHash> {
        match &self.payload {
            ParentEpochPayload::Empty => None,
            ParentEpochPayload::Shared(state) => state.writer_contributions().get(actor).copied(),
            #[cfg(test)]
            ParentEpochPayload::Parts {
                writer_contributions,
                ..
            } => writer_contributions.get(actor).copied(),
        }
    }

    pub(crate) fn frontier_knowledge_metered<E>(
        &self,
        hash: &ChangeHash,
        mut visit: impl FnMut() -> Result<(), E>,
    ) -> Result<ParentFrontierReference, E> {
        visit()?;
        if let Some(knowledge) = self.frontier_knowledge.get(hash).copied() {
            return Ok(knowledge);
        }
        if let Some(knowledge) = self
            .inherited_prior
            .get_metered(hash, &mut visit)?
            .copied()
            .and_then(prior_frontier_reference)
        {
            return Ok(knowledge);
        }
        visit()?;
        if let Some(knowledge) = self
            .additional_prior
            .get(hash)
            .copied()
            .and_then(prior_frontier_reference)
        {
            return Ok(knowledge);
        }
        visit()?;
        Ok(if self.contains(hash) {
            ParentFrontierReference::AcceptedUnderParent
        } else {
            ParentFrontierReference::Unknown
        })
    }

    #[cfg(test)]
    pub(crate) fn frontier_knowledge(&self, hash: &ChangeHash) -> ParentFrontierReference {
        if let Some(knowledge) = self.frontier_knowledge.get(hash).copied() {
            return knowledge;
        }
        if let Some(knowledge) = self
            .inherited_prior
            .get(hash)
            .copied()
            .and_then(prior_frontier_reference)
        {
            return knowledge;
        }
        if let Some(knowledge) = self
            .additional_prior
            .get(hash)
            .copied()
            .and_then(prior_frontier_reference)
        {
            return knowledge;
        }
        if self.contains(hash) {
            ParentFrontierReference::AcceptedUnderParent
        } else {
            ParentFrontierReference::Unknown
        }
    }
}

const fn prior_frontier_reference(
    knowledge: PriorChangeKnowledge,
) -> Option<ParentFrontierReference> {
    Some(match knowledge {
        PriorChangeKnowledge::AcceptedInBase => ParentFrontierReference::AcceptedUnderParent,
        PriorChangeKnowledge::PrunedCanonicalAncestor
        | PriorChangeKnowledge::PriorEquivocationExcluded => {
            ParentFrontierReference::ExcludedUnderParent
        }
        PriorChangeKnowledge::KnownOtherControl => ParentFrontierReference::OtherControl,
        PriorChangeKnowledge::KnownInvalid => ParentFrontierReference::InvalidUnderParent,
        PriorChangeKnowledge::KnownUnsupported => ParentFrontierReference::Unsupported,
        PriorChangeKnowledge::SameEpochCandidate | PriorChangeKnowledge::Unknown => return None,
    })
}

fn empty_hash_set() -> &'static BTreeSet<ChangeHash> {
    static EMPTY: std::sync::OnceLock<BTreeSet<ChangeHash>> = std::sync::OnceLock::new();
    EMPTY.get_or_init(BTreeSet::new)
}

fn empty_dependency_map() -> &'static BTreeMap<ChangeHash, BTreeSet<ChangeHash>> {
    static EMPTY: std::sync::OnceLock<BTreeMap<ChangeHash, BTreeSet<ChangeHash>>> =
        std::sync::OnceLock::new();
    EMPTY.get_or_init(BTreeMap::new)
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::collections::{BTreeMap, BTreeSet};
    use std::rc::Rc;
    use std::sync::Arc;

    use super::{ParentEpochView, ParentEpochViewBuildError};
    use crate::automerge_adapter::materialized_view::MaterializedDocumentView;
    use crate::control::epoch_state::AcceptedEpochState;
    use crate::graph::actor_state::EpochActorState;
    use crate::graph::change_candidate::ChangeCandidate;
    use crate::reference::epoch_engine::{
        EpochEvaluationResult, PriorChangeKnowledge, PriorKnowledgeState,
    };
    use crate::{ActorId, ChangeHash, DevicePublicKey, EventId, ProtocolDisposition};

    fn candidate(actor: u8, hash: u8) -> ChangeCandidate {
        ChangeCandidate {
            change_hash: ChangeHash::from_bytes([hash; 32]),
            actor: ActorId::from_bytes([actor; 32]),
            sequence: 1,
            start_op: 1,
            operation_count: 1,
            dependencies: Vec::new().into(),
            control_id: EventId::from_bytes([7; 32]),
            author: DevicePublicKey::from_bytes([actor; 32]),
            valid_carriers: vec![EventId::from_bytes([hash; 32])].into(),
        }
    }

    fn empty_state() -> Arc<AcceptedEpochState> {
        let materialized = MaterializedDocumentView::empty_for_test();
        assert!(materialized.is_ok());
        let Ok(materialized) = materialized else {
            unreachable!()
        };
        let state = AcceptedEpochState::new(
            BTreeSet::new(),
            BTreeSet::new(),
            BTreeMap::new(),
            Some(materialized),
        );
        assert!(state.is_ok());
        let Ok(state) = state else { unreachable!() };
        Arc::new(state)
    }

    #[test]
    fn parent_result_projection_charges_before_every_read_and_insert() {
        let first = ChangeHash::from_bytes([1; 32]);
        let second = ChangeHash::from_bytes([2; 32]);
        let result = EpochEvaluationResult::from_shared_state(
            empty_state(),
            BTreeMap::from([
                (first, ProtocolDisposition::Accepted),
                (second, ProtocolDisposition::Invalid),
            ]),
            Vec::new(),
        );
        let exact = 4_usize;
        for capacity in 0..=exact + 1 {
            let observed = Cell::new(0_usize);
            let injected = Rc::new("parent projection stop");
            let projected = ParentEpochView::from_result_metered(&result, || {
                if observed.get() == capacity {
                    return Err(Rc::clone(&injected));
                }
                observed.set(observed.get() + 1);
                Ok(())
            });
            if capacity < exact {
                assert!(projected.is_err());
                let Err(ParentEpochViewBuildError::Work(returned)) = projected else {
                    return;
                };
                assert!(Rc::ptr_eq(&returned, &injected));
                assert_eq!(observed.get(), capacity);
            } else {
                assert!(projected.is_ok());
                let Ok(view) = projected else {
                    return;
                };
                assert_eq!(observed.get(), exact);
                assert_eq!(
                    view.frontier_knowledge.get(&first),
                    Some(&super::ParentFrontierReference::AcceptedUnderParent)
                );
                assert_eq!(
                    view.frontier_knowledge.get(&second),
                    Some(&super::ParentFrontierReference::InvalidUnderParent)
                );
            }
        }
    }

    #[test]
    fn additional_prior_projection_is_atomic_at_every_boundary() {
        let first = ChangeHash::from_bytes([3; 32]);
        let second = ChangeHash::from_bytes([4; 32]);
        let source = BTreeMap::from([
            (first, PriorChangeKnowledge::KnownInvalid),
            (second, PriorChangeKnowledge::KnownOtherControl),
        ]);
        let exact = 4_usize;
        for capacity in 0..=exact + 1 {
            let observed = Cell::new(0_usize);
            let injected = Rc::new("additional prior stop");
            let mut view = ParentEpochView::default();
            let projected = view.set_additional_prior_knowledge_metered(&source, || {
                if observed.get() == capacity {
                    return Err(Rc::clone(&injected));
                }
                observed.set(observed.get() + 1);
                Ok(())
            });
            if capacity < exact {
                assert!(projected.is_err());
                let Err(ParentEpochViewBuildError::Work(returned)) = projected else {
                    return;
                };
                assert!(Rc::ptr_eq(&returned, &injected));
                assert_eq!(observed.get(), capacity);
                assert!(view.additional_prior.is_empty());
            } else {
                assert_eq!(projected, Ok(()));
                assert_eq!(observed.get(), exact);
                assert_eq!(view.additional_prior, source);
            }
        }
    }

    #[test]
    fn frontier_knowledge_lookup_exposes_every_ordered_source_boundary() {
        const DEPTH: u8 = 64;
        let direct = ChangeHash::from_bytes([80; 32]);
        let additional = ChangeHash::from_bytes([65; 32]);
        let accepted = candidate(70, 70);
        let accepted_hash = accepted.change_hash;
        let materialized = MaterializedDocumentView::empty_for_test();
        assert!(materialized.is_ok());
        let Ok(materialized) = materialized else {
            return;
        };
        let state = AcceptedEpochState::new(
            BTreeSet::from([accepted_hash]),
            BTreeSet::from([accepted_hash]),
            BTreeMap::from([(accepted_hash, accepted)]),
            Some(materialized),
        );
        assert!(state.is_ok());
        let Ok(state) = state else { return };
        let mut view = ParentEpochView::from_accepted_state(Arc::new(state));
        view.frontier_knowledge
            .insert(direct, super::ParentFrontierReference::PendingUnderParent);
        let mut prior = PriorKnowledgeState::from(BTreeMap::from([(
            ChangeHash::from_bytes([0; 32]),
            PriorChangeKnowledge::KnownInvalid,
        )]));
        for value in 1..DEPTH {
            prior = prior.extend_local(BTreeMap::from([(
                ChangeHash::from_bytes([value; 32]),
                PriorChangeKnowledge::KnownInvalid,
            )]));
        }
        view.share_inherited_prior(&prior);
        let additional_source =
            BTreeMap::from([(additional, PriorChangeKnowledge::KnownOtherControl)]);
        assert_eq!(
            view.set_additional_prior_knowledge_metered(&additional_source, || Ok::<(), ()>(())),
            Ok(())
        );

        for (hash, expected, exact) in [
            (
                direct,
                super::ParentFrontierReference::PendingUnderParent,
                1_usize,
            ),
            (
                ChangeHash::from_bytes([DEPTH - 1; 32]),
                super::ParentFrontierReference::InvalidUnderParent,
                2,
            ),
            (
                ChangeHash::from_bytes([0; 32]),
                super::ParentFrontierReference::InvalidUnderParent,
                1 + usize::from(DEPTH),
            ),
            (
                additional,
                super::ParentFrontierReference::OtherControl,
                2 + usize::from(DEPTH),
            ),
            (
                accepted_hash,
                super::ParentFrontierReference::AcceptedUnderParent,
                3 + usize::from(DEPTH),
            ),
            (
                ChangeHash::from_bytes([99; 32]),
                super::ParentFrontierReference::Unknown,
                3 + usize::from(DEPTH),
            ),
        ] {
            for capacity in 0..=exact + 1 {
                let observed = Cell::new(0_usize);
                let injected = Rc::new("frontier knowledge stop");
                let result = view.frontier_knowledge_metered(&hash, || {
                    if observed.get() == capacity {
                        return Err(Rc::clone(&injected));
                    }
                    observed.set(observed.get() + 1);
                    Ok(())
                });
                if capacity < exact {
                    assert!(result.is_err());
                    let Err(returned) = result else {
                        return;
                    };
                    assert!(Rc::ptr_eq(&returned, &injected));
                    assert_eq!(observed.get(), capacity);
                } else {
                    assert_eq!(result, Ok(expected));
                    assert_eq!(observed.get(), exact);
                }
            }
        }
    }

    #[test]
    fn define_parent_accepted_history_query_interface() {
        let first = candidate(1, 2);
        let second = candidate(3, 4);
        let first_actor = first.actor;
        let second_actor = second.actor;
        let first_hash = first.change_hash;
        let second_hash = second.change_hash;
        let candidates = BTreeMap::from([(first_hash, first), (second_hash, second)]);
        let actors = BTreeMap::from([
            (
                first_actor,
                EpochActorState {
                    last_sequence: 1,
                    next_op: 2,
                    highest_change: first_hash,
                },
            ),
            (
                second_actor,
                EpochActorState {
                    last_sequence: 1,
                    next_op: 2,
                    highest_change: second_hash,
                },
            ),
        ]);
        let materialized = MaterializedDocumentView::empty_for_test();
        assert!(materialized.is_ok());
        let Ok(materialized) = materialized else {
            return;
        };
        let state = AcceptedEpochState::new(
            BTreeSet::from([first_hash, second_hash]),
            BTreeSet::from([first_hash, second_hash]),
            candidates,
            Some(materialized),
        );
        assert!(state.is_ok());
        let Ok(state) = state else {
            return;
        };
        let view = ParentEpochView::from_accepted_state(Arc::new(state));
        assert!(view.contains(&first_hash));
        assert_eq!(
            view.frontier_knowledge(&first_hash),
            crate::control::frontier::ParentFrontierReference::AcceptedUnderParent
        );
        assert_eq!(
            view.frontier_knowledge(&ChangeHash::from_bytes([99; 32])),
            crate::control::frontier::ParentFrontierReference::Unknown
        );
        assert_eq!(view.heads(), &BTreeSet::from([first_hash, second_hash]));
        assert_eq!(view.dependencies(&first_hash), Some(&BTreeSet::new()));
        assert_eq!(
            view.actor_state(&first_actor),
            actors.get(&first_actor).copied()
        );
        assert_eq!(view.writer_contribution(&second_actor), Some(second_hash));
    }

    #[test]
    fn finding_094_parent_epoch_view_shares_accepted_payload() {
        let retained = candidate(1, 2);
        let hash = retained.change_hash;
        let materialized = MaterializedDocumentView::empty_for_test();
        assert!(materialized.is_ok());
        let Ok(materialized) = materialized else {
            return;
        };
        let state = AcceptedEpochState::new(
            BTreeSet::from([hash]),
            BTreeSet::from([hash]),
            BTreeMap::from([(hash, retained)]),
            Some(materialized),
        );
        assert!(state.is_ok());
        let Ok(state) = state else {
            return;
        };
        let state = Arc::new(state);
        let view = ParentEpochView::from_accepted_state(Arc::clone(&state));
        let source = state.accepted_closure().iter().next();
        let inherited = view.accepted().iter().next();
        let shares_payload = matches!(
            (source, inherited),
            (Some(source), Some(inherited)) if core::ptr::eq(source, inherited)
        );
        assert!(
            shares_payload,
            "FINDING_094 reproduced: ParentEpochView deep-copies retained accepted state"
        );
    }
}
