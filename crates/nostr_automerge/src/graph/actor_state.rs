use std::collections::{BTreeMap, BTreeSet};

use super::change_candidate::ChangeCandidate;
use crate::{ActorId, ChangeHash, WorkCounter};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct EpochActorState {
    pub(crate) last_sequence: u64,
    pub(crate) next_op: u64,
    pub(crate) highest_change: ChangeHash,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ActorStateError {
    NoncanonicalInput,
    SequenceGap,
    Equivocation,
    OperationCounter,
    MissingPredecessor,
    ParallelPredecessor,
    SequenceRollback,
    EmptyChange,
    NonEmptyChange,
    DependencyFrontier,
    MissingDependency,
    DependencyCycle,
}

trait EpochProjectionSource<'a> {
    fn member_count(&self) -> usize;
    fn next_member(&mut self) -> Option<ChangeHash>;
    fn accepted_member(&mut self, hash: &ChangeHash) -> bool;
    fn candidate(&mut self, hash: &ChangeHash) -> Option<&'a ChangeCandidate>;
    fn dependency_count(&mut self, candidate: &ChangeCandidate) -> usize;
    fn dependency(&mut self, candidate: &ChangeCandidate, index: usize) -> Option<ChangeHash>;
}

struct CanonicalEpochProjectionSource<'a> {
    members: std::collections::btree_set::Iter<'a, ChangeHash>,
    accepted_closure: &'a BTreeSet<ChangeHash>,
    changes: &'a BTreeMap<ChangeHash, ChangeCandidate>,
}

impl<'a> CanonicalEpochProjectionSource<'a> {
    fn new(
        accepted_closure: &'a BTreeSet<ChangeHash>,
        changes: &'a BTreeMap<ChangeHash, ChangeCandidate>,
    ) -> Self {
        Self {
            members: accepted_closure.iter(),
            accepted_closure,
            changes,
        }
    }
}

impl<'a> EpochProjectionSource<'a> for CanonicalEpochProjectionSource<'a> {
    fn member_count(&self) -> usize {
        self.accepted_closure.len()
    }

    fn next_member(&mut self) -> Option<ChangeHash> {
        self.members.next().copied()
    }

    fn accepted_member(&mut self, hash: &ChangeHash) -> bool {
        self.accepted_closure.contains(hash)
    }

    fn candidate(&mut self, hash: &ChangeHash) -> Option<&'a ChangeCandidate> {
        self.changes.get(hash)
    }

    fn dependency_count(&mut self, candidate: &ChangeCandidate) -> usize {
        candidate.dependencies.len()
    }

    fn dependency(&mut self, candidate: &ChangeCandidate, index: usize) -> Option<ChangeHash> {
        candidate.dependencies.get(index).copied()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MeteredActorStateError<E> {
    Work(E),
    State(ActorStateError),
}

/// Immutable accepted-closure facts used by authoritative epoch semantics.
///
/// Construction is sealed in this module. Consumers receive only copied
/// scalar facts or ordered iterators and cannot mutate the trusted maps.
pub(crate) struct TrustedEpochProjection<'a> {
    branch_membership: &'a BTreeMap<ChangeHash, ChangeCandidate>,
    accepted_closure: &'a BTreeSet<ChangeHash>,
    dependencies: BTreeMap<ChangeHash, BTreeSet<ChangeHash>>,
    frontier_heads: BTreeSet<ChangeHash>,
    actor_states: BTreeMap<ActorId, EpochActorState>,
    writer_contributions: BTreeMap<ActorId, ChangeHash>,
    causal_next_op: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ActorDecisionOperation {
    ActorStateRead,
    PredecessorCandidateRead,
    ActorIdentityDecision,
    SequenceRelationDecision,
}

macro_rules! actor_decision_sites {
    ($( $site:ident => $operation:ident ),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        enum ActorDecisionSite {
            $( $site, )+
        }

        impl ActorDecisionSite {
            const fn descriptor(self) -> ActorDecisionDescriptor {
                ActorDecisionDescriptor {
                    site: self,
                    site_id: match self {
                        $( Self::$site => stringify!($site), )+
                    },
                    phase: "actor_sequence",
                    operation: match self {
                        $( Self::$site => ActorDecisionOperation::$operation, )+
                    },
                    counter: WorkCounter::GraphNode,
                    abstract_owner_class: "direct_operation",
                    applicability: "public_rust",
                }
            }
        }
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ActorDecisionDescriptor {
    site: ActorDecisionSite,
    site_id: &'static str,
    phase: &'static str,
    operation: ActorDecisionOperation,
    counter: WorkCounter,
    abstract_owner_class: &'static str,
    applicability: &'static str,
}

impl ActorDecisionDescriptor {
    const fn operation(self) -> ActorDecisionOperation {
        self.operation
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ActorDecisionObservationKind {
    ChargeAttempt,
    TargetCompleted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ActorDecisionObservation {
    descriptor: ActorDecisionDescriptor,
    kind: ActorDecisionObservationKind,
}

actor_decision_sites! {
    ActorStateRead => ActorStateRead,
    PredecessorCandidateRead => PredecessorCandidateRead,
    ActorIdentityDecision => ActorIdentityDecision,
    SequenceRelationDecision => SequenceRelationDecision,
}

fn perform_actor_decision_operation<T, E>(
    site: ActorDecisionSite,
    charge: &mut impl FnMut(WorkCounter) -> Result<(), E>,
    observed: &mut impl FnMut(ActorDecisionObservation),
    perform: impl FnOnce() -> T,
) -> Result<T, MeteredActorStateError<E>> {
    let descriptor = site.descriptor();
    observed(ActorDecisionObservation {
        descriptor,
        kind: ActorDecisionObservationKind::ChargeAttempt,
    });
    charge(descriptor.counter).map_err(MeteredActorStateError::Work)?;
    let result = perform();
    observed(ActorDecisionObservation {
        descriptor,
        kind: ActorDecisionObservationKind::TargetCompleted,
    });
    Ok(result)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ActorIdentityRelation {
    NoPredecessor,
    Matches,
    InvalidPredecessor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SequenceRelation {
    ValidGenesis,
    ExpectedSuccessor,
    Rollback,
    GapOrMissingPredecessor,
    ArithmeticOverflow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CausalNextOperation {
    StoredCounterRead,
    ExpectedStartComparison,
    CheckedAdvance,
}

macro_rules! causal_next_sites {
    ($( $site:ident => $operation:ident ),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        enum CausalNextSite {
            $( $site, )+
        }

        impl CausalNextSite {
            const fn descriptor(self) -> CausalNextDescriptor {
                CausalNextDescriptor {
                    site: self,
                    site_id: match self {
                        $( Self::$site => stringify!($site), )+
                    },
                    phase: "causal_counter",
                    operation: match self {
                        $( Self::$site => CausalNextOperation::$operation, )+
                    },
                    counter: WorkCounter::GraphNode,
                    abstract_owner_class: "direct_operation",
                    applicability: "public_rust",
                }
            }
        }
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CausalNextDescriptor {
    site: CausalNextSite,
    site_id: &'static str,
    phase: &'static str,
    operation: CausalNextOperation,
    counter: WorkCounter,
    abstract_owner_class: &'static str,
    applicability: &'static str,
}

impl CausalNextDescriptor {
    const fn operation(self) -> CausalNextOperation {
        self.operation
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CausalNextObservationKind {
    ChargeAttempt,
    TargetCompleted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CausalNextObservation {
    descriptor: CausalNextDescriptor,
    kind: CausalNextObservationKind,
}

causal_next_sites! {
    StoredCounterRead => StoredCounterRead,
    ExpectedStartComparison => ExpectedStartComparison,
    CheckedAdvance => CheckedAdvance,
}

fn perform_causal_next_operation<T, E>(
    site: CausalNextSite,
    charge: &mut impl FnMut(WorkCounter) -> Result<(), E>,
    observed: &mut impl FnMut(CausalNextObservation),
    perform: impl FnOnce() -> T,
) -> Result<T, MeteredActorStateError<E>> {
    let descriptor = site.descriptor();
    observed(CausalNextObservation {
        descriptor,
        kind: CausalNextObservationKind::ChargeAttempt,
    });
    charge(descriptor.counter).map_err(MeteredActorStateError::Work)?;
    let result = perform();
    observed(CausalNextObservation {
        descriptor,
        kind: CausalNextObservationKind::TargetCompleted,
    });
    Ok(result)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FrontierComparisonOperation {
    CandidateKindComparison,
    CandidateCount,
    ProjectionCount,
    BaseCount,
    CandidatePull,
    CandidateOrderComparison,
    ProjectionPull,
    BasePull,
    BaseAcceptedLookup,
    ExpectedSourceComparison,
    FrontierEqualityComparison,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CandidateSemanticStage {
    ActorSequence,
    CausalCounter,
    EmptyFrontier,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProjectionPublicationOperation {
    CandidateDependency,
    DependedOn,
    DependantBucket,
    Dependant,
    ReadyCandidate,
    RemainingDependencies,
    Dependencies,
    FrontierHead,
    ActorState,
    WriterContribution,
    CausalCounter,
    ReadyDependant,
    Projection,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProjectionBuildOperation {
    SourceCountRead,
    ExpectedCountComparison,
    CanonicalSourcePull,
    CanonicalOrderCompare,
    MembershipLookup,
    CandidateLookup,
    CandidateIdentityComparison,
    DependencyCountRead,
    DependencyLookup,
    CandidateReadinessComparison,
    StateLookup,
    ReadinessTransition,
    CandidateKindComparison,
    CheckedArithmetic,
    RemainingStateWrite,
    MapInsertion,
    SetInsertion,
    CausalMaximumCompare,
    CompletionComparison,
    ResultPublication,
}

macro_rules! projection_build_sites {
    ($( $site:ident => ($operation:ident, $counter:ident) ),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        enum ProjectionBuildSite {
            $( $site, )+
        }

        impl ProjectionBuildSite {
            const fn operation(self) -> ProjectionBuildOperation {
                match self {
                    $( Self::$site => ProjectionBuildOperation::$operation, )+
                }
            }

            const fn counter(self) -> WorkCounter {
                match self {
                    $( Self::$site => WorkCounter::$counter, )+
                }
            }

            const fn id(self) -> &'static str {
                match self {
                    $( Self::$site => stringify!($site), )+
                }
            }

            const fn descriptor(self) -> ProjectionBuildDescriptor {
                ProjectionBuildDescriptor {
                    site: self,
                    site_id: self.id(),
                    phase: "construction",
                    operation: self.operation(),
                    counter: self.counter(),
                    abstract_owner_class: "source_operation",
                    applicability: "public_rust",
                }
            }
        }
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ProjectionBuildDescriptor {
    site: ProjectionBuildSite,
    site_id: &'static str,
    phase: &'static str,
    operation: ProjectionBuildOperation,
    counter: WorkCounter,
    abstract_owner_class: &'static str,
    applicability: &'static str,
}

impl ProjectionBuildDescriptor {
    const fn operation(self) -> ProjectionBuildOperation {
        self.operation
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProjectionBuildObservationKind {
    ChargeAttempt,
    TargetCompleted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ProjectionBuildObservation {
    descriptor: ProjectionBuildDescriptor,
    kind: ProjectionBuildObservationKind,
}

projection_build_sites! {
    MemberCountRead => (SourceCountRead, GraphNode),
    AcceptedCountMatches => (ExpectedCountComparison, GraphNode),
    NextMemberPull => (CanonicalSourcePull, GraphNode),
    MemberOrderCompare => (CanonicalOrderCompare, GraphNode),
    AcceptedMemberLookup => (MembershipLookup, GraphNode),
    CandidateLookup => (CandidateLookup, GraphNode),
    CandidateIdentityCompare => (CandidateIdentityComparison, GraphNode),
    DependencyCountRead => (DependencyCountRead, GraphNode),
    DependencyPull => (DependencyLookup, GraphEdge),
    DependencyOrderCompare => (CanonicalOrderCompare, GraphEdge),
    AcceptedDependencyLookup => (MembershipLookup, GraphEdge),
    CandidateDependencyInsert => (SetInsertion, GraphEdge),
    DependedOnInsert => (SetInsertion, GraphEdge),
    DependantBucketInsert => (MapInsertion, GraphEdge),
    DependantInsert => (SetInsertion, GraphEdge),
    CandidateReadyCompare => (CandidateReadinessComparison, GraphNode),
    InitialReadyInsert => (ReadinessTransition, GraphNode),
    RemainingDependenciesInsert => (MapInsertion, GraphNode),
    DependenciesInsert => (MapInsertion, GraphNode),
    ReadyNonemptyCompare => (ReadinessTransition, GraphNode),
    ReadyCandidatePull => (CanonicalSourcePull, GraphNode),
    ReadyCandidateLookup => (CandidateLookup, GraphNode),
    DependedOnLookup => (StateLookup, GraphNode),
    FrontierHeadInsert => (SetInsertion, GraphNode),
    ActorStateLookup => (StateLookup, GraphNode),
    ExpectedSequenceAdvance => (CheckedArithmetic, GraphNode),
    SequencePrecedesCompare => (CanonicalOrderCompare, GraphNode),
    SequenceMatchesCompare => (CanonicalOrderCompare, GraphNode),
    CausalNextLookup => (StateLookup, GraphNode),
    StartOperationCompare => (CanonicalOrderCompare, GraphNode),
    CandidateEmptyCompare => (CandidateKindComparison, GraphNode),
    CandidateCausalAdvance => (CheckedArithmetic, GraphNode),
    GlobalCausalMaximum => (CausalMaximumCompare, GraphNode),
    ActorStateInsert => (MapInsertion, GraphNode),
    WriterContributionInsert => (MapInsertion, GraphNode),
    CausalCounterInsert => (MapInsertion, GraphNode),
    ProcessedCountAdvance => (CheckedArithmetic, GraphNode),
    DependantsLookup => (StateLookup, GraphEdge),
    ChildCountRead => (StateLookup, GraphEdge),
    ChildPull => (DependencyLookup, GraphEdge),
    RemainingDependencyLookup => (StateLookup, GraphEdge),
    RemainingDependencyDecrement => (CheckedArithmetic, GraphEdge),
    RemainingDependencyWrite => (RemainingStateWrite, GraphEdge),
    PriorCausalLookup => (StateLookup, GraphEdge),
    PropagatedCausalMaximum => (CausalMaximumCompare, GraphEdge),
    ChildCausalInsert => (MapInsertion, GraphEdge),
    ChildReadyCompare => (ReadinessTransition, GraphEdge),
    ReadyDependantInsert => (ReadinessTransition, GraphNode),
    CompletionCompare => (CompletionComparison, GraphNode),
    ProjectionPublish => (ResultPublication, GraphNode),
}

fn perform_projection_build_operation<T, E>(
    site: ProjectionBuildSite,
    charge: &mut impl FnMut(WorkCounter) -> Result<(), E>,
    observed: &mut impl FnMut(ProjectionBuildObservation),
    perform: impl FnOnce() -> T,
) -> Result<T, MeteredActorStateError<E>> {
    let descriptor = site.descriptor();
    observed(ProjectionBuildObservation {
        descriptor,
        kind: ProjectionBuildObservationKind::ChargeAttempt,
    });
    charge(descriptor.counter).map_err(MeteredActorStateError::Work)?;
    let result = perform();
    observed(ProjectionBuildObservation {
        descriptor,
        kind: ProjectionBuildObservationKind::TargetCompleted,
    });
    Ok(result)
}

pub(crate) type AcceptedEpochStateParts = (
    BTreeSet<ChangeHash>,
    BTreeMap<ChangeHash, BTreeSet<ChangeHash>>,
    BTreeMap<ActorId, EpochActorState>,
    BTreeMap<ActorId, ChangeHash>,
);

impl TrustedEpochProjection<'_> {
    /// Applies the complete actor/counter/frontier decision in protocol order.
    ///
    /// A semantic failure stops before later decision families. Work failures
    /// retain the exact injected cause and likewise prevent later work.
    pub(crate) fn candidate_semantics_decision_metered<E>(
        &self,
        candidate: &ChangeCandidate,
        base_frontier: &BTreeSet<ChangeHash>,
        charge: impl FnMut(WorkCounter) -> Result<(), E>,
    ) -> Result<(), MeteredActorStateError<E>> {
        self.candidate_semantics_decision_metered_observed(candidate, base_frontier, charge, |_| {})
    }

    fn candidate_semantics_decision_metered_observed<E>(
        &self,
        candidate: &ChangeCandidate,
        base_frontier: &BTreeSet<ChangeHash>,
        mut charge: impl FnMut(WorkCounter) -> Result<(), E>,
        mut observed: impl FnMut(CandidateSemanticStage),
    ) -> Result<(), MeteredActorStateError<E>> {
        self.actor_sequence_decision_metered(candidate, &mut charge)?;
        observed(CandidateSemanticStage::ActorSequence);
        self.causal_next_decision_metered(candidate, &mut charge)?;
        observed(CandidateSemanticStage::CausalCounter);
        self.empty_frontier_decision_metered(candidate, base_frontier, charge)?;
        observed(CandidateSemanticStage::EmptyFrontier);
        Ok(())
    }

    /// Decides actor-sequence continuity from immutable projected state.
    ///
    /// The actor predecessor may be anywhere in the accepted closure. It is
    /// intentionally not required to be a direct dependency of `candidate`.
    pub(crate) fn actor_sequence_decision_metered<E>(
        &self,
        candidate: &ChangeCandidate,
        charge: impl FnMut(WorkCounter) -> Result<(), E>,
    ) -> Result<(), MeteredActorStateError<E>> {
        self.actor_sequence_decision_metered_observed(candidate, charge, |_| {})
    }

    fn actor_sequence_decision_metered_observed<E>(
        &self,
        candidate: &ChangeCandidate,
        mut charge: impl FnMut(WorkCounter) -> Result<(), E>,
        mut observed: impl FnMut(ActorDecisionObservation),
    ) -> Result<(), MeteredActorStateError<E>> {
        let actor_state = perform_actor_decision_operation(
            ActorDecisionSite::ActorStateRead,
            &mut charge,
            &mut observed,
            || self.actor_states.get(&candidate.actor).copied(),
        )?;

        let predecessor = if let Some(state) = actor_state {
            perform_actor_decision_operation(
                ActorDecisionSite::PredecessorCandidateRead,
                &mut charge,
                &mut observed,
                || self.branch_membership.get(&state.highest_change),
            )?
        } else {
            None
        };

        let actor_relation = perform_actor_decision_operation(
            ActorDecisionSite::ActorIdentityDecision,
            &mut charge,
            &mut observed,
            || match (actor_state, predecessor) {
                (None, None) => ActorIdentityRelation::NoPredecessor,
                (Some(_), Some(value)) if value.actor == candidate.actor => {
                    ActorIdentityRelation::Matches
                }
                _ => ActorIdentityRelation::InvalidPredecessor,
            },
        )?;
        if actor_relation == ActorIdentityRelation::InvalidPredecessor {
            return Err(MeteredActorStateError::State(
                ActorStateError::MissingPredecessor,
            ));
        }

        let sequence_relation = perform_actor_decision_operation(
            ActorDecisionSite::SequenceRelationDecision,
            &mut charge,
            &mut observed,
            || match (actor_relation, actor_state) {
                (ActorIdentityRelation::NoPredecessor, None) if candidate.sequence == 1 => {
                    SequenceRelation::ValidGenesis
                }
                (ActorIdentityRelation::NoPredecessor, None) => {
                    SequenceRelation::GapOrMissingPredecessor
                }
                (ActorIdentityRelation::Matches, Some(state)) => {
                    match state.last_sequence.checked_add(1) {
                        Some(expected) if candidate.sequence < expected => {
                            SequenceRelation::Rollback
                        }
                        Some(expected) if candidate.sequence == expected => {
                            SequenceRelation::ExpectedSuccessor
                        }
                        Some(_) => SequenceRelation::GapOrMissingPredecessor,
                        None => SequenceRelation::ArithmeticOverflow,
                    }
                }
                _ => SequenceRelation::GapOrMissingPredecessor,
            },
        )?;

        match sequence_relation {
            SequenceRelation::ValidGenesis | SequenceRelation::ExpectedSuccessor => Ok(()),
            SequenceRelation::Rollback => Err(MeteredActorStateError::State(
                ActorStateError::SequenceRollback,
            )),
            SequenceRelation::GapOrMissingPredecessor => Err(MeteredActorStateError::State(
                ActorStateError::MissingPredecessor,
            )),
            SequenceRelation::ArithmeticOverflow => {
                Err(MeteredActorStateError::State(ActorStateError::SequenceGap))
            }
        }
    }

    /// Validates and advances one candidate's causal operation interval from
    /// the immutable closure-wide counter retained by this projection.
    pub(crate) fn causal_next_decision_metered<E>(
        &self,
        candidate: &ChangeCandidate,
        charge: impl FnMut(WorkCounter) -> Result<(), E>,
    ) -> Result<u64, MeteredActorStateError<E>> {
        self.causal_next_decision_metered_observed(candidate, charge, |_| {})
    }

    fn causal_next_decision_metered_observed<E>(
        &self,
        candidate: &ChangeCandidate,
        mut charge: impl FnMut(WorkCounter) -> Result<(), E>,
        mut observed: impl FnMut(CausalNextObservation),
    ) -> Result<u64, MeteredActorStateError<E>> {
        let causal_next_op = perform_causal_next_operation(
            CausalNextSite::StoredCounterRead,
            &mut charge,
            &mut observed,
            || self.causal_next_op,
        )?;

        let start_matches = perform_causal_next_operation(
            CausalNextSite::ExpectedStartComparison,
            &mut charge,
            &mut observed,
            || candidate.start_op == causal_next_op,
        )?;
        if !start_matches {
            return Err(MeteredActorStateError::State(
                ActorStateError::OperationCounter,
            ));
        }

        let advanced = perform_causal_next_operation(
            CausalNextSite::CheckedAdvance,
            &mut charge,
            &mut observed,
            || causal_next_op.checked_add(candidate.operation_count),
        )?;
        advanced.ok_or(MeteredActorStateError::State(
            ActorStateError::OperationCounter,
        ))
    }

    pub(crate) fn dependencies(
        &self,
        change: &ChangeHash,
    ) -> impl Iterator<Item = ChangeHash> + '_ {
        self.dependencies
            .get(change)
            .into_iter()
            .flat_map(|dependencies| dependencies.iter().copied())
    }

    pub(crate) fn dependency_count(&self, change: &ChangeHash) -> usize {
        self.dependencies.get(change).map_or(0, BTreeSet::len)
    }

    pub(crate) fn frontier_heads(&self) -> impl ExactSizeIterator<Item = ChangeHash> + '_ {
        self.frontier_heads.iter().copied()
    }

    pub(crate) fn writer_contributions(
        &self,
    ) -> impl ExactSizeIterator<Item = (ActorId, ChangeHash)> + '_ {
        self.writer_contributions
            .iter()
            .map(|(actor, hash)| (*actor, *hash))
    }

    pub(crate) fn empty_frontier_decision_metered<E>(
        &self,
        candidate: &ChangeCandidate,
        base_frontier: &BTreeSet<ChangeHash>,
        charge: impl FnMut(WorkCounter) -> Result<(), E>,
    ) -> Result<(), MeteredActorStateError<E>> {
        self.empty_frontier_decision_metered_observed(candidate, base_frontier, charge, |_| {})
    }

    fn empty_frontier_decision_metered_observed<E>(
        &self,
        candidate: &ChangeCandidate,
        base_frontier: &BTreeSet<ChangeHash>,
        mut charge: impl FnMut(WorkCounter) -> Result<(), E>,
        mut observed: impl FnMut(FrontierComparisonOperation),
    ) -> Result<(), MeteredActorStateError<E>> {
        let nonempty = metered_frontier_operation(
            &mut charge,
            &mut observed,
            WorkCounter::GraphNode,
            FrontierComparisonOperation::CandidateKindComparison,
            || candidate.operation_count != 0,
        )?;
        if nonempty {
            return Ok(());
        }

        let dependency_count = metered_frontier_operation(
            &mut charge,
            &mut observed,
            WorkCounter::GraphNode,
            FrontierComparisonOperation::CandidateCount,
            || candidate.dependencies.len(),
        )?;
        let projection_count = metered_frontier_operation(
            &mut charge,
            &mut observed,
            WorkCounter::GraphNode,
            FrontierComparisonOperation::ProjectionCount,
            || self.frontier_heads.len(),
        )?;
        let base_count = metered_frontier_operation(
            &mut charge,
            &mut observed,
            WorkCounter::GraphNode,
            FrontierComparisonOperation::BaseCount,
            || base_frontier.len(),
        )?;

        let mut dependency_index = 0_usize;
        let mut dependency = None;
        let mut previous_dependency = None;
        let mut projection_remaining = projection_count;
        let mut projection_iter = self.frontier_heads.iter();
        let mut projection = None;
        let mut base_remaining = base_count;
        let mut base_iter = base_frontier.iter();
        let mut base = None;

        loop {
            if dependency.is_none() && dependency_index < dependency_count {
                let pulled = metered_frontier_operation(
                    &mut charge,
                    &mut observed,
                    WorkCounter::GraphEdge,
                    FrontierComparisonOperation::CandidatePull,
                    || candidate.dependencies.get(dependency_index).copied(),
                )?
                .ok_or(MeteredActorStateError::State(
                    ActorStateError::DependencyFrontier,
                ))?;
                dependency_index = dependency_index.saturating_add(1);
                if let Some(previous) = previous_dependency {
                    let ordered = metered_frontier_operation(
                        &mut charge,
                        &mut observed,
                        WorkCounter::GraphEdge,
                        FrontierComparisonOperation::CandidateOrderComparison,
                        || previous < pulled,
                    )?;
                    if !ordered {
                        return Err(MeteredActorStateError::State(
                            ActorStateError::DependencyFrontier,
                        ));
                    }
                }
                previous_dependency = Some(pulled);
                dependency = Some(pulled);
            }

            if projection.is_none() && projection_remaining > 0 {
                projection = Some(
                    metered_frontier_operation(
                        &mut charge,
                        &mut observed,
                        WorkCounter::GraphNode,
                        FrontierComparisonOperation::ProjectionPull,
                        || projection_iter.next().copied(),
                    )?
                    .ok_or(MeteredActorStateError::State(
                        ActorStateError::DependencyFrontier,
                    ))?,
                );
                projection_remaining = projection_remaining.saturating_sub(1);
            }

            while base.is_none() && base_remaining > 0 {
                let pulled = metered_frontier_operation(
                    &mut charge,
                    &mut observed,
                    WorkCounter::GraphNode,
                    FrontierComparisonOperation::BasePull,
                    || base_iter.next().copied(),
                )?
                .ok_or(MeteredActorStateError::State(
                    ActorStateError::DependencyFrontier,
                ))?;
                base_remaining = base_remaining.saturating_sub(1);
                let accepted = metered_frontier_operation(
                    &mut charge,
                    &mut observed,
                    WorkCounter::GraphNode,
                    FrontierComparisonOperation::BaseAcceptedLookup,
                    || self.accepted_closure.contains(&pulled),
                )?;
                if !accepted {
                    base = Some(pulled);
                }
            }

            let (expected, consume_projection, consume_base) = match (projection, base) {
                (Some(projected), Some(base_head)) => {
                    let ordering = metered_frontier_operation(
                        &mut charge,
                        &mut observed,
                        WorkCounter::GraphNode,
                        FrontierComparisonOperation::ExpectedSourceComparison,
                        || projected.cmp(&base_head),
                    )?;
                    match ordering {
                        core::cmp::Ordering::Less => (Some(projected), true, false),
                        core::cmp::Ordering::Greater => (Some(base_head), false, true),
                        core::cmp::Ordering::Equal => {
                            return Err(MeteredActorStateError::State(
                                ActorStateError::DependencyFrontier,
                            ));
                        }
                    }
                }
                (Some(projected), None) => (Some(projected), true, false),
                (None, Some(base_head)) => (Some(base_head), false, true),
                (None, None) => (None, false, false),
            };

            match (dependency, expected) {
                (None, None) => return Ok(()),
                (Some(actual), Some(expected)) => {
                    let equal = metered_frontier_operation(
                        &mut charge,
                        &mut observed,
                        WorkCounter::GraphEdge,
                        FrontierComparisonOperation::FrontierEqualityComparison,
                        || actual == expected,
                    )?;
                    if !equal {
                        return Err(MeteredActorStateError::State(
                            ActorStateError::DependencyFrontier,
                        ));
                    }
                    dependency = None;
                    if consume_projection {
                        projection = None;
                    }
                    if consume_base {
                        base = None;
                    }
                }
                _ => {
                    return Err(MeteredActorStateError::State(
                        ActorStateError::DependencyFrontier,
                    ));
                }
            }
        }
    }

    pub(crate) fn into_accepted_state_parts(self) -> AcceptedEpochStateParts {
        (
            self.frontier_heads,
            self.dependencies,
            self.actor_states,
            self.writer_contributions,
        )
    }
}

fn metered_frontier_operation<E, T>(
    charge: &mut impl FnMut(WorkCounter) -> Result<(), E>,
    observed: &mut impl FnMut(FrontierComparisonOperation),
    counter: WorkCounter,
    operation: FrontierComparisonOperation,
    target: impl FnOnce() -> T,
) -> Result<T, MeteredActorStateError<E>> {
    charge(counter).map_err(MeteredActorStateError::Work)?;
    let result = target();
    observed(operation);
    Ok(result)
}

#[cfg(test)]
fn reference_apply_empty_counter(
    states: &mut BTreeMap<ActorId, EpochActorState>,
    candidate: &ChangeCandidate,
    current_heads: &std::collections::BTreeSet<ChangeHash>,
) -> Result<(), ActorStateError> {
    if candidate.operation_count != 0 {
        return Err(ActorStateError::NonEmptyChange);
    }
    if candidate
        .dependencies
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>()
        != *current_heads
    {
        return Err(ActorStateError::DependencyFrontier);
    }
    let last_sequence = states
        .get(&candidate.actor)
        .map_or(0, |state| state.last_sequence);
    let next_op = reference_causal_next_op(states);
    if candidate.sequence
        != last_sequence
            .checked_add(1)
            .ok_or(ActorStateError::SequenceGap)?
    {
        return Err(ActorStateError::SequenceGap);
    }
    if candidate.start_op != next_op {
        return Err(ActorStateError::OperationCounter);
    }
    states.insert(
        candidate.actor,
        EpochActorState {
            last_sequence: candidate.sequence,
            next_op,
            highest_change: candidate.change_hash,
        },
    );
    Ok(())
}

#[cfg(test)]
fn reference_apply_nonempty_counter(
    states: &mut BTreeMap<ActorId, EpochActorState>,
    candidate: &ChangeCandidate,
) -> Result<(), ActorStateError> {
    if candidate.operation_count == 0 {
        return Err(ActorStateError::EmptyChange);
    }
    let last_sequence = states
        .get(&candidate.actor)
        .map_or(0, |state| state.last_sequence);
    let next_op = reference_causal_next_op(states);
    if candidate.sequence
        != last_sequence
            .checked_add(1)
            .ok_or(ActorStateError::SequenceGap)?
    {
        return Err(ActorStateError::SequenceGap);
    }
    if candidate.start_op != next_op {
        return Err(ActorStateError::OperationCounter);
    }
    let next_op = next_op
        .checked_add(candidate.operation_count)
        .ok_or(ActorStateError::OperationCounter)?;
    states.insert(
        candidate.actor,
        EpochActorState {
            last_sequence: candidate.sequence,
            next_op,
            highest_change: candidate.change_hash,
        },
    );
    Ok(())
}

#[cfg(test)]
fn reference_causal_next_op(states: &BTreeMap<ActorId, EpochActorState>) -> u64 {
    // Automerge operation counters are causal Lamport counters. Actor sequence
    // remains actor-local, while a new change starts after the greatest
    // operation visible in its exact dependency closure.
    states
        .values()
        .map(|state| state.next_op)
        .max()
        .unwrap_or(1)
}

#[cfg(test)]
pub(crate) fn initialize_actor_states(
    accepted_base: impl IntoIterator<Item = ChangeCandidate>,
) -> Result<BTreeMap<ActorId, EpochActorState>, ActorStateError> {
    let changes = accepted_base
        .into_iter()
        .map(|candidate| (candidate.change_hash, candidate))
        .collect::<BTreeMap<_, _>>();
    let mut remaining_dependencies = BTreeMap::new();
    let mut dependants = BTreeMap::<ChangeHash, BTreeSet<ChangeHash>>::new();
    let mut ready = BTreeSet::new();
    for (hash, candidate) in &changes {
        let dependencies = candidate
            .dependencies
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        if !dependencies
            .iter()
            .all(|dependency| changes.contains_key(dependency))
        {
            return Err(ActorStateError::MissingDependency);
        }
        if dependencies.is_empty() {
            ready.insert(*hash);
        }
        remaining_dependencies.insert(*hash, dependencies.len());
        for dependency in dependencies {
            dependants.entry(dependency).or_default().insert(*hash);
        }
    }
    let mut states = BTreeMap::<ActorId, EpochActorState>::new();
    let mut causal_next_by_change = BTreeMap::<ChangeHash, u64>::new();
    let mut processed = 0usize;
    while let Some(hash) = ready.pop_first() {
        let candidate = &changes[&hash];
        let expected_sequence = match states.get(&candidate.actor) {
            Some(state) => state
                .last_sequence
                .checked_add(1)
                .ok_or(ActorStateError::SequenceGap)?,
            None => 1,
        };
        if candidate.sequence < expected_sequence {
            return Err(ActorStateError::Equivocation);
        }
        if candidate.sequence != expected_sequence {
            return Err(ActorStateError::SequenceGap);
        }
        let next_op = candidate
            .dependencies
            .iter()
            .filter_map(|dependency| causal_next_by_change.get(dependency).copied())
            .max()
            .unwrap_or(1);
        let advanced = if candidate.operation_count == 0 {
            if candidate.start_op != next_op {
                return Err(ActorStateError::OperationCounter);
            }
            next_op
        } else {
            if candidate.start_op != next_op {
                return Err(ActorStateError::OperationCounter);
            }
            next_op
                .checked_add(candidate.operation_count)
                .ok_or(ActorStateError::OperationCounter)?
        };
        states.insert(
            candidate.actor,
            EpochActorState {
                last_sequence: candidate.sequence,
                next_op: advanced,
                highest_change: candidate.change_hash,
            },
        );
        causal_next_by_change.insert(candidate.change_hash, advanced);
        processed = processed
            .checked_add(1)
            .ok_or(ActorStateError::DependencyCycle)?;
        if let Some(children) = dependants.get(&hash) {
            for child in children {
                let Some(remaining) = remaining_dependencies.get_mut(child) else {
                    return Err(ActorStateError::MissingDependency);
                };
                *remaining = remaining
                    .checked_sub(1)
                    .ok_or(ActorStateError::DependencyCycle)?;
                if *remaining == 0 {
                    ready.insert(*child);
                }
            }
        }
    }
    if processed != changes.len() {
        return Err(ActorStateError::DependencyCycle);
    }
    Ok(states)
}

pub(crate) fn initialize_actor_states_metered<'a, E>(
    accepted_closure: &'a BTreeSet<ChangeHash>,
    changes: &'a BTreeMap<ChangeHash, ChangeCandidate>,
    charge: impl FnMut(WorkCounter) -> Result<(), E>,
) -> Result<TrustedEpochProjection<'a>, MeteredActorStateError<E>> {
    let mut source = CanonicalEpochProjectionSource::new(accepted_closure, changes);
    build_trusted_epoch_projection(accepted_closure, changes, &mut source, charge)
}

fn build_trusted_epoch_projection<'a, E>(
    accepted_closure: &'a BTreeSet<ChangeHash>,
    changes: &'a BTreeMap<ChangeHash, ChangeCandidate>,
    source: &mut impl EpochProjectionSource<'a>,
    charge: impl FnMut(WorkCounter) -> Result<(), E>,
) -> Result<TrustedEpochProjection<'a>, MeteredActorStateError<E>> {
    build_trusted_epoch_projection_observed(
        accepted_closure,
        changes,
        source,
        charge,
        |_| {},
        |_| {},
    )
}

fn build_trusted_epoch_projection_observed<'a, E>(
    accepted_closure: &'a BTreeSet<ChangeHash>,
    changes: &'a BTreeMap<ChangeHash, ChangeCandidate>,
    source: &mut impl EpochProjectionSource<'a>,
    mut charge: impl FnMut(WorkCounter) -> Result<(), E>,
    mut built: impl FnMut(ProjectionBuildObservation),
    mut published: impl FnMut(ProjectionPublicationOperation),
) -> Result<TrustedEpochProjection<'a>, MeteredActorStateError<E>> {
    let member_count = perform_projection_build_operation(
        ProjectionBuildSite::MemberCountRead,
        &mut charge,
        &mut built,
        || source.member_count(),
    )?;
    let input_is_canonical = perform_projection_build_operation(
        ProjectionBuildSite::AcceptedCountMatches,
        &mut charge,
        &mut built,
        || member_count == accepted_closure.len(),
    )?;
    if !input_is_canonical {
        return Err(MeteredActorStateError::State(
            ActorStateError::NoncanonicalInput,
        ));
    }
    let mut dependencies = BTreeMap::new();
    let mut depended_on = BTreeSet::new();
    let mut remaining_dependencies = BTreeMap::new();
    let mut dependants = BTreeMap::<ChangeHash, BTreeSet<ChangeHash>>::new();
    let mut ready = BTreeSet::new();
    let mut previous_hash = None;
    for _ in 0..member_count {
        let Some(hash) = perform_projection_build_operation(
            ProjectionBuildSite::NextMemberPull,
            &mut charge,
            &mut built,
            || source.next_member(),
        )?
        else {
            return Err(MeteredActorStateError::State(
                ActorStateError::NoncanonicalInput,
            ));
        };
        if let Some(previous) = previous_hash {
            let ordered = perform_projection_build_operation(
                ProjectionBuildSite::MemberOrderCompare,
                &mut charge,
                &mut built,
                || previous < hash,
            )?;
            if !ordered {
                return Err(MeteredActorStateError::State(
                    ActorStateError::NoncanonicalInput,
                ));
            }
        }
        previous_hash = Some(hash);
        let accepted_member = perform_projection_build_operation(
            ProjectionBuildSite::AcceptedMemberLookup,
            &mut charge,
            &mut built,
            || source.accepted_member(&hash),
        )?;
        if !accepted_member {
            return Err(MeteredActorStateError::State(
                ActorStateError::NoncanonicalInput,
            ));
        }
        let Some(candidate) = perform_projection_build_operation(
            ProjectionBuildSite::CandidateLookup,
            &mut charge,
            &mut built,
            || source.candidate(&hash),
        )?
        else {
            return Err(MeteredActorStateError::State(
                ActorStateError::MissingDependency,
            ));
        };
        let candidate_identity_matches = perform_projection_build_operation(
            ProjectionBuildSite::CandidateIdentityCompare,
            &mut charge,
            &mut built,
            || candidate.change_hash == hash,
        )?;
        if !candidate_identity_matches {
            return Err(MeteredActorStateError::State(
                ActorStateError::NoncanonicalInput,
            ));
        }
        let mut candidate_dependencies = BTreeSet::new();
        let dependency_count = perform_projection_build_operation(
            ProjectionBuildSite::DependencyCountRead,
            &mut charge,
            &mut built,
            || source.dependency_count(candidate),
        )?;
        let mut previous_dependency = None;
        for index in 0..dependency_count {
            let Some(dependency) = perform_projection_build_operation(
                ProjectionBuildSite::DependencyPull,
                &mut charge,
                &mut built,
                || source.dependency(candidate, index),
            )?
            else {
                return Err(MeteredActorStateError::State(
                    ActorStateError::NoncanonicalInput,
                ));
            };
            if let Some(previous) = previous_dependency {
                let ordered = perform_projection_build_operation(
                    ProjectionBuildSite::DependencyOrderCompare,
                    &mut charge,
                    &mut built,
                    || previous < dependency,
                )?;
                if !ordered {
                    return Err(MeteredActorStateError::State(
                        ActorStateError::NoncanonicalInput,
                    ));
                }
            }
            previous_dependency = Some(dependency);
            let accepted_dependency = perform_projection_build_operation(
                ProjectionBuildSite::AcceptedDependencyLookup,
                &mut charge,
                &mut built,
                || source.accepted_member(&dependency),
            )?;
            if !accepted_dependency {
                return Err(MeteredActorStateError::State(
                    ActorStateError::MissingDependency,
                ));
            }
            perform_projection_build_operation(
                ProjectionBuildSite::CandidateDependencyInsert,
                &mut charge,
                &mut built,
                || candidate_dependencies.insert(dependency),
            )?;
            published(ProjectionPublicationOperation::CandidateDependency);
            perform_projection_build_operation(
                ProjectionBuildSite::DependedOnInsert,
                &mut charge,
                &mut built,
                || depended_on.insert(dependency),
            )?;
            published(ProjectionPublicationOperation::DependedOn);
            let dependant_bucket = perform_projection_build_operation(
                ProjectionBuildSite::DependantBucketInsert,
                &mut charge,
                &mut built,
                || dependants.entry(dependency).or_default(),
            )?;
            published(ProjectionPublicationOperation::DependantBucket);
            perform_projection_build_operation(
                ProjectionBuildSite::DependantInsert,
                &mut charge,
                &mut built,
                || dependant_bucket.insert(hash),
            )?;
            published(ProjectionPublicationOperation::Dependant);
        }
        let candidate_is_ready = perform_projection_build_operation(
            ProjectionBuildSite::CandidateReadyCompare,
            &mut charge,
            &mut built,
            || candidate_dependencies.is_empty(),
        )?;
        if candidate_is_ready {
            perform_projection_build_operation(
                ProjectionBuildSite::InitialReadyInsert,
                &mut charge,
                &mut built,
                || ready.insert(hash),
            )?;
            published(ProjectionPublicationOperation::ReadyCandidate);
        }
        perform_projection_build_operation(
            ProjectionBuildSite::RemainingDependenciesInsert,
            &mut charge,
            &mut built,
            || remaining_dependencies.insert(hash, dependency_count),
        )?;
        published(ProjectionPublicationOperation::RemainingDependencies);
        perform_projection_build_operation(
            ProjectionBuildSite::DependenciesInsert,
            &mut charge,
            &mut built,
            || dependencies.insert(hash, candidate_dependencies),
        )?;
        published(ProjectionPublicationOperation::Dependencies);
    }

    let mut states = BTreeMap::<ActorId, EpochActorState>::new();
    let mut frontier_heads = BTreeSet::new();
    let mut writer_contributions = BTreeMap::new();
    let mut causal_next_by_change = BTreeMap::<ChangeHash, u64>::new();
    let mut causal_next_op = 1_u64;
    let mut processed = 0usize;
    loop {
        let has_ready = perform_projection_build_operation(
            ProjectionBuildSite::ReadyNonemptyCompare,
            &mut charge,
            &mut built,
            || !ready.is_empty(),
        )?;
        if !has_ready {
            break;
        }
        let Some(hash) = perform_projection_build_operation(
            ProjectionBuildSite::ReadyCandidatePull,
            &mut charge,
            &mut built,
            || ready.pop_first(),
        )?
        else {
            return Err(MeteredActorStateError::State(
                ActorStateError::DependencyCycle,
            ));
        };
        let Some(candidate) = perform_projection_build_operation(
            ProjectionBuildSite::ReadyCandidateLookup,
            &mut charge,
            &mut built,
            || source.candidate(&hash),
        )?
        else {
            return Err(MeteredActorStateError::State(
                ActorStateError::MissingDependency,
            ));
        };
        let is_depended_on = perform_projection_build_operation(
            ProjectionBuildSite::DependedOnLookup,
            &mut charge,
            &mut built,
            || depended_on.contains(&hash),
        )?;
        if !is_depended_on {
            perform_projection_build_operation(
                ProjectionBuildSite::FrontierHeadInsert,
                &mut charge,
                &mut built,
                || frontier_heads.insert(hash),
            )?;
            published(ProjectionPublicationOperation::FrontierHead);
        }
        let previous_state = perform_projection_build_operation(
            ProjectionBuildSite::ActorStateLookup,
            &mut charge,
            &mut built,
            || states.get(&candidate.actor).copied(),
        )?;
        let expected_sequence = match previous_state {
            Some(state) => perform_projection_build_operation(
                ProjectionBuildSite::ExpectedSequenceAdvance,
                &mut charge,
                &mut built,
                || state.last_sequence.checked_add(1),
            )?
            .ok_or(MeteredActorStateError::State(ActorStateError::SequenceGap))?,
            None => 1,
        };
        let sequence_precedes = perform_projection_build_operation(
            ProjectionBuildSite::SequencePrecedesCompare,
            &mut charge,
            &mut built,
            || candidate.sequence < expected_sequence,
        )?;
        if sequence_precedes {
            return Err(MeteredActorStateError::State(ActorStateError::Equivocation));
        }
        let sequence_matches = perform_projection_build_operation(
            ProjectionBuildSite::SequenceMatchesCompare,
            &mut charge,
            &mut built,
            || candidate.sequence == expected_sequence,
        )?;
        if !sequence_matches {
            return Err(MeteredActorStateError::State(ActorStateError::SequenceGap));
        }
        let next_op = perform_projection_build_operation(
            ProjectionBuildSite::CausalNextLookup,
            &mut charge,
            &mut built,
            || causal_next_by_change.get(&hash).copied().unwrap_or(1),
        )?;
        let start_matches = perform_projection_build_operation(
            ProjectionBuildSite::StartOperationCompare,
            &mut charge,
            &mut built,
            || candidate.start_op == next_op,
        )?;
        if !start_matches {
            return Err(MeteredActorStateError::State(
                ActorStateError::OperationCounter,
            ));
        }
        let candidate_is_empty = perform_projection_build_operation(
            ProjectionBuildSite::CandidateEmptyCompare,
            &mut charge,
            &mut built,
            || candidate.operation_count == 0,
        )?;
        let advanced = if candidate_is_empty {
            next_op
        } else {
            perform_projection_build_operation(
                ProjectionBuildSite::CandidateCausalAdvance,
                &mut charge,
                &mut built,
                || next_op.checked_add(candidate.operation_count),
            )?
            .ok_or(MeteredActorStateError::State(
                ActorStateError::OperationCounter,
            ))?
        };
        causal_next_op = perform_projection_build_operation(
            ProjectionBuildSite::GlobalCausalMaximum,
            &mut charge,
            &mut built,
            || causal_next_op.max(advanced),
        )?;
        perform_projection_build_operation(
            ProjectionBuildSite::ActorStateInsert,
            &mut charge,
            &mut built,
            || {
                states.insert(
                    candidate.actor,
                    EpochActorState {
                        last_sequence: candidate.sequence,
                        next_op: advanced,
                        highest_change: candidate.change_hash,
                    },
                )
            },
        )?;
        published(ProjectionPublicationOperation::ActorState);
        perform_projection_build_operation(
            ProjectionBuildSite::WriterContributionInsert,
            &mut charge,
            &mut built,
            || writer_contributions.insert(candidate.actor, candidate.change_hash),
        )?;
        published(ProjectionPublicationOperation::WriterContribution);
        perform_projection_build_operation(
            ProjectionBuildSite::CausalCounterInsert,
            &mut charge,
            &mut built,
            || causal_next_by_change.insert(candidate.change_hash, advanced),
        )?;
        published(ProjectionPublicationOperation::CausalCounter);
        processed = perform_projection_build_operation(
            ProjectionBuildSite::ProcessedCountAdvance,
            &mut charge,
            &mut built,
            || processed.checked_add(1),
        )?
        .ok_or(MeteredActorStateError::State(
            ActorStateError::DependencyCycle,
        ))?;
        let children = perform_projection_build_operation(
            ProjectionBuildSite::DependantsLookup,
            &mut charge,
            &mut built,
            || dependants.get(&hash),
        )?;
        if let Some(children) = children {
            let child_count = perform_projection_build_operation(
                ProjectionBuildSite::ChildCountRead,
                &mut charge,
                &mut built,
                || children.len(),
            )?;
            let mut child_iter = children.iter();
            for _ in 0..child_count {
                let Some(child) = perform_projection_build_operation(
                    ProjectionBuildSite::ChildPull,
                    &mut charge,
                    &mut built,
                    || child_iter.next().copied(),
                )?
                else {
                    return Err(MeteredActorStateError::State(
                        ActorStateError::DependencyCycle,
                    ));
                };
                let Some(remaining) = perform_projection_build_operation(
                    ProjectionBuildSite::RemainingDependencyLookup,
                    &mut charge,
                    &mut built,
                    || remaining_dependencies.get_mut(&child),
                )?
                else {
                    return Err(MeteredActorStateError::State(
                        ActorStateError::MissingDependency,
                    ));
                };
                let updated_remaining = perform_projection_build_operation(
                    ProjectionBuildSite::RemainingDependencyDecrement,
                    &mut charge,
                    &mut built,
                    || remaining.checked_sub(1),
                )?
                .ok_or(MeteredActorStateError::State(
                    ActorStateError::DependencyCycle,
                ))?;
                perform_projection_build_operation(
                    ProjectionBuildSite::RemainingDependencyWrite,
                    &mut charge,
                    &mut built,
                    || *remaining = updated_remaining,
                )?;
                let prior_causal = perform_projection_build_operation(
                    ProjectionBuildSite::PriorCausalLookup,
                    &mut charge,
                    &mut built,
                    || causal_next_by_change.get(&child).copied(),
                )?;
                let propagated = if let Some(prior) = prior_causal {
                    perform_projection_build_operation(
                        ProjectionBuildSite::PropagatedCausalMaximum,
                        &mut charge,
                        &mut built,
                        || prior.max(advanced),
                    )?
                } else {
                    advanced
                };
                perform_projection_build_operation(
                    ProjectionBuildSite::ChildCausalInsert,
                    &mut charge,
                    &mut built,
                    || causal_next_by_change.insert(child, propagated),
                )?;
                published(ProjectionPublicationOperation::CausalCounter);
                let became_ready = perform_projection_build_operation(
                    ProjectionBuildSite::ChildReadyCompare,
                    &mut charge,
                    &mut built,
                    || updated_remaining == 0,
                )?;
                if became_ready {
                    perform_projection_build_operation(
                        ProjectionBuildSite::ReadyDependantInsert,
                        &mut charge,
                        &mut built,
                        || ready.insert(child),
                    )?;
                    published(ProjectionPublicationOperation::ReadyDependant);
                }
            }
        }
    }
    let is_complete = perform_projection_build_operation(
        ProjectionBuildSite::CompletionCompare,
        &mut charge,
        &mut built,
        || processed == member_count,
    )?;
    if !is_complete {
        return Err(MeteredActorStateError::State(
            ActorStateError::DependencyCycle,
        ));
    }
    let projection = perform_projection_build_operation(
        ProjectionBuildSite::ProjectionPublish,
        &mut charge,
        &mut built,
        || TrustedEpochProjection {
            branch_membership: changes,
            accepted_closure,
            dependencies,
            frontier_heads,
            actor_states: states,
            writer_contributions,
            causal_next_op,
        },
    )?;
    published(ProjectionPublicationOperation::Projection);
    Ok(projection)
}

#[cfg(test)]
pub(crate) mod tests {
    use std::cell::{Cell, RefCell};
    use std::collections::BTreeSet;
    use std::rc::Rc;

    use super::{
        ActorDecisionDescriptor, ActorDecisionObservationKind, ActorDecisionOperation,
        ActorDecisionSite, ActorStateError, CandidateSemanticStage, CanonicalEpochProjectionSource,
        CausalNextDescriptor, CausalNextObservationKind, CausalNextOperation, CausalNextSite,
        EpochActorState, EpochProjectionSource, FrontierComparisonOperation,
        MeteredActorStateError, ProjectionBuildDescriptor, ProjectionBuildObservation,
        ProjectionBuildObservationKind, ProjectionBuildOperation, ProjectionBuildSite,
        ProjectionPublicationOperation, TrustedEpochProjection, build_trusted_epoch_projection,
        build_trusted_epoch_projection_observed, initialize_actor_states,
        initialize_actor_states_metered, perform_projection_build_operation,
        reference_apply_empty_counter, reference_apply_nonempty_counter,
    };
    use crate::graph::change_candidate::ChangeCandidate;
    use crate::{ActorId, ChangeHash, Completion, DevicePublicKey, EventId, WorkCounter};
    use std::collections::BTreeMap;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct TrustedEpochView {
        branch_member: bool,
        accepted_member: bool,
        predecessor: Option<ChangeHash>,
        predecessor_is_direct_dependency: bool,
        actor_identity_matches: bool,
        expected_sequence: u64,
        sequence_matches: bool,
        causal_next_op: u64,
        expected_next_matches: bool,
    }

    impl TrustedEpochView {
        const fn is_branch_member(self) -> bool {
            self.branch_member
        }

        const fn is_accepted_member(self) -> bool {
            self.accepted_member
        }

        const fn predecessor(self) -> Option<ChangeHash> {
            self.predecessor
        }

        const fn predecessor_is_direct_dependency(self) -> bool {
            self.predecessor_is_direct_dependency
        }

        const fn actor_identity_matches(self) -> bool {
            self.actor_identity_matches
        }

        const fn expected_sequence(self) -> u64 {
            self.expected_sequence
        }

        const fn sequence_matches(self) -> bool {
            self.sequence_matches
        }

        const fn causal_next_op(self) -> u64 {
            self.causal_next_op
        }

        const fn expected_next_matches(self) -> bool {
            self.expected_next_matches
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum ProjectionLookupOperation {
        BranchMembership,
        AcceptedMembership,
        ActorState,
        DirectDependency,
        PredecessorCandidate,
        ActorIdentityComparison,
        ExpectedSequence,
        SequenceComparison,
        ExpectedNextComparison,
    }

    impl TrustedEpochProjection<'_> {
        fn candidate_metered<E>(
            &self,
            candidate: &ChangeCandidate,
            charge: impl FnMut(WorkCounter) -> Result<(), E>,
        ) -> Result<TrustedEpochView, MeteredActorStateError<E>> {
            self.candidate_metered_observed(candidate, charge, |_| {})
        }

        fn candidate_metered_observed<E>(
            &self,
            candidate: &ChangeCandidate,
            mut charge: impl FnMut(WorkCounter) -> Result<(), E>,
            mut observed: impl FnMut(ProjectionLookupOperation),
        ) -> Result<TrustedEpochView, MeteredActorStateError<E>> {
            charge(WorkCounter::GraphNode).map_err(MeteredActorStateError::Work)?;
            let branch_member = self.branch_membership.contains_key(&candidate.change_hash);
            observed(ProjectionLookupOperation::BranchMembership);

            charge(WorkCounter::GraphNode).map_err(MeteredActorStateError::Work)?;
            let accepted_member = self.accepted_closure.contains(&candidate.change_hash);
            observed(ProjectionLookupOperation::AcceptedMembership);

            charge(WorkCounter::GraphNode).map_err(MeteredActorStateError::Work)?;
            let actor = self.actor_states.get(&candidate.actor).copied();
            observed(ProjectionLookupOperation::ActorState);

            let (predecessor, predecessor_is_direct_dependency, actor_identity_matches) =
                if let Some(state) = actor {
                    charge(WorkCounter::GraphEdge).map_err(MeteredActorStateError::Work)?;
                    let direct = candidate
                        .dependencies
                        .binary_search(&state.highest_change)
                        .is_ok();
                    observed(ProjectionLookupOperation::DirectDependency);

                    charge(WorkCounter::GraphNode).map_err(MeteredActorStateError::Work)?;
                    let predecessor_candidate = self.branch_membership.get(&state.highest_change);
                    observed(ProjectionLookupOperation::PredecessorCandidate);

                    charge(WorkCounter::GraphNode).map_err(MeteredActorStateError::Work)?;
                    let actor_matches = predecessor_candidate
                        .is_some_and(|predecessor| predecessor.actor == candidate.actor);
                    observed(ProjectionLookupOperation::ActorIdentityComparison);
                    (Some(state.highest_change), direct, actor_matches)
                } else {
                    (None, false, true)
                };

            charge(WorkCounter::GraphNode).map_err(MeteredActorStateError::Work)?;
            let expected_sequence =
                actor.map_or(Some(1), |state| state.last_sequence.checked_add(1));
            observed(ProjectionLookupOperation::ExpectedSequence);
            let expected_sequence = expected_sequence
                .ok_or(MeteredActorStateError::State(ActorStateError::SequenceGap))?;

            charge(WorkCounter::GraphNode).map_err(MeteredActorStateError::Work)?;
            let sequence_matches = candidate.sequence == expected_sequence;
            observed(ProjectionLookupOperation::SequenceComparison);

            charge(WorkCounter::GraphNode).map_err(MeteredActorStateError::Work)?;
            let expected_next_matches = candidate.start_op == self.causal_next_op;
            observed(ProjectionLookupOperation::ExpectedNextComparison);

            Ok(TrustedEpochView {
                branch_member,
                accepted_member,
                predecessor,
                predecessor_is_direct_dependency,
                actor_identity_matches,
                expected_sequence,
                sequence_matches,
                causal_next_op: self.causal_next_op,
                expected_next_matches,
            })
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum SourceOperation {
        PullMember(ChangeHash),
        ReadAcceptedMember(ChangeHash),
        ReadCandidate(ChangeHash),
        PullDependency(ChangeHash, usize, ChangeHash),
    }

    #[test]
    fn projection_build_operation_boundary_is_sealed_exhaustive_and_immediate() {
        let operations = [
            ProjectionBuildOperation::SourceCountRead,
            ProjectionBuildOperation::ExpectedCountComparison,
            ProjectionBuildOperation::CanonicalSourcePull,
            ProjectionBuildOperation::CanonicalOrderCompare,
            ProjectionBuildOperation::MembershipLookup,
            ProjectionBuildOperation::CandidateLookup,
            ProjectionBuildOperation::CandidateIdentityComparison,
            ProjectionBuildOperation::DependencyCountRead,
            ProjectionBuildOperation::DependencyLookup,
            ProjectionBuildOperation::CandidateReadinessComparison,
            ProjectionBuildOperation::StateLookup,
            ProjectionBuildOperation::ReadinessTransition,
            ProjectionBuildOperation::CandidateKindComparison,
            ProjectionBuildOperation::CheckedArithmetic,
            ProjectionBuildOperation::RemainingStateWrite,
            ProjectionBuildOperation::MapInsertion,
            ProjectionBuildOperation::SetInsertion,
            ProjectionBuildOperation::CausalMaximumCompare,
            ProjectionBuildOperation::CompletionComparison,
            ProjectionBuildOperation::ResultPublication,
        ];
        assert_eq!(operations.len(), 20);

        let events = RefCell::new(Vec::new());
        let mut charge = |counter| {
            events.borrow_mut().push(("charge", Some(counter), None));
            Ok::<_, Completion>(())
        };
        let mut observed = |observation: ProjectionBuildObservation| {
            events.borrow_mut().push((
                match observation.kind {
                    ProjectionBuildObservationKind::ChargeAttempt => "attempt",
                    ProjectionBuildObservationKind::TargetCompleted => "observed",
                },
                Some(observation.descriptor.counter),
                Some(observation.descriptor.site),
            ));
        };
        let result = perform_projection_build_operation(
            ProjectionBuildSite::NextMemberPull,
            &mut charge,
            &mut observed,
            || {
                events.borrow_mut().push(("operation", None, None));
                7
            },
        );
        assert_eq!(result, Ok(7));
        assert_eq!(
            events.into_inner(),
            [
                (
                    "attempt",
                    Some(WorkCounter::GraphNode),
                    Some(ProjectionBuildSite::NextMemberPull),
                ),
                ("charge", Some(WorkCounter::GraphNode), None),
                ("operation", None, None),
                (
                    "observed",
                    Some(WorkCounter::GraphNode),
                    Some(ProjectionBuildSite::NextMemberPull),
                ),
            ]
        );

        let performed = Cell::new(false);
        let observations = Cell::new(0);
        let injected = Completion::Cancelled;
        let stopped = perform_projection_build_operation(
            ProjectionBuildSite::DependencyPull,
            &mut |_| Err(&injected),
            &mut |_| observations.set(observations.get() + 1),
            || performed.set(true),
        );
        assert!(matches!(
            stopped,
            Err(MeteredActorStateError::Work(error)) if core::ptr::eq(error, &injected)
        ));
        assert!(!performed.get());
        assert_eq!(observations.get(), 1);
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum TraversalTrace {
        Charge(WorkCounter),
        Operation(SourceOperation),
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum BuildTrace {
        Attempt(ProjectionBuildDescriptor),
        Charge(WorkCounter),
        Operation(ProjectionBuildDescriptor),
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum LookupTrace {
        Charge(WorkCounter),
        Operation(ProjectionLookupOperation),
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum ActorDecisionTrace {
        Attempt(ActorDecisionDescriptor),
        Charge(WorkCounter),
        Operation(ActorDecisionDescriptor),
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum CausalNextTrace {
        Attempt(CausalNextDescriptor),
        Charge(WorkCounter),
        Operation(CausalNextDescriptor),
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum FrontierTrace {
        Charge(WorkCounter),
        Operation(FrontierComparisonOperation),
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum PublicationTrace {
        Charge(WorkCounter),
        Publication(ProjectionPublicationOperation),
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum ProjectionWorkTrace {
        Charge(WorkCounter),
        Target,
    }

    struct ObservedEpochProjectionSource<'a> {
        members: Vec<ChangeHash>,
        cursor: usize,
        accepted_closure: &'a BTreeSet<ChangeHash>,
        changes: &'a BTreeMap<ChangeHash, ChangeCandidate>,
        trace: Rc<RefCell<Vec<TraversalTrace>>>,
    }

    struct WorkContractEpochProjectionSource<'a> {
        members: std::collections::btree_set::Iter<'a, ChangeHash>,
        accepted_closure: &'a BTreeSet<ChangeHash>,
        changes: &'a BTreeMap<ChangeHash, ChangeCandidate>,
        trace: Rc<RefCell<Vec<ProjectionWorkTrace>>>,
    }

    impl<'a> EpochProjectionSource<'a> for WorkContractEpochProjectionSource<'a> {
        fn member_count(&self) -> usize {
            self.accepted_closure.len()
        }

        fn next_member(&mut self) -> Option<ChangeHash> {
            let member = self.members.next().copied();
            if member.is_some() {
                self.trace.borrow_mut().push(ProjectionWorkTrace::Target);
            }
            member
        }

        fn accepted_member(&mut self, hash: &ChangeHash) -> bool {
            self.trace.borrow_mut().push(ProjectionWorkTrace::Target);
            self.accepted_closure.contains(hash)
        }

        fn candidate(&mut self, hash: &ChangeHash) -> Option<&'a ChangeCandidate> {
            self.trace.borrow_mut().push(ProjectionWorkTrace::Target);
            self.changes.get(hash)
        }

        fn dependency_count(&mut self, candidate: &ChangeCandidate) -> usize {
            candidate.dependencies.len()
        }

        fn dependency(&mut self, candidate: &ChangeCandidate, index: usize) -> Option<ChangeHash> {
            let dependency = candidate.dependencies.get(index).copied();
            if dependency.is_some() {
                self.trace.borrow_mut().push(ProjectionWorkTrace::Target);
            }
            dependency
        }
    }

    impl<'a> EpochProjectionSource<'a> for ObservedEpochProjectionSource<'a> {
        fn member_count(&self) -> usize {
            self.members.len()
        }

        fn next_member(&mut self) -> Option<ChangeHash> {
            let member = self.members.get(self.cursor).copied();
            self.cursor = self.cursor.saturating_add(1);
            if let Some(member) = member {
                self.trace.borrow_mut().push(TraversalTrace::Operation(
                    SourceOperation::PullMember(member),
                ));
            }
            member
        }

        fn accepted_member(&mut self, hash: &ChangeHash) -> bool {
            self.trace.borrow_mut().push(TraversalTrace::Operation(
                SourceOperation::ReadAcceptedMember(*hash),
            ));
            self.accepted_closure.contains(hash)
        }

        fn candidate(&mut self, hash: &ChangeHash) -> Option<&'a ChangeCandidate> {
            self.trace.borrow_mut().push(TraversalTrace::Operation(
                SourceOperation::ReadCandidate(*hash),
            ));
            self.changes.get(hash)
        }

        fn dependency_count(&mut self, candidate: &ChangeCandidate) -> usize {
            candidate.dependencies.len()
        }

        fn dependency(&mut self, candidate: &ChangeCandidate, index: usize) -> Option<ChangeHash> {
            let dependency = candidate.dependencies.get(index).copied();
            if let Some(dependency) = dependency {
                self.trace.borrow_mut().push(TraversalTrace::Operation(
                    SourceOperation::PullDependency(candidate.change_hash, index, dependency),
                ));
            }
            dependency
        }
    }

    pub(crate) fn candidate(actor: u8, sequence: u64, start: u64, count: u64) -> ChangeCandidate {
        ChangeCandidate {
            change_hash: ChangeHash::from_bytes([u8::try_from(sequence).unwrap_or_default(); 32]),
            actor: ActorId::from_bytes([actor; 32]),
            sequence,
            start_op: start,
            operation_count: count,
            dependencies: Vec::new().into(),
            control_id: EventId::from_bytes([9; 32]),
            author: DevicePublicKey::from_bytes([actor; 32]),
            valid_carriers: vec![EventId::from_bytes([actor; 32])].into(),
        }
    }

    #[test]
    fn initialize_actor_state_from_epoch_base() {
        let mut first = candidate(1, 1, 1, 2);
        first.change_hash = ChangeHash::from_bytes([9; 32]);
        let mut empty_second = candidate(1, 2, 3, 0);
        empty_second.change_hash = ChangeHash::from_bytes([1; 32]);
        empty_second.dependencies = vec![first.change_hash].into();
        let mut other_actor = candidate(2, 1, 1, 0);
        other_actor.change_hash = ChangeHash::from_bytes([8; 32]);
        let states = initialize_actor_states([empty_second.clone(), other_actor, first.clone()]);
        assert!(states.is_ok());
        let states = match states {
            Ok(states) => states,
            Err(_) => return,
        };
        assert_eq!(states[&ActorId::from_bytes([1; 32])].last_sequence, 2);
        assert_eq!(states[&ActorId::from_bytes([1; 32])].next_op, 3);
        assert_eq!(
            states[&ActorId::from_bytes([1; 32])].highest_change,
            empty_second.change_hash
        );
        assert_eq!(
            initialize_actor_states([candidate(1, 2, 1, 1)]),
            Err(ActorStateError::SequenceGap)
        );
        let mut conflict = candidate(1, 1, 1, 1);
        conflict.change_hash = ChangeHash::from_bytes([8; 32]);
        assert_eq!(
            initialize_actor_states([candidate(1, 1, 1, 1), conflict]),
            Err(ActorStateError::Equivocation)
        );

        let mut missing = candidate(3, 1, 1, 1);
        missing.dependencies = vec![ChangeHash::from_bytes([7; 32])].into();
        assert_eq!(
            initialize_actor_states([missing]),
            Err(ActorStateError::MissingDependency)
        );

        let mut left = candidate(3, 1, 1, 1);
        left.change_hash = ChangeHash::from_bytes([3; 32]);
        let mut right = candidate(4, 1, 1, 1);
        right.change_hash = ChangeHash::from_bytes([4; 32]);
        left.dependencies = vec![right.change_hash].into();
        right.dependencies = vec![left.change_hash].into();
        assert_eq!(
            initialize_actor_states([right, left]),
            Err(ActorStateError::DependencyCycle)
        );
    }

    #[test]
    fn metered_actor_projection_borrows_a_candidate_superset() {
        let first = candidate(1, 1, 1, 1);
        let mut ignored = candidate(2, 1, 1, 1);
        ignored.change_hash = ChangeHash::from_bytes([2; 32]);
        let closure = BTreeSet::from([first.change_hash]);
        let candidates = BTreeMap::from([
            (first.change_hash, first.clone()),
            (ignored.change_hash, ignored),
        ]);
        let mut charges = Vec::new();
        let result = initialize_actor_states_metered(&closure, &candidates, |counter| {
            charges.push(counter);
            Ok::<_, ()>(())
        });
        assert!(result.is_ok_and(|projection| {
            let mut lookup_charges = Vec::new();
            let Ok(view) = projection.candidate_metered(&first, |counter| {
                lookup_charges.push(counter);
                Ok::<_, ()>(())
            }) else {
                return false;
            };
            lookup_charges.len() == 9
                && view.is_branch_member()
                && view.is_accepted_member()
                && view
                    .predecessor()
                    .is_some_and(|hash| hash == first.change_hash)
                && !view.predecessor_is_direct_dependency()
                && view.actor_identity_matches()
                && view.expected_sequence() == 2
                && !view.sequence_matches()
                && view.causal_next_op() == 2
                && !view.expected_next_matches()
                && projection.dependency_count(&first.change_hash) == 0
                && projection.dependencies(&first.change_hash).next().is_none()
                && projection.frontier_heads().eq([first.change_hash])
                && projection
                    .writer_contributions()
                    .eq([(first.actor, first.change_hash)])
        }));
        assert_eq!(
            charges
                .iter()
                .filter(|counter| **counter == WorkCounter::GraphNode)
                .count(),
            31
        );
        assert_eq!(
            charges
                .iter()
                .filter(|counter| **counter == WorkCounter::GraphEdge)
                .count(),
            1
        );
    }

    fn observed_candidate_lookup<E: Copy>(
        projection: &TrustedEpochProjection<'_>,
        candidate: &ChangeCandidate,
        successful_limit: usize,
        stopped: E,
    ) -> (
        Result<TrustedEpochView, MeteredActorStateError<E>>,
        Vec<LookupTrace>,
    ) {
        let trace = Rc::new(RefCell::new(Vec::new()));
        let successful = Cell::new(0_usize);
        let result = projection.candidate_metered_observed(
            candidate,
            |counter| {
                trace.borrow_mut().push(LookupTrace::Charge(counter));
                if successful.get() == successful_limit {
                    Err(stopped)
                } else {
                    successful.set(successful.get().saturating_add(1));
                    Ok(())
                }
            },
            |operation| {
                trace.borrow_mut().push(LookupTrace::Operation(operation));
            },
        );
        let observed = trace.borrow().clone();
        (result, observed)
    }

    fn observed_actor_sequence<E: Copy>(
        projection: &TrustedEpochProjection<'_>,
        candidate: &ChangeCandidate,
        successful_limit: usize,
        stopped: E,
    ) -> (
        Result<(), MeteredActorStateError<E>>,
        Vec<ActorDecisionTrace>,
    ) {
        let trace = Rc::new(RefCell::new(Vec::new()));
        let successful = Cell::new(0_usize);
        let result = projection.actor_sequence_decision_metered_observed(
            candidate,
            |counter| {
                trace.borrow_mut().push(ActorDecisionTrace::Charge(counter));
                if successful.get() == successful_limit {
                    Err(stopped)
                } else {
                    successful.set(successful.get().saturating_add(1));
                    Ok(())
                }
            },
            |observation| {
                trace.borrow_mut().push(match observation.kind {
                    ActorDecisionObservationKind::ChargeAttempt => {
                        ActorDecisionTrace::Attempt(observation.descriptor)
                    }
                    ActorDecisionObservationKind::TargetCompleted => {
                        ActorDecisionTrace::Operation(observation.descriptor)
                    }
                });
            },
        );
        let observed = trace.borrow().clone();
        (result, observed)
    }

    fn observed_causal_next<E: Copy>(
        projection: &TrustedEpochProjection<'_>,
        candidate: &ChangeCandidate,
        successful_limit: usize,
        stopped: E,
    ) -> (Result<u64, MeteredActorStateError<E>>, Vec<CausalNextTrace>) {
        let trace = Rc::new(RefCell::new(Vec::new()));
        let successful = Cell::new(0_usize);
        let result = projection.causal_next_decision_metered_observed(
            candidate,
            |counter| {
                trace.borrow_mut().push(CausalNextTrace::Charge(counter));
                if successful.get() == successful_limit {
                    Err(stopped)
                } else {
                    successful.set(successful.get().saturating_add(1));
                    Ok(())
                }
            },
            |observation| {
                trace.borrow_mut().push(match observation.kind {
                    CausalNextObservationKind::ChargeAttempt => {
                        CausalNextTrace::Attempt(observation.descriptor)
                    }
                    CausalNextObservationKind::TargetCompleted => {
                        CausalNextTrace::Operation(observation.descriptor)
                    }
                });
            },
        );
        let observed = trace.borrow().clone();
        (result, observed)
    }

    fn observed_empty_frontier<E: Copy>(
        projection: &TrustedEpochProjection<'_>,
        candidate: &ChangeCandidate,
        base_frontier: &BTreeSet<ChangeHash>,
        successful_limit: usize,
        stopped: E,
    ) -> (Result<(), MeteredActorStateError<E>>, Vec<FrontierTrace>) {
        let trace = Rc::new(RefCell::new(Vec::new()));
        let successful = Cell::new(0_usize);
        let result = projection.empty_frontier_decision_metered_observed(
            candidate,
            base_frontier,
            |counter| {
                trace.borrow_mut().push(FrontierTrace::Charge(counter));
                if successful.get() == successful_limit {
                    Err(stopped)
                } else {
                    successful.set(successful.get().saturating_add(1));
                    Ok(())
                }
            },
            |operation| {
                trace.borrow_mut().push(FrontierTrace::Operation(operation));
            },
        );
        let observed = trace.borrow().clone();
        (result, observed)
    }

    fn observed_projection_publication<'a>(
        accepted_closure: &'a BTreeSet<ChangeHash>,
        changes: &'a BTreeMap<ChangeHash, ChangeCandidate>,
        successful_limit: usize,
        stopped: Completion,
    ) -> (
        Result<TrustedEpochProjection<'a>, MeteredActorStateError<Completion>>,
        Vec<PublicationTrace>,
    ) {
        let trace = Rc::new(RefCell::new(Vec::new()));
        let successful = Cell::new(0_usize);
        let mut source = CanonicalEpochProjectionSource::new(accepted_closure, changes);
        let result = build_trusted_epoch_projection_observed(
            accepted_closure,
            changes,
            &mut source,
            |counter| {
                trace.borrow_mut().push(PublicationTrace::Charge(counter));
                if successful.get() == successful_limit {
                    Err(stopped)
                } else {
                    successful.set(successful.get().saturating_add(1));
                    Ok(())
                }
            },
            |_| {},
            |operation| {
                trace
                    .borrow_mut()
                    .push(PublicationTrace::Publication(operation));
            },
        );
        let observed = trace.borrow().clone();
        (result, observed)
    }

    fn assert_projection_build_family_exact(family: ProjectionBuildOperation) {
        let first = candidate(1, 1, 1, 1);
        let mut second = candidate(1, 2, 2, 1);
        second.change_hash = ChangeHash::from_bytes([2; 32]);
        second.dependencies = vec![first.change_hash].into();
        let accepted = BTreeSet::from([first.change_hash, second.change_hash]);
        let changes = BTreeMap::from([(first.change_hash, first), (second.change_hash, second)]);
        let (complete, trace) = observed_projection_build_operations(
            &accepted,
            &changes,
            usize::MAX,
            Completion::BudgetExhausted,
        );
        assert!(complete.is_ok());
        let mut charges = 0_usize;
        let target_charge = trace.iter().find_map(|entry| match entry {
            BuildTrace::Attempt(_) => None,
            BuildTrace::Charge(_) => {
                charges = charges.saturating_add(1);
                None
            }
            BuildTrace::Operation(site) if site.operation() == family => Some(charges),
            BuildTrace::Operation(_) => None,
        });
        assert!(target_charge.is_some_and(|value| value > 0));
        let Some(target_charge) = target_charge else {
            return;
        };
        for stopped in [Completion::BudgetExhausted, Completion::Cancelled] {
            let (blocked, blocked_trace) = observed_projection_build_operations(
                &accepted,
                &changes,
                target_charge - 1,
                stopped,
            );
            assert!(matches!(
                blocked,
                Err(MeteredActorStateError::Work(value)) if value == stopped
            ));
            assert_eq!(
                blocked_trace
                    .iter()
                    .filter(
                        |entry| matches!(entry, BuildTrace::Operation(site) if site.operation() == family)
                    )
                    .count(),
                0
            );
        }
        for allowance in [target_charge, target_charge + 1] {
            let (_, admitted_trace) = observed_projection_build_operations(
                &accepted,
                &changes,
                allowance,
                Completion::BudgetExhausted,
            );
            assert!(admitted_trace.iter().any(
                |entry| matches!(entry, BuildTrace::Operation(site) if site.operation() == family)
            ));
        }
    }

    macro_rules! projection_build_family_proofs {
        ($(($test:ident, $family:ident)),+ $(,)?) => {
            $(
                #[test]
                fn $test() {
                    assert_projection_build_family_exact(ProjectionBuildOperation::$family);
                }
            )+
        };
    }

    projection_build_family_proofs!(
        (
            causal_projection_proof_construction_source_count_read,
            SourceCountRead
        ),
        (
            causal_projection_proof_construction_expected_count_comparison,
            ExpectedCountComparison
        ),
        (
            causal_projection_proof_construction_canonical_source_pull,
            CanonicalSourcePull
        ),
        (
            causal_projection_proof_construction_canonical_order_compare,
            CanonicalOrderCompare
        ),
        (
            causal_projection_proof_construction_membership_lookup,
            MembershipLookup
        ),
        (
            causal_projection_proof_construction_candidate_lookup,
            CandidateLookup
        ),
        (
            causal_projection_proof_construction_candidate_identity_comparison,
            CandidateIdentityComparison
        ),
        (
            causal_projection_proof_construction_dependency_count_read,
            DependencyCountRead
        ),
        (
            causal_projection_proof_construction_dependency_lookup,
            DependencyLookup
        ),
        (
            causal_projection_proof_construction_candidate_readiness_comparison,
            CandidateReadinessComparison
        ),
        (
            causal_projection_proof_construction_state_lookup,
            StateLookup
        ),
        (
            causal_projection_proof_construction_readiness_transition,
            ReadinessTransition
        ),
        (
            causal_projection_proof_construction_candidate_kind_comparison,
            CandidateKindComparison
        ),
        (
            causal_projection_proof_construction_checked_arithmetic,
            CheckedArithmetic
        ),
        (
            causal_projection_proof_construction_remaining_state_write,
            RemainingStateWrite
        ),
        (
            causal_projection_proof_construction_map_insertion,
            MapInsertion
        ),
        (
            causal_projection_proof_construction_set_insertion,
            SetInsertion
        ),
        (
            causal_projection_proof_construction_causal_maximum_compare,
            CausalMaximumCompare
        ),
        (
            causal_projection_proof_construction_completion_comparison,
            CompletionComparison
        ),
        (
            causal_projection_proof_construction_result_publication,
            ResultPublication
        ),
    );

    fn assert_projection_lookup_family_exact(family: ProjectionLookupOperation) {
        let first = candidate(1, 1, 1, 1);
        let mut second = candidate(1, 2, 2, 1);
        second.change_hash = ChangeHash::from_bytes([2; 32]);
        second.dependencies = vec![first.change_hash].into();
        let closure = BTreeSet::from([first.change_hash, second.change_hash]);
        let changes = BTreeMap::from([
            (first.change_hash, first),
            (second.change_hash, second.clone()),
        ]);
        let projection = initialize_actor_states_metered(&closure, &changes, |_| Ok::<_, ()>(()));
        assert!(projection.is_ok());
        let Some(projection) = projection.ok() else {
            return;
        };
        let mut query = candidate(1, 3, 3, 1);
        query.change_hash = ChangeHash::from_bytes([3; 32]);
        query.dependencies = vec![second.change_hash].into();
        let (complete, trace) =
            observed_candidate_lookup(&projection, &query, usize::MAX, Completion::BudgetExhausted);
        assert!(complete.is_ok());
        let target = trace
            .iter()
            .filter(|entry| matches!(entry, LookupTrace::Operation(_)))
            .position(|entry| matches!(entry, LookupTrace::Operation(value) if *value == family))
            .map(|index| index + 1);
        assert!(target.is_some());
        let Some(target) = target else { return };
        for stopped in [Completion::BudgetExhausted, Completion::Cancelled] {
            let (blocked, blocked_trace) =
                observed_candidate_lookup(&projection, &query, target - 1, stopped);
            assert!(matches!(
                blocked,
                Err(MeteredActorStateError::Work(value)) if value == stopped
            ));
            assert!(
                !blocked_trace.iter().any(
                    |entry| matches!(entry, LookupTrace::Operation(value) if *value == family)
                )
            );
        }
        for allowance in [target, target + 1] {
            let (_, admitted_trace) = observed_candidate_lookup(
                &projection,
                &query,
                allowance,
                Completion::BudgetExhausted,
            );
            assert!(
                admitted_trace.iter().any(
                    |entry| matches!(entry, LookupTrace::Operation(value) if *value == family)
                )
            );
        }
    }

    macro_rules! projection_lookup_family_proofs {
        ($(($test:ident, $family:ident)),+ $(,)?) => {
            $(
                #[test]
                fn $test() {
                    assert_projection_lookup_family_exact(ProjectionLookupOperation::$family);
                }
            )+
        };
    }

    projection_lookup_family_proofs!(
        (
            causal_projection_proof_lookup_branch_membership,
            BranchMembership
        ),
        (
            causal_projection_proof_lookup_accepted_membership,
            AcceptedMembership
        ),
        (causal_projection_proof_lookup_actor_state, ActorState),
        (
            causal_projection_proof_lookup_direct_dependency,
            DirectDependency
        ),
        (
            causal_projection_proof_lookup_predecessor_candidate,
            PredecessorCandidate
        ),
        (
            causal_projection_proof_lookup_actor_identity_comparison,
            ActorIdentityComparison
        ),
        (
            causal_projection_proof_lookup_expected_sequence,
            ExpectedSequence
        ),
        (
            causal_projection_proof_lookup_sequence_comparison,
            SequenceComparison
        ),
        (
            causal_projection_proof_lookup_expected_next_comparison,
            ExpectedNextComparison
        ),
    );

    fn assert_causal_consumer_family_exact(family: CausalNextOperation) {
        let branch = BTreeMap::new();
        let closure = BTreeSet::new();
        let projection = focused_causal_consumer_projection(&branch, &closure, 7);
        let candidate = candidate(1, 1, 7, 1);
        let (complete, trace) = observed_causal_next(
            &projection,
            &candidate,
            usize::MAX,
            Completion::BudgetExhausted,
        );
        assert_eq!(complete, Ok(8));
        let target = trace
            .iter()
            .filter(|entry| matches!(entry, CausalNextTrace::Operation(_)))
            .position(
                |entry| matches!(entry, CausalNextTrace::Operation(value) if value.operation() == family),
            )
            .map(|index| index + 1);
        assert!(target.is_some());
        let Some(target) = target else { return };
        for stopped in [Completion::BudgetExhausted, Completion::Cancelled] {
            let (blocked, blocked_trace) =
                observed_causal_next(&projection, &candidate, target - 1, stopped);
            assert!(matches!(
                blocked,
                Err(MeteredActorStateError::Work(value)) if value == stopped
            ));
            assert!(!blocked_trace.iter().any(
                |entry| matches!(entry, CausalNextTrace::Operation(value) if value.operation() == family)
            ));
        }
        for allowance in [target, target + 1] {
            let (_, admitted_trace) = observed_causal_next(
                &projection,
                &candidate,
                allowance,
                Completion::BudgetExhausted,
            );
            assert!(admitted_trace.iter().any(
                |entry| matches!(entry, CausalNextTrace::Operation(value) if value.operation() == family)
            ));
        }
        let injected = family;
        let (failed, _) = observed_causal_next(&projection, &candidate, target - 1, &injected);
        assert!(matches!(
            failed,
            Err(MeteredActorStateError::Work(error)) if core::ptr::eq(error, &injected)
        ));
    }

    fn assert_frontier_family_exact(family: FrontierComparisonOperation) {
        let projected_first = ChangeHash::from_bytes([10; 32]);
        let base_only = ChangeHash::from_bytes([20; 32]);
        let projected_last = ChangeHash::from_bytes([30; 32]);
        let branch = BTreeMap::new();
        let accepted = BTreeSet::from([projected_first, projected_last]);
        let projection = TrustedEpochProjection {
            branch_membership: &branch,
            accepted_closure: &accepted,
            dependencies: BTreeMap::new(),
            frontier_heads: BTreeSet::from([projected_first, projected_last]),
            actor_states: BTreeMap::new(),
            writer_contributions: BTreeMap::new(),
            causal_next_op: 1,
        };
        let base_frontier = BTreeSet::from([projected_first, base_only]);
        let mut exact = candidate(1, 1, 1, 0);
        exact.dependencies = vec![projected_first, base_only, projected_last].into();
        let (complete, trace) = observed_empty_frontier(
            &projection,
            &exact,
            &base_frontier,
            usize::MAX,
            Completion::BudgetExhausted,
        );
        assert_eq!(complete, Ok(()));
        let target = trace
            .iter()
            .filter(|entry| matches!(entry, FrontierTrace::Operation(_)))
            .position(|entry| matches!(entry, FrontierTrace::Operation(value) if *value == family))
            .map(|index| index + 1);
        assert!(target.is_some(), "missing frontier operation {family:?}");
        let Some(target) = target else { return };
        for stopped in [Completion::BudgetExhausted, Completion::Cancelled] {
            let (blocked, blocked_trace) =
                observed_empty_frontier(&projection, &exact, &base_frontier, target - 1, stopped);
            assert!(matches!(
                blocked,
                Err(MeteredActorStateError::Work(value)) if value == stopped
            ));
            assert!(
                !blocked_trace.iter().any(
                    |entry| matches!(entry, FrontierTrace::Operation(value) if *value == family)
                )
            );
        }
        for allowance in [target, target + 1] {
            let (_, admitted_trace) = observed_empty_frontier(
                &projection,
                &exact,
                &base_frontier,
                allowance,
                Completion::BudgetExhausted,
            );
            assert!(
                admitted_trace.iter().any(
                    |entry| matches!(entry, FrontierTrace::Operation(value) if *value == family)
                )
            );
        }
        let injected = family;
        let (failed, _) =
            observed_empty_frontier(&projection, &exact, &base_frontier, target - 1, &injected);
        assert!(matches!(
            failed,
            Err(MeteredActorStateError::Work(error)) if core::ptr::eq(error, &injected)
        ));
    }

    macro_rules! frontier_family_proofs {
        ($(($test:ident, $family:ident)),+ $(,)?) => {
            $(
                #[test]
                fn $test() {
                    assert_frontier_family_exact(FrontierComparisonOperation::$family);
                }
            )+
        };
    }

    frontier_family_proofs!(
        (
            causal_projection_proof_frontier_candidate_kind_comparison,
            CandidateKindComparison
        ),
        (
            causal_projection_proof_frontier_candidate_count,
            CandidateCount
        ),
        (
            causal_projection_proof_frontier_projection_count,
            ProjectionCount
        ),
        (causal_projection_proof_frontier_base_count, BaseCount),
        (
            causal_projection_proof_frontier_candidate_pull,
            CandidatePull
        ),
        (
            causal_projection_proof_frontier_candidate_order_comparison,
            CandidateOrderComparison
        ),
        (
            causal_projection_proof_frontier_projection_pull,
            ProjectionPull
        ),
        (causal_projection_proof_frontier_base_pull, BasePull),
        (
            causal_projection_proof_frontier_base_accepted_lookup,
            BaseAcceptedLookup
        ),
        (
            causal_projection_proof_frontier_expected_source_comparison,
            ExpectedSourceComparison
        ),
        (
            causal_projection_proof_frontier_frontier_equality_comparison,
            FrontierEqualityComparison
        ),
    );

    #[test]
    fn projection_allocation_insertion_and_publication_are_charged_before_work() {
        let first = candidate(1, 1, 1, 1);
        let mut second = candidate(2, 1, 2, 1);
        second.change_hash = ChangeHash::from_bytes([2; 32]);
        second.dependencies = vec![first.change_hash].into();
        let closure = BTreeSet::from([first.change_hash, second.change_hash]);
        let changes = BTreeMap::from([(first.change_hash, first), (second.change_hash, second)]);
        let (ample, full_trace) = observed_projection_publication(
            &closure,
            &changes,
            usize::MAX,
            Completion::BudgetExhausted,
        );
        assert!(ample.is_ok());

        let mut charge_count = 0_usize;
        let mut publication_count = 0_usize;
        let mut boundaries = Vec::new();
        for (index, entry) in full_trace.iter().enumerate() {
            match entry {
                PublicationTrace::Charge(_) => charge_count = charge_count.saturating_add(1),
                PublicationTrace::Publication(operation) => {
                    publication_count = publication_count.saturating_add(1);
                    let expected_counter = match operation {
                        ProjectionPublicationOperation::CandidateDependency
                        | ProjectionPublicationOperation::DependedOn
                        | ProjectionPublicationOperation::DependantBucket
                        | ProjectionPublicationOperation::Dependant => WorkCounter::GraphEdge,
                        ProjectionPublicationOperation::ReadyCandidate
                        | ProjectionPublicationOperation::RemainingDependencies
                        | ProjectionPublicationOperation::Dependencies
                        | ProjectionPublicationOperation::FrontierHead
                        | ProjectionPublicationOperation::ActorState
                        | ProjectionPublicationOperation::WriterContribution
                        | ProjectionPublicationOperation::ReadyDependant
                        | ProjectionPublicationOperation::Projection => WorkCounter::GraphNode,
                        ProjectionPublicationOperation::CausalCounter => {
                            match index.checked_sub(1).and_then(|prior| full_trace.get(prior)) {
                                Some(PublicationTrace::Charge(counter)) => *counter,
                                _ => WorkCounter::GraphNode,
                            }
                        }
                    };
                    assert_eq!(
                        index.checked_sub(1).and_then(|prior| full_trace.get(prior)),
                        Some(&PublicationTrace::Charge(expected_counter))
                    );
                    boundaries.push((charge_count, publication_count));
                }
            }
        }
        assert_eq!(boundaries.len(), 19);

        let count_publications = |trace: &[PublicationTrace]| {
            trace
                .iter()
                .filter(|entry| matches!(entry, PublicationTrace::Publication(_)))
                .count()
        };
        for (target_charge, target_publication) in boundaries {
            let (before, before_trace) = observed_projection_publication(
                &closure,
                &changes,
                target_charge - 1,
                Completion::BudgetExhausted,
            );
            assert!(matches!(
                before,
                Err(MeteredActorStateError::Work(Completion::BudgetExhausted))
            ));
            assert_eq!(count_publications(&before_trace), target_publication - 1);

            for allowance in [target_charge, target_charge + 1] {
                let (_, allowed_trace) = observed_projection_publication(
                    &closure,
                    &changes,
                    allowance,
                    Completion::BudgetExhausted,
                );
                assert!(count_publications(&allowed_trace) >= target_publication);
            }

            let (cancelled, cancelled_trace) = observed_projection_publication(
                &closure,
                &changes,
                target_charge - 1,
                Completion::Cancelled,
            );
            assert!(matches!(
                cancelled,
                Err(MeteredActorStateError::Work(Completion::Cancelled))
            ));
            assert_eq!(count_publications(&cancelled_trace), target_publication - 1);
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct ExpectedProjectionCase {
        predecessor: Option<ChangeHash>,
        direct: bool,
        actor_matches: bool,
        expected_sequence: u64,
        sequence_matches: bool,
        causal_next_op: u64,
        expected_next_matches: bool,
        frontier: Vec<ChangeHash>,
        writers: Vec<(ActorId, ChangeHash)>,
    }

    #[test]
    fn projection_semantic_matrix_is_complete_and_order_invariant() {
        let first = candidate(1, 1, 1, 1);
        let mut inherited_other = candidate(2, 1, 2, 1);
        inherited_other.change_hash = ChangeHash::from_bytes([2; 32]);
        inherited_other.dependencies = vec![first.change_hash].into();
        let mut unrelated_other = candidate(2, 1, 1, 1);
        unrelated_other.change_hash = ChangeHash::from_bytes([4; 32]);

        let cases = [
            (
                "empty",
                Vec::new(),
                candidate(1, 1, 1, 1),
                ExpectedProjectionCase {
                    predecessor: None,
                    direct: false,
                    actor_matches: true,
                    expected_sequence: 1,
                    sequence_matches: true,
                    causal_next_op: 1,
                    expected_next_matches: true,
                    frontier: Vec::new(),
                    writers: Vec::new(),
                },
            ),
            (
                "single",
                vec![first.clone()],
                candidate(1, 2, 2, 1),
                ExpectedProjectionCase {
                    predecessor: Some(first.change_hash),
                    direct: false,
                    actor_matches: true,
                    expected_sequence: 2,
                    sequence_matches: true,
                    causal_next_op: 2,
                    expected_next_matches: true,
                    frontier: vec![first.change_hash],
                    writers: vec![(first.actor, first.change_hash)],
                },
            ),
            (
                "deep_predecessor",
                vec![first.clone(), inherited_other.clone()],
                {
                    let mut query = candidate(1, 2, 3, 1);
                    query.change_hash = ChangeHash::from_bytes([3; 32]);
                    query.dependencies = vec![inherited_other.change_hash].into();
                    query
                },
                ExpectedProjectionCase {
                    predecessor: Some(first.change_hash),
                    direct: false,
                    actor_matches: true,
                    expected_sequence: 2,
                    sequence_matches: true,
                    causal_next_op: 3,
                    expected_next_matches: true,
                    frontier: vec![inherited_other.change_hash],
                    writers: vec![
                        (first.actor, first.change_hash),
                        (inherited_other.actor, inherited_other.change_hash),
                    ],
                },
            ),
            (
                "unrelated_dependency",
                vec![first.clone(), unrelated_other.clone()],
                {
                    let mut query = candidate(1, 2, 2, 1);
                    query.change_hash = ChangeHash::from_bytes([5; 32]);
                    query.dependencies = vec![unrelated_other.change_hash].into();
                    query
                },
                ExpectedProjectionCase {
                    predecessor: Some(first.change_hash),
                    direct: false,
                    actor_matches: true,
                    expected_sequence: 2,
                    sequence_matches: true,
                    causal_next_op: 2,
                    expected_next_matches: true,
                    frontier: vec![first.change_hash, unrelated_other.change_hash],
                    writers: vec![
                        (first.actor, first.change_hash),
                        (unrelated_other.actor, unrelated_other.change_hash),
                    ],
                },
            ),
            (
                "actor_gap",
                vec![first.clone()],
                candidate(1, 3, 2, 1),
                ExpectedProjectionCase {
                    predecessor: Some(first.change_hash),
                    direct: false,
                    actor_matches: true,
                    expected_sequence: 2,
                    sequence_matches: false,
                    causal_next_op: 2,
                    expected_next_matches: true,
                    frontier: vec![first.change_hash],
                    writers: vec![(first.actor, first.change_hash)],
                },
            ),
            (
                "actor_rollback",
                vec![first.clone()],
                candidate(1, 1, 2, 1),
                ExpectedProjectionCase {
                    predecessor: Some(first.change_hash),
                    direct: false,
                    actor_matches: true,
                    expected_sequence: 2,
                    sequence_matches: false,
                    causal_next_op: 2,
                    expected_next_matches: true,
                    frontier: vec![first.change_hash],
                    writers: vec![(first.actor, first.change_hash)],
                },
            ),
        ];

        for (name, accepted, query, expected) in cases {
            let accepted_hashes = accepted
                .iter()
                .map(|candidate| candidate.change_hash)
                .collect::<BTreeSet<_>>();
            for mut order in [accepted.clone(), accepted.into_iter().rev().collect()] {
                let changes = order
                    .drain(..)
                    .map(|candidate| (candidate.change_hash, candidate))
                    .collect::<BTreeMap<_, _>>();
                let projection = initialize_actor_states_metered(
                    &accepted_hashes,
                    &changes,
                    |_| Ok::<_, ()>(()),
                );
                assert!(projection.is_ok(), "{name}");
                let Some(projection) = projection.ok() else {
                    continue;
                };
                let view = projection.candidate_metered(&query, |_| Ok::<_, ()>(()));
                assert!(view.is_ok(), "{name}");
                let Some(view) = view.ok() else { continue };
                let actual = ExpectedProjectionCase {
                    predecessor: view.predecessor(),
                    direct: view.predecessor_is_direct_dependency(),
                    actor_matches: view.actor_identity_matches(),
                    expected_sequence: view.expected_sequence(),
                    sequence_matches: view.sequence_matches(),
                    causal_next_op: view.causal_next_op(),
                    expected_next_matches: view.expected_next_matches(),
                    frontier: projection.frontier_heads().collect(),
                    writers: projection.writer_contributions().collect(),
                };
                assert_eq!(actual, expected, "{name}");
            }
        }

        let overflow_hash = ChangeHash::from_bytes([8; 32]);
        let overflow_actor = ActorId::from_bytes([8; 32]);
        let overflow_candidate = ChangeCandidate {
            change_hash: overflow_hash,
            actor: overflow_actor,
            sequence: u64::MAX,
            start_op: 1,
            operation_count: 1,
            dependencies: Vec::new().into(),
            control_id: EventId::from_bytes([9; 32]),
            author: DevicePublicKey::from_bytes([8; 32]),
            valid_carriers: Vec::new().into(),
        };
        let overflow_branch = BTreeMap::from([(overflow_hash, overflow_candidate.clone())]);
        let overflow_closure = BTreeSet::from([overflow_hash]);
        let overflow_projection = TrustedEpochProjection {
            branch_membership: &overflow_branch,
            accepted_closure: &overflow_closure,
            dependencies: BTreeMap::from([(overflow_hash, BTreeSet::new())]),
            frontier_heads: BTreeSet::from([overflow_hash]),
            actor_states: BTreeMap::from([(
                overflow_actor,
                EpochActorState {
                    last_sequence: u64::MAX,
                    next_op: 1,
                    highest_change: overflow_hash,
                },
            )]),
            writer_contributions: BTreeMap::from([(overflow_actor, overflow_hash)]),
            causal_next_op: 1,
        };
        assert!(matches!(
            overflow_projection.candidate_metered(&overflow_candidate, |_| Ok::<_, ()>(())),
            Err(MeteredActorStateError::State(ActorStateError::SequenceGap))
        ));

        let wide = (1_u8..=8)
            .map(|index| {
                let mut change = candidate(index, 1, 1, 1);
                change.change_hash = ChangeHash::from_bytes([index; 32]);
                change
            })
            .collect::<Vec<_>>();
        let wide_hashes = wide
            .iter()
            .map(|candidate| candidate.change_hash)
            .collect::<BTreeSet<_>>();
        let wide_changes = wide
            .iter()
            .cloned()
            .map(|candidate| (candidate.change_hash, candidate))
            .collect::<BTreeMap<_, _>>();
        let wide_projection =
            initialize_actor_states_metered(&wide_hashes, &wide_changes, |_| Ok::<_, ()>(()));
        assert!(wide_projection.is_ok());
        let Some(wide_projection) = wide_projection.ok() else {
            return;
        };
        assert!(
            wide_projection
                .frontier_heads()
                .eq(wide_hashes.iter().copied())
        );
        assert_eq!(wide_projection.writer_contributions().count(), 8);
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct ProjectionSemanticSnapshot {
        accepted: Vec<ChangeHash>,
        dependencies: Vec<(ChangeHash, Vec<ChangeHash>)>,
        states: Vec<(ActorId, u64, u64, ChangeHash)>,
        writers: Vec<(ActorId, ChangeHash)>,
        frontier: Vec<ChangeHash>,
        causal_next_op: u64,
    }

    fn projection_semantic_snapshot(
        projection: TrustedEpochProjection<'_>,
    ) -> ProjectionSemanticSnapshot {
        let frontier = projection.frontier_heads().collect::<Vec<_>>();
        let causal_next_op = projection.causal_next_op;
        let (accepted, dependencies, states, writers) = projection.into_accepted_state_parts();
        ProjectionSemanticSnapshot {
            accepted: accepted.into_iter().collect(),
            dependencies: dependencies
                .into_iter()
                .map(|(hash, values)| (hash, values.into_iter().collect()))
                .collect(),
            states: states
                .into_iter()
                .map(|(actor, state)| {
                    (
                        actor,
                        state.last_sequence,
                        state.next_op,
                        state.highest_change,
                    )
                })
                .collect(),
            writers: writers.into_iter().collect(),
            frontier,
            causal_next_op,
        }
    }

    #[test]
    fn projection_complete_construction_matrix_matches_the_semantic_oracle() {
        let mut singleton = candidate(1, 1, 1, 1);
        singleton.change_hash = ChangeHash::from_bytes([1; 32]);

        let mut empty_change = candidate(1, 1, 1, 0);
        empty_change.change_hash = ChangeHash::from_bytes([10; 32]);

        let many_actors = (1_u8..=16)
            .map(|actor| {
                let mut value = candidate(actor, 1, 1, 1);
                value.change_hash = ChangeHash::from_bytes([actor; 32]);
                value
            })
            .collect::<Vec<_>>();

        let mut deep_first = candidate(1, 1, 1, 1);
        deep_first.change_hash = ChangeHash::from_bytes([21; 32]);
        let mut deep_second = candidate(2, 1, 2, 1);
        deep_second.change_hash = ChangeHash::from_bytes([22; 32]);
        deep_second.dependencies = vec![deep_first.change_hash].into();
        let mut deep_third = candidate(1, 2, 3, 1);
        deep_third.change_hash = ChangeHash::from_bytes([23; 32]);
        deep_third.dependencies = vec![deep_second.change_hash].into();
        let mut deep_fourth = candidate(3, 1, 4, 1);
        deep_fourth.change_hash = ChangeHash::from_bytes([24; 32]);
        deep_fourth.dependencies = vec![deep_third.change_hash].into();
        let deep_chain = vec![
            deep_first.clone(),
            deep_second,
            deep_third,
            deep_fourth.clone(),
        ];

        let mut fork_root = candidate(1, 1, 1, 1);
        fork_root.change_hash = ChangeHash::from_bytes([31; 32]);
        let mut fork_left = candidate(2, 1, 2, 1);
        fork_left.change_hash = ChangeHash::from_bytes([32; 32]);
        fork_left.dependencies = vec![fork_root.change_hash].into();
        let mut fork_right = candidate(3, 1, 2, 1);
        fork_right.change_hash = ChangeHash::from_bytes([33; 32]);
        fork_right.dependencies = vec![fork_root.change_hash].into();
        let fork = vec![fork_root, fork_left.clone(), fork_right.clone()];

        let mut maximum = candidate(1, 1, 1, u64::MAX - 1);
        maximum.change_hash = ChangeHash::from_bytes([40; 32]);

        let mut accepted = candidate(1, 1, 1, 1);
        accepted.change_hash = ChangeHash::from_bytes([50; 32]);
        let mut nonaccepted = candidate(1, 99, 99, 1);
        nonaccepted.change_hash = ChangeHash::from_bytes([51; 32]);

        let cases = [
            ("empty_history", Vec::new(), Vec::new(), 1_u64, Vec::new()),
            (
                "singleton",
                vec![singleton.clone()],
                vec![singleton.change_hash],
                2,
                vec![singleton.change_hash],
            ),
            (
                "empty_change",
                vec![empty_change.clone()],
                vec![empty_change.change_hash],
                1,
                vec![empty_change.change_hash],
            ),
            (
                "many_actors",
                many_actors.clone(),
                many_actors.iter().map(|value| value.change_hash).collect(),
                2,
                many_actors.iter().map(|value| value.change_hash).collect(),
            ),
            (
                "deep_chain",
                deep_chain.clone(),
                deep_chain.iter().map(|value| value.change_hash).collect(),
                5,
                vec![deep_fourth.change_hash],
            ),
            (
                "fork",
                fork.clone(),
                fork.iter().map(|value| value.change_hash).collect(),
                3,
                vec![fork_left.change_hash, fork_right.change_hash],
            ),
            (
                "maximum_counter",
                vec![maximum.clone()],
                vec![maximum.change_hash],
                u64::MAX,
                vec![maximum.change_hash],
            ),
            (
                "accepted_nonaccepted_mixture",
                vec![accepted.clone(), nonaccepted],
                vec![accepted.change_hash],
                2,
                vec![accepted.change_hash],
            ),
        ];

        for (name, candidates, accepted_hashes, expected_next, expected_frontier) in cases {
            let accepted_closure = accepted_hashes.into_iter().collect::<BTreeSet<_>>();
            let mut snapshots = Vec::new();
            for order in [
                candidates.clone(),
                candidates.iter().cloned().rev().collect(),
                {
                    let mut rotated = candidates.clone();
                    if !rotated.is_empty() {
                        rotated.rotate_left(1);
                    }
                    rotated
                },
            ] {
                let changes = order
                    .iter()
                    .cloned()
                    .map(|value| (value.change_hash, value))
                    .collect::<BTreeMap<_, _>>();
                let projection =
                    initialize_actor_states_metered(&accepted_closure, &changes, |_| {
                        Ok::<_, ()>(())
                    });
                assert!(projection.is_ok(), "{name}");
                let Ok(projection) = projection else { continue };
                let metered_states = projection.actor_states.clone();
                let snapshot = projection_semantic_snapshot(projection);
                assert_eq!(snapshot.causal_next_op, expected_next, "{name}");
                assert_eq!(snapshot.frontier, expected_frontier, "{name}");
                let predecessor = initialize_actor_states(
                    order
                        .into_iter()
                        .filter(|value| accepted_closure.contains(&value.change_hash)),
                );
                assert!(predecessor.is_ok(), "{name}");
                let Ok(predecessor) = predecessor else {
                    continue;
                };
                assert_eq!(
                    actor_state_bytes(&metered_states),
                    actor_state_bytes(&predecessor),
                    "{name}"
                );
                snapshots.push(snapshot);
            }
            assert!(
                snapshots.windows(2).all(|pair| pair[0] == pair[1]),
                "{name}"
            );
        }

        let mut overflow = candidate(1, 1, 1, u64::MAX);
        overflow.change_hash = ChangeHash::from_bytes([60; 32]);

        let mut missing = candidate(1, 1, 1, 1);
        missing.change_hash = ChangeHash::from_bytes([61; 32]);
        missing.dependencies = vec![ChangeHash::from_bytes([62; 32])].into();

        let mut duplicate_root = candidate(1, 1, 1, 1);
        duplicate_root.change_hash = ChangeHash::from_bytes([63; 32]);
        let mut duplicate = candidate(2, 1, 2, 1);
        duplicate.change_hash = ChangeHash::from_bytes([64; 32]);
        duplicate.dependencies =
            vec![duplicate_root.change_hash, duplicate_root.change_hash].into();

        let mut low = candidate(1, 1, 1, 1);
        low.change_hash = ChangeHash::from_bytes([65; 32]);
        let mut high = candidate(2, 1, 1, 1);
        high.change_hash = ChangeHash::from_bytes([66; 32]);
        let mut noncanonical = candidate(3, 1, 2, 1);
        noncanonical.change_hash = ChangeHash::from_bytes([67; 32]);
        noncanonical.dependencies = vec![high.change_hash, low.change_hash].into();

        for (name, candidates, accepted_hashes, expected) in [
            (
                "overflow",
                vec![overflow.clone()],
                vec![overflow.change_hash],
                ActorStateError::OperationCounter,
            ),
            (
                "missing_dependency",
                vec![missing.clone()],
                vec![missing.change_hash],
                ActorStateError::MissingDependency,
            ),
            (
                "duplicate_dependency",
                vec![duplicate_root.clone(), duplicate.clone()],
                vec![duplicate_root.change_hash, duplicate.change_hash],
                ActorStateError::NoncanonicalInput,
            ),
            (
                "noncanonical_dependency",
                vec![low.clone(), high.clone(), noncanonical.clone()],
                vec![low.change_hash, high.change_hash, noncanonical.change_hash],
                ActorStateError::NoncanonicalInput,
            ),
        ] {
            let closure = accepted_hashes.into_iter().collect::<BTreeSet<_>>();
            for order in [candidates.clone(), candidates.into_iter().rev().collect()] {
                let changes = order
                    .into_iter()
                    .map(|value| (value.change_hash, value))
                    .collect::<BTreeMap<_, _>>();
                assert!(
                    matches!(
                        initialize_actor_states_metered(&closure, &changes, |_| Ok::<_, ()>(())),
                        Err(MeteredActorStateError::State(actual)) if actual == expected
                    ),
                    "{name}"
                );
            }
        }
    }

    #[test]
    fn projected_actor_sequence_decision_is_nonmutating_and_complete() {
        let genesis = candidate(1, 1, 1, 1);
        let empty_closure = BTreeSet::new();
        let empty_changes = BTreeMap::new();
        let empty =
            initialize_actor_states_metered(&empty_closure, &empty_changes, |_| Ok::<_, ()>(()));
        assert!(empty.is_ok());
        let Some(empty) = empty.ok() else { return };
        assert!(
            empty
                .actor_sequence_decision_metered(&genesis, |_| Ok::<_, ()>(()))
                .is_ok()
        );

        let first = candidate(1, 1, 1, 1);
        let mut bridge = candidate(2, 1, 2, 1);
        bridge.change_hash = ChangeHash::from_bytes([2; 32]);
        bridge.dependencies = vec![first.change_hash].into();
        let closure = BTreeSet::from([first.change_hash, bridge.change_hash]);
        let changes = BTreeMap::from([
            (first.change_hash, first.clone()),
            (bridge.change_hash, bridge.clone()),
        ]);
        let projection = initialize_actor_states_metered(&closure, &changes, |_| Ok::<_, ()>(()));
        assert!(projection.is_ok());
        let Some(projection) = projection.ok() else {
            return;
        };

        let mut deep = candidate(1, 2, 3, 1);
        deep.change_hash = ChangeHash::from_bytes([3; 32]);
        deep.dependencies = vec![bridge.change_hash].into();
        let deep_view = projection.candidate_metered(&deep, |_| Ok::<_, ()>(()));
        assert!(deep_view.is_ok_and(|view| {
            view.predecessor() == Some(first.change_hash)
                && !view.predecessor_is_direct_dependency()
        }));
        assert!(
            projection
                .actor_sequence_decision_metered(&deep, |_| Ok::<_, ()>(()))
                .is_ok()
        );

        let mut independent = candidate(2, 1, 1, 1);
        independent.change_hash = ChangeHash::from_bytes([4; 32]);
        let unrelated_closure = BTreeSet::from([first.change_hash, independent.change_hash]);
        let unrelated_changes = BTreeMap::from([
            (first.change_hash, first.clone()),
            (independent.change_hash, independent.clone()),
        ]);
        let unrelated_projection =
            initialize_actor_states_metered(&unrelated_closure, &unrelated_changes, |_| {
                Ok::<_, ()>(())
            });
        assert!(unrelated_projection.is_ok());
        let Some(unrelated_projection) = unrelated_projection.ok() else {
            return;
        };
        let mut unrelated = deep.clone();
        unrelated.change_hash = ChangeHash::from_bytes([6; 32]);
        unrelated.dependencies = vec![independent.change_hash].into();
        assert!(
            unrelated_projection
                .actor_sequence_decision_metered(&unrelated, |_| Ok::<_, ()>(()))
                .is_ok()
        );

        let mut gap = deep.clone();
        gap.sequence = 3;
        assert!(matches!(
            projection.actor_sequence_decision_metered(&gap, |_| Ok::<_, ()>(())),
            Err(MeteredActorStateError::State(
                ActorStateError::MissingPredecessor
            ))
        ));
        let mut rollback = deep.clone();
        rollback.sequence = 1;
        assert!(matches!(
            projection.actor_sequence_decision_metered(&rollback, |_| Ok::<_, ()>(())),
            Err(MeteredActorStateError::State(
                ActorStateError::SequenceRollback
            ))
        ));
        let mut duplicate = first.clone();
        duplicate.change_hash = ChangeHash::from_bytes([5; 32]);
        assert!(matches!(
            projection.actor_sequence_decision_metered(&duplicate, |_| Ok::<_, ()>(())),
            Err(MeteredActorStateError::State(
                ActorStateError::SequenceRollback
            ))
        ));

        let overflow_hash = ChangeHash::from_bytes([8; 32]);
        let overflow_actor = ActorId::from_bytes([8; 32]);
        let overflow_candidate = ChangeCandidate {
            change_hash: overflow_hash,
            actor: overflow_actor,
            sequence: u64::MAX,
            start_op: 1,
            operation_count: 1,
            dependencies: Vec::new().into(),
            control_id: EventId::from_bytes([9; 32]),
            author: DevicePublicKey::from_bytes([8; 32]),
            valid_carriers: Vec::new().into(),
        };
        let overflow_branch = BTreeMap::from([(overflow_hash, overflow_candidate.clone())]);
        let overflow_closure = BTreeSet::from([overflow_hash]);
        let overflow_projection = TrustedEpochProjection {
            branch_membership: &overflow_branch,
            accepted_closure: &overflow_closure,
            dependencies: BTreeMap::from([(overflow_hash, BTreeSet::new())]),
            frontier_heads: BTreeSet::from([overflow_hash]),
            actor_states: BTreeMap::from([(
                overflow_actor,
                EpochActorState {
                    last_sequence: u64::MAX,
                    next_op: 1,
                    highest_change: overflow_hash,
                },
            )]),
            writer_contributions: BTreeMap::from([(overflow_actor, overflow_hash)]),
            causal_next_op: 1,
        };
        assert!(matches!(
            overflow_projection
                .actor_sequence_decision_metered(&overflow_candidate, |_| Ok::<_, ()>(())),
            Err(MeteredActorStateError::State(ActorStateError::SequenceGap))
        ));

        const LOOKUP_CHARGES: usize = 4;
        for successful in 0..LOOKUP_CHARGES {
            for stopped in [Completion::BudgetExhausted, Completion::Cancelled] {
                let observed = Cell::new(0_usize);
                let result = projection.actor_sequence_decision_metered(&deep, |_| {
                    if observed.get() == successful {
                        Err(stopped)
                    } else {
                        observed.set(observed.get().saturating_add(1));
                        Ok(())
                    }
                });
                assert!(matches!(
                    result,
                    Err(MeteredActorStateError::Work(actual)) if actual == stopped
                ));
                assert_eq!(observed.get(), successful);
            }
        }
        for allowance in [LOOKUP_CHARGES, LOOKUP_CHARGES + 1] {
            let observed = Cell::new(0_usize);
            let result = projection.actor_sequence_decision_metered(&deep, |_| {
                if observed.get() == allowance {
                    Err(Completion::BudgetExhausted)
                } else {
                    observed.set(observed.get().saturating_add(1));
                    Ok(())
                }
            });
            assert!(result.is_ok());
            assert_eq!(observed.get(), LOOKUP_CHARGES);
        }
    }

    #[test]
    fn actor_identity_and_sequence_relations_are_owned_immediate_and_short_circuiting() {
        let first = candidate(1, 1, 1, 1);
        let closure = BTreeSet::from([first.change_hash]);
        let changes = BTreeMap::from([(first.change_hash, first.clone())]);
        let projection = initialize_actor_states_metered(&closure, &changes, |_| Ok::<_, ()>(()));
        assert!(projection.is_ok());
        let Some(projection) = projection.ok() else {
            return;
        };
        let mut next = candidate(1, 2, 2, 1);
        next.change_hash = ChangeHash::from_bytes([2; 32]);
        next.dependencies = vec![first.change_hash].into();
        let expected = [
            ActorDecisionSite::ActorStateRead,
            ActorDecisionSite::PredecessorCandidateRead,
            ActorDecisionSite::ActorIdentityDecision,
            ActorDecisionSite::SequenceRelationDecision,
        ]
        .into_iter()
        .flat_map(|site| {
            let descriptor = site.descriptor();
            [
                ActorDecisionTrace::Attempt(descriptor),
                ActorDecisionTrace::Charge(WorkCounter::GraphNode),
                ActorDecisionTrace::Operation(descriptor),
            ]
        })
        .collect::<Vec<_>>();
        let (complete, trace) =
            observed_actor_sequence(&projection, &next, usize::MAX, Completion::BudgetExhausted);
        assert_eq!(complete, Ok(()));
        assert_eq!(trace, expected);

        for successful in 0..4 {
            for stopped in [Completion::BudgetExhausted, Completion::Cancelled] {
                let (result, trace) =
                    observed_actor_sequence(&projection, &next, successful, stopped);
                assert_eq!(result, Err(MeteredActorStateError::Work(stopped)));
                assert_eq!(
                    trace
                        .iter()
                        .filter(|entry| matches!(entry, ActorDecisionTrace::Operation(_)))
                        .count(),
                    successful
                );
            }
        }
        for allowance in [4, 5] {
            let (result, trace) =
                observed_actor_sequence(&projection, &next, allowance, Completion::BudgetExhausted);
            assert_eq!(result, Ok(()));
            assert_eq!(trace, expected);
        }

        let wrong_actor = candidate(2, 2, 2, 1);
        let mut invalid_projection = focused_causal_consumer_projection(&changes, &closure, 2);
        invalid_projection.actor_states = BTreeMap::from([(
            wrong_actor.actor,
            EpochActorState {
                last_sequence: 1,
                next_op: 2,
                highest_change: first.change_hash,
            },
        )]);
        let (invalid, invalid_trace) = observed_actor_sequence(
            &invalid_projection,
            &wrong_actor,
            usize::MAX,
            Completion::BudgetExhausted,
        );
        assert_eq!(
            invalid,
            Err(MeteredActorStateError::State(
                ActorStateError::MissingPredecessor
            ))
        );
        assert_eq!(
            invalid_trace.last(),
            Some(&ActorDecisionTrace::Operation(
                ActorDecisionSite::ActorIdentityDecision.descriptor()
            ))
        );
        assert!(!invalid_trace.iter().any(|entry| matches!(
            entry,
            ActorDecisionTrace::Operation(descriptor)
                if descriptor.site == ActorDecisionSite::SequenceRelationDecision
        )));

        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        struct Injected;
        let injected = Injected;
        let result = projection.actor_sequence_decision_metered(&next, |_| Err(&injected));
        assert!(matches!(
            result,
            Err(MeteredActorStateError::Work(error)) if core::ptr::eq(error, &injected)
        ));
    }

    #[test]
    fn projected_causal_next_decision_is_checked_constant_size_and_exactly_metered() {
        let empty_closure = BTreeSet::new();
        let empty_changes = BTreeMap::new();
        let empty =
            initialize_actor_states_metered(&empty_closure, &empty_changes, |_| Ok::<_, ()>(()));
        assert!(empty.is_ok());
        let Some(empty) = empty.ok() else { return };
        let empty_change = candidate(1, 1, 1, 0);
        assert_eq!(
            empty.causal_next_decision_metered(&empty_change, |_| Ok::<_, ()>(())),
            Ok(1)
        );

        let many = (1_u8..=64)
            .map(|actor| {
                let mut change = candidate(actor, 1, 1, u64::from(actor));
                change.change_hash = ChangeHash::from_bytes([actor; 32]);
                change
            })
            .collect::<Vec<_>>();
        let many_closure = many
            .iter()
            .map(|candidate| candidate.change_hash)
            .collect::<BTreeSet<_>>();
        let many_changes = many
            .into_iter()
            .map(|candidate| (candidate.change_hash, candidate))
            .collect::<BTreeMap<_, _>>();
        let many_projection =
            initialize_actor_states_metered(&many_closure, &many_changes, |_| Ok::<_, ()>(()));
        assert!(many_projection.is_ok());
        let Some(many_projection) = many_projection.ok() else {
            return;
        };
        let mut next = candidate(100, 1, 65, 1);
        next.change_hash = ChangeHash::from_bytes([100; 32]);
        let states_before = many_projection.actor_states.clone();
        assert_eq!(
            many_projection.causal_next_decision_metered(&next, |_| Ok::<_, ()>(())),
            Ok(66)
        );
        assert_eq!(many_projection.actor_states, states_before);
        let mut legacy_states = states_before;
        assert_eq!(
            reference_apply_nonempty_counter(&mut legacy_states, &next),
            Ok(())
        );
        assert_eq!(legacy_states[&next.actor].next_op, 66);

        let mut gap = next.clone();
        gap.start_op = 66;
        assert_eq!(
            many_projection.causal_next_decision_metered(&gap, |_| Ok::<_, ()>(())),
            Err(MeteredActorStateError::State(
                ActorStateError::OperationCounter
            ))
        );
        let mut duplicate = next.clone();
        duplicate.start_op = 64;
        assert_eq!(
            many_projection.causal_next_decision_metered(&duplicate, |_| Ok::<_, ()>(())),
            Err(MeteredActorStateError::State(
                ActorStateError::OperationCounter
            ))
        );

        let max_actor = ActorId::from_bytes([200; 32]);
        let max_hash = ChangeHash::from_bytes([200; 32]);
        let max_branch = BTreeMap::new();
        let max_closure = BTreeSet::new();
        let max_projection = TrustedEpochProjection {
            branch_membership: &max_branch,
            accepted_closure: &max_closure,
            dependencies: BTreeMap::new(),
            frontier_heads: BTreeSet::new(),
            actor_states: BTreeMap::from([(
                max_actor,
                EpochActorState {
                    last_sequence: 1,
                    next_op: u64::MAX,
                    highest_change: max_hash,
                },
            )]),
            writer_contributions: BTreeMap::from([(max_actor, max_hash)]),
            causal_next_op: u64::MAX,
        };
        let mut max_empty = candidate(201, 1, u64::MAX, 0);
        max_empty.change_hash = ChangeHash::from_bytes([201; 32]);
        assert_eq!(
            max_projection.causal_next_decision_metered(&max_empty, |_| Ok::<_, ()>(())),
            Ok(u64::MAX)
        );
        let mut overflow = max_empty.clone();
        overflow.operation_count = 1;
        assert_eq!(
            max_projection.causal_next_decision_metered(&overflow, |_| Ok::<_, ()>(())),
            Err(MeteredActorStateError::State(
                ActorStateError::OperationCounter
            ))
        );

        let expected_trace = [
            CausalNextSite::StoredCounterRead,
            CausalNextSite::ExpectedStartComparison,
            CausalNextSite::CheckedAdvance,
        ]
        .into_iter()
        .flat_map(|site| {
            let descriptor = site.descriptor();
            [
                CausalNextTrace::Attempt(descriptor),
                CausalNextTrace::Charge(WorkCounter::GraphNode),
                CausalNextTrace::Operation(descriptor),
            ]
        })
        .collect::<Vec<_>>();
        let (ample, ample_trace) = observed_causal_next(
            &many_projection,
            &next,
            usize::MAX,
            Completion::BudgetExhausted,
        );
        assert_eq!(ample, Ok(66));
        assert_eq!(ample_trace, expected_trace);

        const DECISION_CHARGES: usize = 3;
        for successful in 0..DECISION_CHARGES {
            for stopped in [Completion::BudgetExhausted, Completion::Cancelled] {
                let (result, trace) =
                    observed_causal_next(&many_projection, &next, successful, stopped);
                assert_eq!(result, Err(MeteredActorStateError::Work(stopped)));
                assert_eq!(
                    trace
                        .iter()
                        .filter(|entry| matches!(entry, CausalNextTrace::Operation(_)))
                        .count(),
                    successful
                );
            }
        }
        for allowance in [DECISION_CHARGES, DECISION_CHARGES + 1] {
            let (result, trace) = observed_causal_next(
                &many_projection,
                &next,
                allowance,
                Completion::BudgetExhausted,
            );
            assert_eq!(result, Ok(66));
            assert_eq!(trace, expected_trace);
        }
    }

    fn focused_causal_consumer_projection<'a>(
        branch_membership: &'a BTreeMap<ChangeHash, ChangeCandidate>,
        accepted_closure: &'a BTreeSet<ChangeHash>,
        causal_next_op: u64,
    ) -> TrustedEpochProjection<'a> {
        TrustedEpochProjection {
            branch_membership,
            accepted_closure,
            dependencies: BTreeMap::new(),
            frontier_heads: BTreeSet::new(),
            actor_states: BTreeMap::new(),
            writer_contributions: BTreeMap::new(),
            causal_next_op,
        }
    }

    #[test]
    fn causal_consumer_stored_counter_read_is_owned() {
        assert_causal_consumer_family_exact(CausalNextOperation::StoredCounterRead);
        let branch = BTreeMap::new();
        let closure = BTreeSet::new();
        let projection = focused_causal_consumer_projection(&branch, &closure, 7);
        let candidate = candidate(1, 1, 7, 1);
        let (blocked, blocked_trace) =
            observed_causal_next(&projection, &candidate, 0, Completion::BudgetExhausted);
        assert_eq!(
            blocked,
            Err(MeteredActorStateError::Work(Completion::BudgetExhausted))
        );
        assert_eq!(
            blocked_trace,
            [
                CausalNextTrace::Attempt(CausalNextSite::StoredCounterRead.descriptor()),
                CausalNextTrace::Charge(WorkCounter::GraphNode),
            ]
        );

        let (admitted, admitted_trace) =
            observed_causal_next(&projection, &candidate, 1, Completion::Cancelled);
        assert_eq!(
            admitted,
            Err(MeteredActorStateError::Work(Completion::Cancelled))
        );
        assert_eq!(
            admitted_trace,
            [
                CausalNextTrace::Attempt(CausalNextSite::StoredCounterRead.descriptor()),
                CausalNextTrace::Charge(WorkCounter::GraphNode),
                CausalNextTrace::Operation(CausalNextSite::StoredCounterRead.descriptor()),
                CausalNextTrace::Attempt(CausalNextSite::ExpectedStartComparison.descriptor()),
                CausalNextTrace::Charge(WorkCounter::GraphNode),
            ]
        );
    }

    #[test]
    fn causal_consumer_expected_start_comparison_is_owned() {
        assert_causal_consumer_family_exact(CausalNextOperation::ExpectedStartComparison);
        let branch = BTreeMap::new();
        let closure = BTreeSet::new();
        let projection = focused_causal_consumer_projection(&branch, &closure, 7);
        let mut candidate = candidate(1, 1, 7, 1);
        candidate.start_op = 8;
        let (blocked, blocked_trace) =
            observed_causal_next(&projection, &candidate, 1, Completion::BudgetExhausted);
        assert_eq!(
            blocked,
            Err(MeteredActorStateError::Work(Completion::BudgetExhausted))
        );
        assert_eq!(
            blocked_trace
                .iter()
                .filter(|entry| matches!(entry, CausalNextTrace::Operation(_)))
                .count(),
            1
        );

        let (admitted, admitted_trace) =
            observed_causal_next(&projection, &candidate, 2, Completion::Cancelled);
        assert_eq!(
            admitted,
            Err(MeteredActorStateError::State(
                ActorStateError::OperationCounter
            ))
        );
        assert_eq!(
            admitted_trace
                .iter()
                .filter(|entry| matches!(entry, CausalNextTrace::Operation(_)))
                .count(),
            2
        );
        assert!(matches!(
            admitted_trace.last(),
            Some(CausalNextTrace::Operation(descriptor))
                if descriptor.site == CausalNextSite::ExpectedStartComparison
        ));
    }

    #[test]
    fn causal_consumer_checked_advance_is_owned() {
        assert_causal_consumer_family_exact(CausalNextOperation::CheckedAdvance);
        let branch = BTreeMap::new();
        let closure = BTreeSet::new();
        let projection = focused_causal_consumer_projection(&branch, &closure, 7);
        let candidate = candidate(1, 1, 7, 1);
        let (blocked, blocked_trace) =
            observed_causal_next(&projection, &candidate, 2, Completion::Cancelled);
        assert_eq!(
            blocked,
            Err(MeteredActorStateError::Work(Completion::Cancelled))
        );
        assert_eq!(
            blocked_trace
                .iter()
                .filter(|entry| matches!(entry, CausalNextTrace::Operation(_)))
                .count(),
            2
        );

        let (admitted, admitted_trace) =
            observed_causal_next(&projection, &candidate, 3, Completion::BudgetExhausted);
        assert_eq!(admitted, Ok(8));
        assert_eq!(
            admitted_trace.last(),
            Some(&CausalNextTrace::Operation(
                CausalNextSite::CheckedAdvance.descriptor()
            ))
        );
    }

    #[test]
    fn empty_frontier_comparison_is_streaming_exact_and_immediately_metered() {
        let projected_first = ChangeHash::from_bytes([10; 32]);
        let base_only = ChangeHash::from_bytes([20; 32]);
        let projected_last = ChangeHash::from_bytes([30; 32]);
        let branch = BTreeMap::new();
        let accepted = BTreeSet::from([projected_first, projected_last]);
        let projection = TrustedEpochProjection {
            branch_membership: &branch,
            accepted_closure: &accepted,
            dependencies: BTreeMap::new(),
            frontier_heads: BTreeSet::from([projected_first, projected_last]),
            actor_states: BTreeMap::new(),
            writer_contributions: BTreeMap::new(),
            causal_next_op: 1,
        };
        let base_frontier = BTreeSet::from([projected_first, base_only]);
        let mut exact = candidate(1, 1, 1, 0);
        exact.dependencies = vec![projected_first, base_only, projected_last].into();
        assert!(
            projection
                .empty_frontier_decision_metered(&exact, &base_frontier, |_| Ok::<_, ()>(()))
                .is_ok()
        );

        let empty_branch = BTreeMap::new();
        let empty_accepted = BTreeSet::new();
        let empty_projection = TrustedEpochProjection {
            branch_membership: &empty_branch,
            accepted_closure: &empty_accepted,
            dependencies: BTreeMap::new(),
            frontier_heads: BTreeSet::new(),
            actor_states: BTreeMap::new(),
            writer_contributions: BTreeMap::new(),
            causal_next_op: 1,
        };
        let empty_base = BTreeSet::new();
        let empty = candidate(1, 1, 1, 0);
        assert!(
            empty_projection
                .empty_frontier_decision_metered(&empty, &empty_base, |_| Ok::<_, ()>(()))
                .is_ok()
        );

        let mut nonempty = exact.clone();
        nonempty.operation_count = 1;
        nonempty.dependencies = vec![ChangeHash::from_bytes([99; 32])].into();
        let (nonempty_result, nonempty_trace) = observed_empty_frontier(
            &projection,
            &nonempty,
            &base_frontier,
            usize::MAX,
            Completion::BudgetExhausted,
        );
        assert_eq!(nonempty_result, Ok(()));
        assert_eq!(
            nonempty_trace,
            vec![
                FrontierTrace::Charge(WorkCounter::GraphNode),
                FrontierTrace::Operation(FrontierComparisonOperation::CandidateKindComparison),
            ]
        );

        for dependencies in [
            vec![projected_first, base_only],
            vec![
                projected_first,
                base_only,
                ChangeHash::from_bytes([25; 32]),
                projected_last,
            ],
            vec![projected_first, base_only, base_only, projected_last],
            vec![projected_first, projected_last, base_only],
        ] {
            let mut malformed = exact.clone();
            malformed.dependencies = dependencies.into();
            assert_eq!(
                projection.empty_frontier_decision_metered(&malformed, &base_frontier, |_| Ok::<
                    _,
                    (),
                >(
                    ()
                )),
                Err(MeteredActorStateError::State(
                    ActorStateError::DependencyFrontier
                ))
            );
        }

        let (ample, trace) = observed_empty_frontier(
            &projection,
            &exact,
            &base_frontier,
            usize::MAX,
            Completion::BudgetExhausted,
        );
        assert_eq!(ample, Ok(()));
        assert!(trace.chunks_exact(2).all(|pair| {
            matches!(pair[0], FrontierTrace::Charge(_))
                && matches!(pair[1], FrontierTrace::Operation(_))
        }));
        let operation_count = trace.len() / 2;
        assert!(operation_count > 3);

        for successful in 0..operation_count {
            for stopped in [Completion::BudgetExhausted, Completion::Cancelled] {
                let (result, stopped_trace) = observed_empty_frontier(
                    &projection,
                    &exact,
                    &base_frontier,
                    successful,
                    stopped,
                );
                assert_eq!(result, Err(MeteredActorStateError::Work(stopped)));
                assert_eq!(
                    stopped_trace
                        .iter()
                        .filter(|entry| matches!(entry, FrontierTrace::Operation(_)))
                        .count(),
                    successful
                );
            }
        }
        for allowance in [operation_count, operation_count + 1] {
            let (result, allowed_trace) = observed_empty_frontier(
                &projection,
                &exact,
                &base_frontier,
                allowance,
                Completion::BudgetExhausted,
            );
            assert_eq!(result, Ok(()));
            assert_eq!(allowed_trace, trace);
        }

        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        struct Injected;
        let injected = Injected;
        let result =
            projection.empty_frontier_decision_metered(&exact, &base_frontier, |_| Err(&injected));
        assert!(matches!(
            result,
            Err(MeteredActorStateError::Work(error)) if core::ptr::eq(error, &injected)
        ));

        let source = include_str!("actor_state.rs");
        let method = source
            .split_once("fn empty_frontier_decision_metered_observed")
            .map(|item| item.1)
            .and_then(|body| body.split_once("pub(crate) fn into_accepted_state_parts"))
            .map_or("", |item| item.0);
        assert!(!method.contains(".collect::<"));
        assert!(!method.contains(".clone()"));
        assert!(!method.contains(".sort"));
        assert!(!method.contains(".dedup"));
    }

    #[test]
    fn complete_candidate_semantics_preserve_precedence_and_every_stop_boundary() {
        let first = candidate(1, 1, 1, 1);
        let closure = BTreeSet::from([first.change_hash]);
        let changes = BTreeMap::from([(first.change_hash, first.clone())]);
        let projection = initialize_actor_states_metered(&closure, &changes, |_| Ok::<_, ()>(()));
        assert!(projection.is_ok());
        let Some(projection) = projection.ok() else {
            return;
        };
        let base_frontier = BTreeSet::new();
        let mut valid = candidate(1, 2, 2, 0);
        valid.change_hash = ChangeHash::from_bytes([2; 32]);
        valid.dependencies = vec![first.change_hash].into();

        let charges = Cell::new(0_usize);
        let mut completed = Vec::new();
        let ample = projection.candidate_semantics_decision_metered_observed(
            &valid,
            &base_frontier,
            |_| {
                charges.set(charges.get().saturating_add(1));
                Ok::<_, Completion>(())
            },
            |stage| completed.push((stage, charges.get())),
        );
        assert_eq!(ample, Ok(()));
        assert_eq!(
            completed
                .iter()
                .map(|(stage, _)| *stage)
                .collect::<Vec<_>>(),
            vec![
                CandidateSemanticStage::ActorSequence,
                CandidateSemanticStage::CausalCounter,
                CandidateSemanticStage::EmptyFrontier,
            ]
        );
        assert!(completed.windows(2).all(|pair| pair[0].1 < pair[1].1));
        assert_eq!(
            completed.last().map(|(_, count)| *count),
            Some(charges.get())
        );

        for successful in 0..charges.get() {
            for stopped in [Completion::BudgetExhausted, Completion::Cancelled] {
                let observed = Cell::new(0_usize);
                let mut stopped_stages = Vec::new();
                let result = projection.candidate_semantics_decision_metered_observed(
                    &valid,
                    &base_frontier,
                    |_| {
                        if observed.get() == successful {
                            Err(stopped)
                        } else {
                            observed.set(observed.get().saturating_add(1));
                            Ok(())
                        }
                    },
                    |stage| stopped_stages.push(stage),
                );
                assert_eq!(result, Err(MeteredActorStateError::Work(stopped)));
                assert_eq!(observed.get(), successful);
                assert_eq!(
                    stopped_stages,
                    completed
                        .iter()
                        .take_while(|(_, boundary)| *boundary <= successful)
                        .map(|(stage, _)| *stage)
                        .collect::<Vec<_>>()
                );
            }
        }
        for allowance in [charges.get(), charges.get().saturating_add(1)] {
            let observed = Cell::new(0_usize);
            let result =
                projection.candidate_semantics_decision_metered(&valid, &base_frontier, |_| {
                    if observed.get() == allowance {
                        Err(Completion::BudgetExhausted)
                    } else {
                        observed.set(observed.get().saturating_add(1));
                        Ok(())
                    }
                });
            assert_eq!(result, Ok(()));
            assert_eq!(observed.get(), charges.get());
        }

        let mut actor_invalid = valid.clone();
        actor_invalid.sequence = 3;
        actor_invalid.start_op = 9;
        actor_invalid.dependencies = Vec::new().into();
        assert_eq!(
            projection.candidate_semantics_decision_metered(
                &actor_invalid,
                &base_frontier,
                |_| Ok::<_, Completion>(())
            ),
            Err(MeteredActorStateError::State(
                ActorStateError::MissingPredecessor
            ))
        );

        let mut counter_invalid = valid.clone();
        counter_invalid.start_op = 9;
        counter_invalid.dependencies = Vec::new().into();
        assert_eq!(
            projection.candidate_semantics_decision_metered(
                &counter_invalid,
                &base_frontier,
                |_| Ok::<_, Completion>(())
            ),
            Err(MeteredActorStateError::State(
                ActorStateError::OperationCounter
            ))
        );

        let mut frontier_invalid = valid;
        frontier_invalid.dependencies = Vec::new().into();
        assert_eq!(
            projection.candidate_semantics_decision_metered(
                &frontier_invalid,
                &base_frontier,
                |_| Ok::<_, Completion>(())
            ),
            Err(MeteredActorStateError::State(
                ActorStateError::DependencyFrontier
            ))
        );

        #[derive(Debug, PartialEq, Eq)]
        struct Injected;
        let injected = Injected;
        assert!(matches!(
            projection.candidate_semantics_decision_metered(
                &frontier_invalid,
                &base_frontier,
                |_| Err(&injected)
            ),
            Err(MeteredActorStateError::Work(error)) if core::ptr::eq(error, &injected)
        ));
    }

    #[test]
    fn v16_actor_relation_is_not_classified_outside_owned_stage() {
        let source = include_str!("actor_state.rs");
        let method = source
            .split_once("pub(crate) fn actor_sequence_decision_metered")
            .map(|item| item.1)
            .and_then(|body| body.split_once("pub(crate) fn causal_next_decision_metered"))
            .map_or("", |item| item.0);
        assert!(!method.contains("match view.predecessor()"));
        assert!(!method.contains("candidate.sequence < view.expected_sequence()"));
        assert!(!method.contains("!view.sequence_matches()"));
    }

    #[test]
    fn v16_actor_failure_performs_zero_causal_counter_work() {
        let first = candidate(1, 1, 1, 1);
        let closure = BTreeSet::from([first.change_hash]);
        let changes = BTreeMap::from([(first.change_hash, first.clone())]);
        let projection = initialize_actor_states_metered(&closure, &changes, |_| Ok::<_, ()>(()));
        let Some(projection) = projection.ok() else {
            return;
        };
        let mut invalid = candidate(1, 3, 99, 1);
        invalid.change_hash = ChangeHash::from_bytes([3; 32]);
        invalid.dependencies = Vec::new().into();

        for stopped in [Completion::BudgetExhausted, Completion::Cancelled] {
            let successful = Cell::new(0_usize);
            let result = projection.actor_sequence_decision_metered(&invalid, |_| {
                if successful.get() == 8 {
                    Err(stopped)
                } else {
                    successful.set(successful.get().saturating_add(1));
                    Ok(())
                }
            });
            assert_eq!(
                result,
                Err(MeteredActorStateError::State(
                    ActorStateError::MissingPredecessor
                ))
            );
            assert!(successful.get() < 8);
        }
        let mut stages = Vec::new();
        let result = projection.candidate_semantics_decision_metered_observed(
            &invalid,
            &BTreeSet::new(),
            |_| Ok::<_, Completion>(()),
            |stage| stages.push(stage),
        );
        assert_eq!(
            result,
            Err(MeteredActorStateError::State(
                ActorStateError::MissingPredecessor
            ))
        );
        assert!(stages.is_empty());
    }

    #[test]
    fn v16_candidate_start_counter_is_compared_once() {
        let source = include_str!("actor_state.rs");
        let production = source
            .split_once("#[cfg(test)]\npub(crate) mod tests")
            .map_or(source, |item| item.0);
        let actor = production
            .split_once("fn actor_sequence_decision_metered_observed")
            .map(|item| item.1)
            .and_then(|body| body.split_once("pub(crate) fn causal_next_decision_metered"))
            .map_or("", |item| item.0);
        let causal = production
            .split_once("fn causal_next_decision_metered_observed")
            .map(|item| item.1)
            .and_then(|body| body.split_once("pub(crate) fn dependencies"))
            .map_or("", |item| item.0);
        assert_eq!(
            actor.matches("candidate.start_op ==").count()
                + causal.matches("candidate.start_op ==").count(),
            1
        );
    }

    #[test]
    fn v16_actor_semantic_comparison_has_an_immediate_charge_boundary() {
        let source = include_str!("actor_state.rs");
        let method = source
            .split_once("pub(crate) fn actor_sequence_decision_metered")
            .map(|item| item.1)
            .and_then(|body| body.split_once("pub(crate) fn causal_next_decision_metered"))
            .map_or("", |item| item.0);
        assert!(method.contains("ActorIdentityDecision"));
        assert!(method.contains("SequenceRelationDecision"));
        assert!(!method.contains("let view = self.candidate_metered"));
    }

    #[test]
    fn projection_lookups_and_semantic_comparisons_are_immediately_charged() {
        let first = candidate(1, 1, 1, 1);
        let mut second = candidate(1, 2, 2, 1);
        second.change_hash = ChangeHash::from_bytes([2; 32]);
        second.dependencies = vec![first.change_hash].into();
        let closure = BTreeSet::from([first.change_hash, second.change_hash]);
        let changes = BTreeMap::from([
            (first.change_hash, first),
            (second.change_hash, second.clone()),
        ]);
        let projection = initialize_actor_states_metered(&closure, &changes, |_| Ok::<_, ()>(()));
        assert!(projection.is_ok(), "canonical projection");
        let Some(projection) = projection.ok() else {
            return;
        };
        let mut query = candidate(1, 3, 3, 1);
        query.change_hash = ChangeHash::from_bytes([3; 32]);
        query.dependencies = vec![second.change_hash].into();

        let (ample, trace) =
            observed_candidate_lookup(&projection, &query, usize::MAX, Completion::BudgetExhausted);
        assert!(ample.is_ok(), "ample lookup");
        let Some(view) = ample.ok() else {
            return;
        };
        assert!(!view.is_branch_member());
        assert!(!view.is_accepted_member());
        assert_eq!(view.predecessor(), Some(second.change_hash));
        assert!(view.predecessor_is_direct_dependency());
        assert!(view.actor_identity_matches());
        assert_eq!(view.expected_sequence(), 3);
        assert!(view.sequence_matches());
        assert_eq!(view.causal_next_op(), 3);
        assert!(view.expected_next_matches());

        let operations = [
            (
                WorkCounter::GraphNode,
                ProjectionLookupOperation::BranchMembership,
            ),
            (
                WorkCounter::GraphNode,
                ProjectionLookupOperation::AcceptedMembership,
            ),
            (
                WorkCounter::GraphNode,
                ProjectionLookupOperation::ActorState,
            ),
            (
                WorkCounter::GraphEdge,
                ProjectionLookupOperation::DirectDependency,
            ),
            (
                WorkCounter::GraphNode,
                ProjectionLookupOperation::PredecessorCandidate,
            ),
            (
                WorkCounter::GraphNode,
                ProjectionLookupOperation::ActorIdentityComparison,
            ),
            (
                WorkCounter::GraphNode,
                ProjectionLookupOperation::ExpectedSequence,
            ),
            (
                WorkCounter::GraphNode,
                ProjectionLookupOperation::SequenceComparison,
            ),
            (
                WorkCounter::GraphNode,
                ProjectionLookupOperation::ExpectedNextComparison,
            ),
        ];
        let expected = operations
            .iter()
            .flat_map(|(counter, operation)| {
                [
                    LookupTrace::Charge(*counter),
                    LookupTrace::Operation(*operation),
                ]
            })
            .collect::<Vec<_>>();
        assert_eq!(trace, expected);

        for target in 1..=operations.len() {
            let count_operations = |trace: &[LookupTrace]| {
                trace
                    .iter()
                    .filter(|entry| matches!(entry, LookupTrace::Operation(_)))
                    .count()
            };
            let (before, before_trace) = observed_candidate_lookup(
                &projection,
                &query,
                target - 1,
                Completion::BudgetExhausted,
            );
            assert_eq!(
                before,
                Err(MeteredActorStateError::Work(Completion::BudgetExhausted))
            );
            assert_eq!(count_operations(&before_trace), target - 1);

            for allowance in [target, target + 1] {
                let (_, allowed_trace) = observed_candidate_lookup(
                    &projection,
                    &query,
                    allowance,
                    Completion::BudgetExhausted,
                );
                assert!(count_operations(&allowed_trace) >= target);
            }

            let (cancelled, cancelled_trace) =
                observed_candidate_lookup(&projection, &query, target - 1, Completion::Cancelled);
            assert_eq!(
                cancelled,
                Err(MeteredActorStateError::Work(Completion::Cancelled))
            );
            assert_eq!(count_operations(&cancelled_trace), target - 1);
        }

        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        struct Injected;
        let injected = Injected;
        let result = projection.candidate_metered(&query, |_| Err(&injected));
        assert!(matches!(
            result,
            Err(MeteredActorStateError::Work(error)) if core::ptr::eq(error, &injected)
        ));
    }

    fn projection_work_contract_run<'a>(
        accepted_closure: &'a BTreeSet<ChangeHash>,
        changes: &'a BTreeMap<ChangeHash, ChangeCandidate>,
        query: &ChangeCandidate,
        successful_limit: usize,
        stopped: Completion,
    ) -> (
        Result<(TrustedEpochProjection<'a>, TrustedEpochView), MeteredActorStateError<Completion>>,
        Vec<ProjectionWorkTrace>,
    ) {
        let trace = Rc::new(RefCell::new(Vec::new()));
        let successful = Cell::new(0_usize);
        let mut charge = |counter| {
            trace
                .borrow_mut()
                .push(ProjectionWorkTrace::Charge(counter));
            if successful.get() == successful_limit {
                Err(stopped)
            } else {
                successful.set(successful.get().saturating_add(1));
                Ok(())
            }
        };
        let mut source = WorkContractEpochProjectionSource {
            members: accepted_closure.iter(),
            accepted_closure,
            changes,
            trace: Rc::clone(&trace),
        };
        let projection = build_trusted_epoch_projection_observed(
            accepted_closure,
            changes,
            &mut source,
            &mut charge,
            |_| {},
            |_| trace.borrow_mut().push(ProjectionWorkTrace::Target),
        );
        let result = match projection {
            Ok(projection) => projection
                .candidate_metered_observed(query, &mut charge, |_| {
                    trace.borrow_mut().push(ProjectionWorkTrace::Target)
                })
                .map(|view| (projection, view)),
            Err(error) => Err(error),
        };
        let observed = trace.borrow().clone();
        (result, observed)
    }

    fn actor_state_bytes(states: &BTreeMap<ActorId, EpochActorState>) -> Vec<u8> {
        let mut bytes = Vec::new();
        for (actor, state) in states {
            bytes.extend_from_slice(actor.as_bytes());
            bytes.extend_from_slice(&state.last_sequence.to_be_bytes());
            bytes.extend_from_slice(&state.next_op.to_be_bytes());
            bytes.extend_from_slice(state.highest_change.as_bytes());
        }
        bytes
    }

    #[test]
    #[allow(clippy::panic)]
    fn projection_work_contract_preserves_first_stop_and_predecessor_output() {
        let first = candidate(1, 1, 1, 1);
        let mut second = candidate(1, 2, 2, 1);
        second.change_hash = ChangeHash::from_bytes([2; 32]);
        second.dependencies = vec![first.change_hash].into();
        let accepted = vec![first.clone(), second.clone()];
        let closure = BTreeSet::from([first.change_hash, second.change_hash]);
        let changes = accepted
            .iter()
            .cloned()
            .map(|candidate| (candidate.change_hash, candidate))
            .collect::<BTreeMap<_, _>>();
        let mut query = candidate(1, 3, 3, 1);
        query.change_hash = ChangeHash::from_bytes([3; 32]);
        query.dependencies = vec![second.change_hash].into();

        const TOTAL_CHARGES: usize = 83;
        const GRAPH_NODES: usize = 66;
        const GRAPH_EDGES: usize = 17;
        let (ample, trace) = projection_work_contract_run(
            &closure,
            &changes,
            &query,
            usize::MAX,
            Completion::BudgetExhausted,
        );
        assert!(ample.is_ok());
        assert_eq!(
            trace
                .iter()
                .filter(|entry| matches!(entry, ProjectionWorkTrace::Charge(_)))
                .count(),
            TOTAL_CHARGES
        );
        assert_eq!(
            trace
                .iter()
                .filter(|entry| {
                    matches!(entry, ProjectionWorkTrace::Charge(WorkCounter::GraphNode))
                })
                .count(),
            GRAPH_NODES
        );
        assert_eq!(
            trace
                .iter()
                .filter(|entry| {
                    matches!(entry, ProjectionWorkTrace::Charge(WorkCounter::GraphEdge))
                })
                .count(),
            GRAPH_EDGES
        );
        let Some((projection, view)) = ample.ok() else {
            return;
        };
        assert!(view.predecessor_is_direct_dependency());
        let (_, _, metered_states, _) = projection.into_accepted_state_parts();
        let predecessor_states = initialize_actor_states(accepted);
        assert!(predecessor_states.is_ok());
        let Some(predecessor_states) = predecessor_states.ok() else {
            return;
        };
        assert_eq!(
            actor_state_bytes(&metered_states),
            actor_state_bytes(&predecessor_states)
        );

        for successful_limit in 0..TOTAL_CHARGES {
            for stopped in [Completion::BudgetExhausted, Completion::Cancelled] {
                let (result, stopped_trace) = projection_work_contract_run(
                    &closure,
                    &changes,
                    &query,
                    successful_limit,
                    stopped,
                );
                assert!(matches!(
                    result,
                    Err(MeteredActorStateError::Work(actual)) if actual == stopped
                ));
                assert_eq!(
                    stopped_trace
                        .iter()
                        .filter(|entry| matches!(entry, ProjectionWorkTrace::Charge(_)))
                        .count(),
                    successful_limit + 1
                );
                assert!(matches!(
                    stopped_trace.last(),
                    Some(ProjectionWorkTrace::Charge(_))
                ));
            }
        }
        for successful_limit in [TOTAL_CHARGES, TOTAL_CHARGES + 1] {
            let (result, _) = projection_work_contract_run(
                &closure,
                &changes,
                &query,
                successful_limit,
                Completion::BudgetExhausted,
            );
            assert!(result.is_ok());
        }

        #[derive(Debug)]
        struct Injected;
        let injected = Injected;
        let result = initialize_actor_states_metered(&closure, &changes, |_| Err(&injected));
        assert!(matches!(
            result,
            Err(MeteredActorStateError::Work(error)) if core::ptr::eq(error, &injected)
        ));

        static PANIC_IDENTITY: u8 = 41;
        let panic = std::panic::catch_unwind(|| {
            let _ = initialize_actor_states_metered(&closure, &changes, |_| {
                std::panic::panic_any(PANIC_IDENTITY);
                #[allow(unreachable_code)]
                Ok::<_, ()>(())
            });
        });
        assert!(panic.is_err());
        assert!(
            panic
                .err()
                .and_then(|payload| payload.downcast::<u8>().ok())
                .is_some_and(|identity| *identity == PANIC_IDENTITY)
        );
    }

    fn observed_projection<'a>(
        members: Vec<ChangeHash>,
        accepted_closure: &'a BTreeSet<ChangeHash>,
        changes: &'a BTreeMap<ChangeHash, ChangeCandidate>,
        successful_limit: usize,
        stopped: Completion,
    ) -> (
        Result<super::TrustedEpochProjection<'a>, MeteredActorStateError<Completion>>,
        Vec<TraversalTrace>,
    ) {
        let trace = Rc::new(RefCell::new(Vec::new()));
        let mut source = ObservedEpochProjectionSource {
            members,
            cursor: 0,
            accepted_closure,
            changes,
            trace: Rc::clone(&trace),
        };
        let successful = Cell::new(0_usize);
        let result =
            build_trusted_epoch_projection(accepted_closure, changes, &mut source, |counter| {
                trace.borrow_mut().push(TraversalTrace::Charge(counter));
                if successful.get() == successful_limit {
                    Err(stopped)
                } else {
                    successful.set(successful.get().saturating_add(1));
                    Ok(())
                }
            });
        let observed = trace.borrow().clone();
        (result, observed)
    }

    fn observed_projection_build_operations<'a, E: Copy>(
        accepted_closure: &'a BTreeSet<ChangeHash>,
        changes: &'a BTreeMap<ChangeHash, ChangeCandidate>,
        successful_limit: usize,
        stopped: E,
    ) -> (
        Result<TrustedEpochProjection<'a>, MeteredActorStateError<E>>,
        Vec<BuildTrace>,
    ) {
        let trace = Rc::new(RefCell::new(Vec::new()));
        let successful = Cell::new(0_usize);
        let mut source = CanonicalEpochProjectionSource::new(accepted_closure, changes);
        let result = build_trusted_epoch_projection_observed(
            accepted_closure,
            changes,
            &mut source,
            |counter| {
                trace.borrow_mut().push(BuildTrace::Charge(counter));
                if successful.get() == successful_limit {
                    Err(stopped)
                } else {
                    successful.set(successful.get().saturating_add(1));
                    Ok(())
                }
            },
            |observation| {
                trace.borrow_mut().push(match observation.kind {
                    ProjectionBuildObservationKind::ChargeAttempt => {
                        BuildTrace::Attempt(observation.descriptor)
                    }
                    ProjectionBuildObservationKind::TargetCompleted => {
                        BuildTrace::Operation(observation.descriptor)
                    }
                });
            },
            |_| {},
        );
        let observed = trace.borrow().clone();
        (result, observed)
    }

    #[test]
    fn projection_build_trace_records_exact_attempt_and_completion_descriptors() {
        let first = candidate(1, 1, 1, 1);
        let accepted = BTreeSet::from([first.change_hash]);
        let changes = BTreeMap::from([(first.change_hash, first)]);
        let (result, trace) = observed_projection_build_operations(
            &accepted,
            &changes,
            usize::MAX,
            Completion::BudgetExhausted,
        );
        assert!(result.is_ok());
        assert!(!trace.is_empty());
        assert_eq!(trace.len() % 3, 0);
        for events in trace.chunks_exact(3) {
            assert!(matches!(
                events,
                [
                    BuildTrace::Attempt(_),
                    BuildTrace::Charge(_),
                    BuildTrace::Operation(_),
                ]
            ));
            let [
                BuildTrace::Attempt(attempt),
                BuildTrace::Charge(counter),
                BuildTrace::Operation(completed),
            ] = events
            else {
                return;
            };
            assert_eq!(attempt, completed);
            assert_eq!(*counter, attempt.counter);
            assert_eq!(attempt.site_id, attempt.site.id());
            assert_eq!(attempt.operation, attempt.site.operation());
            assert_eq!(attempt.phase, "construction");
            assert_eq!(attempt.abstract_owner_class, "source_operation");
            assert_eq!(attempt.applicability, "public_rust");
        }

        let injected = Completion::Cancelled;
        let (blocked, blocked_trace) =
            observed_projection_build_operations(&accepted, &changes, 0, injected);
        assert!(matches!(
            blocked,
            Err(MeteredActorStateError::Work(error)) if error == injected
        ));
        assert_eq!(
            blocked_trace,
            [
                BuildTrace::Attempt(ProjectionBuildSite::MemberCountRead.descriptor()),
                BuildTrace::Charge(WorkCounter::GraphNode),
            ]
        );
    }

    #[test]
    fn projection_source_operations_use_the_sealed_boundary() {
        let first = candidate(1, 1, 1, 1);
        let mut second = candidate(1, 2, 2, 1);
        second.dependencies = vec![first.change_hash].into();
        let closure = BTreeSet::from([first.change_hash, second.change_hash]);
        let changes = BTreeMap::from([(first.change_hash, first), (second.change_hash, second)]);
        let (ample, full_trace) = observed_projection_build_operations(
            &closure,
            &changes,
            usize::MAX,
            Completion::BudgetExhausted,
        );
        assert!(ample.is_ok());
        let owned = full_trace
            .iter()
            .filter_map(|entry| match entry {
                BuildTrace::Operation(site) => Some(site.operation()),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(owned.len(), 74);
        assert_eq!(
            owned
                .iter()
                .filter(|operation| {
                    **operation == ProjectionBuildOperation::CanonicalSourcePull
                })
                .count(),
            4
        );
        assert_eq!(
            owned
                .iter()
                .filter(|operation| {
                    **operation == ProjectionBuildOperation::CanonicalOrderCompare
                })
                .count(),
            7
        );
        assert_eq!(
            owned
                .iter()
                .filter(|operation| **operation == ProjectionBuildOperation::StateLookup)
                .count(),
            11
        );
        assert_eq!(
            owned
                .iter()
                .filter(|operation| {
                    **operation == ProjectionBuildOperation::ReadinessTransition
                })
                .count(),
            6
        );
        assert_eq!(
            owned
                .iter()
                .filter(|operation| **operation == ProjectionBuildOperation::CheckedArithmetic)
                .count(),
            6
        );
        assert_eq!(
            owned
                .iter()
                .filter(|operation| {
                    **operation == ProjectionBuildOperation::RemainingStateWrite
                })
                .count(),
            1
        );
        assert_eq!(
            owned
                .iter()
                .filter(|operation| **operation == ProjectionBuildOperation::MapInsertion)
                .count(),
            12
        );
        assert_eq!(
            owned
                .iter()
                .filter(|operation| **operation == ProjectionBuildOperation::SetInsertion)
                .count(),
            4
        );
        assert_eq!(
            owned
                .iter()
                .filter(|operation| {
                    **operation == ProjectionBuildOperation::CausalMaximumCompare
                })
                .count(),
            2
        );
        assert_eq!(
            owned
                .iter()
                .filter(|operation| {
                    **operation == ProjectionBuildOperation::CompletionComparison
                })
                .count(),
            1
        );
        assert_eq!(
            owned
                .iter()
                .filter(|operation| {
                    **operation == ProjectionBuildOperation::CandidateReadinessComparison
                })
                .count(),
            2
        );
        assert_eq!(
            owned
                .iter()
                .filter(|operation| {
                    **operation == ProjectionBuildOperation::CandidateKindComparison
                })
                .count(),
            2
        );
        assert_eq!(
            owned
                .iter()
                .filter(|operation| **operation == ProjectionBuildOperation::SourceCountRead)
                .count(),
            1
        );
        assert_eq!(
            owned
                .iter()
                .filter(|operation| {
                    **operation == ProjectionBuildOperation::ExpectedCountComparison
                })
                .count(),
            1
        );
        assert_eq!(
            owned
                .iter()
                .filter(|operation| {
                    **operation == ProjectionBuildOperation::CandidateIdentityComparison
                })
                .count(),
            2
        );
        assert_eq!(
            owned
                .iter()
                .filter(|operation| {
                    **operation == ProjectionBuildOperation::DependencyCountRead
                })
                .count(),
            2
        );
        assert_eq!(
            owned
                .iter()
                .filter(|operation| { **operation == ProjectionBuildOperation::ResultPublication })
                .count(),
            1
        );

        let boundaries = full_trace
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| match entry {
                BuildTrace::Operation(site) if owned.contains(&site.operation()) => {
                    assert!(matches!(
                        index.checked_sub(1).and_then(|prior| full_trace.get(prior)),
                        Some(BuildTrace::Charge(_))
                    ));
                    Some((
                        full_trace[..index]
                            .iter()
                            .filter(|prior| matches!(prior, BuildTrace::Charge(_)))
                            .count(),
                        full_trace[..=index]
                            .iter()
                            .filter(|prior| matches!(prior, BuildTrace::Operation(_)))
                            .count(),
                    ))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(boundaries.len(), owned.len());

        for (charge_boundary, operation_boundary) in boundaries {
            for stopped in [Completion::BudgetExhausted, Completion::Cancelled] {
                let (result, trace) = observed_projection_build_operations(
                    &closure,
                    &changes,
                    charge_boundary.saturating_sub(1),
                    stopped,
                );
                assert!(
                    matches!(result, Err(MeteredActorStateError::Work(error)) if error == stopped)
                );
                assert_eq!(
                    trace
                        .iter()
                        .filter(|entry| matches!(entry, BuildTrace::Operation(_)))
                        .count(),
                    operation_boundary - 1
                );
            }
        }
    }

    #[test]
    fn projection_causal_maximum_is_charged_once_per_accepted_change() {
        for (mut candidates, expected_next) in [
            (Vec::new(), 1_u64),
            (vec![candidate(1, 1, 1, 2)], 3_u64),
            (
                (1_u8..=4)
                    .map(|actor| {
                        let mut value = candidate(actor, 1, 1, u64::from(actor));
                        value.change_hash = ChangeHash::from_bytes([actor; 32]);
                        value
                    })
                    .collect::<Vec<_>>(),
                5_u64,
            ),
        ] {
            candidates.sort_by_key(|candidate| candidate.change_hash);
            let accepted = candidates
                .iter()
                .map(|candidate| candidate.change_hash)
                .collect::<BTreeSet<_>>();
            let changes = candidates
                .into_iter()
                .map(|candidate| (candidate.change_hash, candidate))
                .collect::<BTreeMap<_, _>>();
            let (result, trace) = observed_projection_build_operations(
                &accepted,
                &changes,
                usize::MAX,
                Completion::BudgetExhausted,
            );
            assert!(result.is_ok());
            let Ok(projection) = result else { return };
            assert_eq!(projection.causal_next_op, expected_next);
            assert_eq!(
                trace
                    .iter()
                    .filter(|entry| {
                        matches!(
                            entry,
                            BuildTrace::Operation(site)
                                if site.operation()
                                    == ProjectionBuildOperation::CausalMaximumCompare
                        )
                    })
                    .count(),
                accepted.len()
            );
        }
    }

    #[test]
    fn projection_operation_families_have_exact_n_minus_one_n_and_n_plus_one_stops() {
        let first = candidate(1, 1, 1, 1);
        let mut second = candidate(1, 2, 2, 1);
        second.change_hash = ChangeHash::from_bytes([2; 32]);
        second.dependencies = vec![first.change_hash].into();
        let accepted = BTreeSet::from([first.change_hash, second.change_hash]);
        let changes = BTreeMap::from([(first.change_hash, first), (second.change_hash, second)]);
        let (complete, trace) = observed_projection_build_operations(
            &accepted,
            &changes,
            usize::MAX,
            Completion::BudgetExhausted,
        );
        assert!(complete.is_ok());
        let operations = trace
            .iter()
            .filter_map(|entry| match entry {
                BuildTrace::Operation(site) => Some(site.operation()),
                BuildTrace::Attempt(_) | BuildTrace::Charge(_) => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(operations.len(), 74);
        let families = [
            (ProjectionBuildOperation::SourceCountRead, 1),
            (ProjectionBuildOperation::ExpectedCountComparison, 1),
            (ProjectionBuildOperation::CanonicalSourcePull, 4),
            (ProjectionBuildOperation::CanonicalOrderCompare, 7),
            (ProjectionBuildOperation::MembershipLookup, 3),
            (ProjectionBuildOperation::CandidateLookup, 4),
            (ProjectionBuildOperation::CandidateIdentityComparison, 2),
            (ProjectionBuildOperation::DependencyCountRead, 2),
            (ProjectionBuildOperation::DependencyLookup, 2),
            (ProjectionBuildOperation::CandidateReadinessComparison, 2),
            (ProjectionBuildOperation::StateLookup, 11),
            (ProjectionBuildOperation::ReadinessTransition, 6),
            (ProjectionBuildOperation::CandidateKindComparison, 2),
            (ProjectionBuildOperation::CheckedArithmetic, 6),
            (ProjectionBuildOperation::RemainingStateWrite, 1),
            (ProjectionBuildOperation::MapInsertion, 12),
            (ProjectionBuildOperation::SetInsertion, 4),
            (ProjectionBuildOperation::CausalMaximumCompare, 2),
            (ProjectionBuildOperation::CompletionComparison, 1),
            (ProjectionBuildOperation::ResultPublication, 1),
        ];
        for (family, expected) in families {
            assert_eq!(
                operations
                    .iter()
                    .filter(|operation| **operation == family)
                    .count(),
                expected
            );
        }

        let mut operation_ordinal = 0_usize;
        let mut successful_charges = 0_usize;
        for entry in &trace {
            match entry {
                BuildTrace::Attempt(_) => {}
                BuildTrace::Charge(_) => successful_charges += 1,
                BuildTrace::Operation(expected) => {
                    operation_ordinal += 1;
                    assert!(successful_charges > 0);
                    let blocked_limit = successful_charges - 1;
                    for stopped in [Completion::BudgetExhausted, Completion::Cancelled] {
                        let (blocked, blocked_trace) = observed_projection_build_operations(
                            &accepted,
                            &changes,
                            blocked_limit,
                            stopped,
                        );
                        assert!(matches!(
                            blocked,
                            Err(MeteredActorStateError::Work(value)) if value == stopped
                        ));
                        assert_eq!(
                            blocked_trace
                                .iter()
                                .filter(|entry| matches!(entry, BuildTrace::Operation(_)))
                                .count(),
                            operation_ordinal - 1
                        );

                        for allowance in [successful_charges, successful_charges + 1] {
                            let (admitted, admitted_trace) = observed_projection_build_operations(
                                &accepted, &changes, allowance, stopped,
                            );
                            let admitted_operations = admitted_trace
                                .iter()
                                .filter_map(|entry| match entry {
                                    BuildTrace::Operation(operation) => Some(*operation),
                                    BuildTrace::Attempt(_) | BuildTrace::Charge(_) => None,
                                })
                                .collect::<Vec<_>>();
                            assert_eq!(
                                admitted_operations.get(operation_ordinal - 1),
                                Some(expected)
                            );
                            if allowance < 74 {
                                assert!(matches!(
                                    admitted,
                                    Err(MeteredActorStateError::Work(value)) if value == stopped
                                ));
                            } else {
                                assert!(admitted.is_ok());
                            }
                        }
                    }
                }
            }
        }
        assert_eq!(successful_charges, 74);
        assert_eq!(operation_ordinal, 74);
    }

    #[test]
    fn charged_projection_traversal_stops_before_every_source_read() {
        let first = candidate(1, 1, 1, 1);
        let mut second = candidate(1, 2, 2, 1);
        second.dependencies = vec![first.change_hash].into();
        let closure = BTreeSet::from([first.change_hash, second.change_hash]);
        let changes = BTreeMap::from([
            (first.change_hash, first.clone()),
            (second.change_hash, second.clone()),
        ]);
        let members = vec![first.change_hash, second.change_hash];
        let (ample, full_trace) = observed_projection(
            members.clone(),
            &closure,
            &changes,
            usize::MAX,
            Completion::BudgetExhausted,
        );
        assert!(ample.is_ok());

        let mut charge_count = 0_usize;
        let mut operation_count = 0_usize;
        let mut boundaries = Vec::new();
        for (index, entry) in full_trace.iter().enumerate() {
            match entry {
                TraversalTrace::Charge(_) => charge_count = charge_count.saturating_add(1),
                TraversalTrace::Operation(_) => {
                    operation_count = operation_count.saturating_add(1);
                    let expected_counter = match entry {
                        TraversalTrace::Operation(
                            SourceOperation::PullMember(_) | SourceOperation::ReadCandidate(_),
                        ) => WorkCounter::GraphNode,
                        TraversalTrace::Operation(SourceOperation::PullDependency(_, _, _)) => {
                            WorkCounter::GraphEdge
                        }
                        TraversalTrace::Operation(SourceOperation::ReadAcceptedMember(hash)) => {
                            let counter =
                                full_trace[..index]
                                    .iter()
                                    .rev()
                                    .find_map(|prior| match prior {
                                        TraversalTrace::Operation(SourceOperation::PullMember(
                                            member,
                                        )) if member == hash => Some(WorkCounter::GraphNode),
                                        TraversalTrace::Operation(
                                            SourceOperation::PullDependency(_, _, dependency),
                                        ) if dependency == hash => Some(WorkCounter::GraphEdge),
                                        _ => None,
                                    });
                            assert!(
                                counter.is_some(),
                                "accepted membership lacks its source pull"
                            );
                            counter.unwrap_or(WorkCounter::GraphNode)
                        }
                        TraversalTrace::Charge(_) => unreachable!(),
                    };
                    assert_eq!(
                        index.checked_sub(1).and_then(|prior| full_trace.get(prior)),
                        Some(&TraversalTrace::Charge(expected_counter))
                    );
                    boundaries.push((charge_count, operation_count));
                }
            }
        }
        assert_eq!(boundaries.len(), 10);

        for (target_charge, target_operation) in boundaries {
            let operation_prefix = |trace: &[TraversalTrace]| {
                trace
                    .iter()
                    .filter(|entry| matches!(entry, TraversalTrace::Operation(_)))
                    .count()
            };
            let (before, before_trace) = observed_projection(
                members.clone(),
                &closure,
                &changes,
                target_charge.saturating_sub(1),
                Completion::BudgetExhausted,
            );
            assert!(matches!(
                before,
                Err(MeteredActorStateError::Work(Completion::BudgetExhausted))
            ));
            assert_eq!(operation_prefix(&before_trace), target_operation - 1);

            for allowance in [target_charge, target_charge.saturating_add(1)] {
                let (_, trace) = observed_projection(
                    members.clone(),
                    &closure,
                    &changes,
                    allowance,
                    Completion::BudgetExhausted,
                );
                assert!(operation_prefix(&trace) >= target_operation);
            }

            let (cancelled, cancelled_trace) = observed_projection(
                members.clone(),
                &closure,
                &changes,
                target_charge.saturating_sub(1),
                Completion::Cancelled,
            );
            assert!(matches!(
                cancelled,
                Err(MeteredActorStateError::Work(Completion::Cancelled))
            ));
            assert_eq!(operation_prefix(&cancelled_trace), target_operation - 1);
        }
    }

    #[test]
    fn projection_rejects_noncanonical_members_and_dependencies_without_repair() {
        let first = candidate(1, 1, 1, 1);
        let mut second = candidate(2, 1, 1, 1);
        second.change_hash = ChangeHash::from_bytes([2; 32]);
        let closure = BTreeSet::from([first.change_hash, second.change_hash]);
        let changes = BTreeMap::from([
            (first.change_hash, first.clone()),
            (second.change_hash, second.clone()),
        ]);
        for members in [
            vec![second.change_hash, first.change_hash],
            vec![first.change_hash, first.change_hash],
        ] {
            let (result, _) = observed_projection(
                members,
                &closure,
                &changes,
                usize::MAX,
                Completion::BudgetExhausted,
            );
            assert!(matches!(
                result,
                Err(MeteredActorStateError::State(
                    ActorStateError::NoncanonicalInput
                ))
            ));
        }
        for members in [
            vec![first.change_hash],
            vec![first.change_hash, second.change_hash, second.change_hash],
        ] {
            let (result, trace) = observed_projection(
                members,
                &closure,
                &changes,
                usize::MAX,
                Completion::BudgetExhausted,
            );
            assert!(matches!(
                result,
                Err(MeteredActorStateError::State(
                    ActorStateError::NoncanonicalInput
                ))
            ));
            assert_eq!(
                trace,
                [
                    TraversalTrace::Charge(WorkCounter::GraphNode),
                    TraversalTrace::Charge(WorkCounter::GraphNode),
                ]
            );
        }

        let mut third = candidate(3, 1, 2, 1);
        third.change_hash = ChangeHash::from_bytes([3; 32]);
        let closure = BTreeSet::from([first.change_hash, second.change_hash, third.change_hash]);
        for dependencies in [
            vec![second.change_hash, first.change_hash],
            vec![first.change_hash, first.change_hash],
        ] {
            third.dependencies = dependencies.into();
            let changes = BTreeMap::from([
                (first.change_hash, first.clone()),
                (second.change_hash, second.clone()),
                (third.change_hash, third.clone()),
            ]);
            assert!(matches!(
                initialize_actor_states_metered(&closure, &changes, |_| Ok::<_, ()>(())),
                Err(MeteredActorStateError::State(
                    ActorStateError::NoncanonicalInput
                ))
            ));
        }

        let mismatched = BTreeMap::from([(first.change_hash, second)]);
        assert!(matches!(
            initialize_actor_states_metered(
                &BTreeSet::from([first.change_hash]),
                &mismatched,
                |_| Ok::<_, ()>(())
            ),
            Err(MeteredActorStateError::State(
                ActorStateError::NoncanonicalInput
            ))
        ));
    }

    #[test]
    fn trusted_epoch_projection_shape_and_construction_are_sealed() {
        fn assert_shape<T: Send + Sync>() {}
        assert_shape::<super::TrustedEpochProjection<'static>>();
        assert_shape::<TrustedEpochView>();

        let source = include_str!("actor_state.rs");
        let production = source
            .split_once("#[cfg(test)]\npub(crate) mod tests")
            .map_or(source, |item| item.0);
        assert_eq!(production.matches("|| TrustedEpochProjection {").count(), 1);
        assert_eq!(
            production
                .matches("#[cfg(test)]\npub(crate) fn initialize_actor_states(")
                .count(),
            1
        );
        assert!(!production.contains("pub struct TrustedEpochProjection"));
        assert!(!production.contains("pub(crate) struct TrustedEpochProjectionParts"));
        assert!(!production.contains("&mut TrustedEpochProjection"));
        assert!(!production.contains("pub(crate) dependencies:"));
        assert!(!production.contains("pub(crate) actor_states:"));
    }

    #[test]
    fn validate_next_op_for_nonempty_changes() {
        let actor = ActorId::from_bytes([1; 32]);
        let mut states = BTreeMap::new();
        let first = candidate(1, 1, 1, 2);
        assert_eq!(
            reference_apply_nonempty_counter(&mut states, &first),
            Ok(())
        );
        assert_eq!(states[&actor].next_op, 3);
        let second = candidate(1, 2, 3, 1);
        assert_eq!(
            reference_apply_nonempty_counter(&mut states, &second),
            Ok(())
        );

        let mut gap = candidate(1, 3, 5, 1);
        assert_eq!(
            reference_apply_nonempty_counter(&mut states.clone(), &gap),
            Err(ActorStateError::OperationCounter)
        );
        gap.start_op = 2;
        assert_eq!(
            reference_apply_nonempty_counter(&mut states.clone(), &gap),
            Err(ActorStateError::OperationCounter)
        );
        let mut overflow_states = BTreeMap::from([(
            actor,
            EpochActorState {
                last_sequence: 1,
                next_op: u64::MAX,
                highest_change: first.change_hash,
            },
        )]);
        let overflow = candidate(1, 2, u64::MAX, 1);
        assert_eq!(
            reference_apply_nonempty_counter(&mut overflow_states, &overflow),
            Err(ActorStateError::OperationCounter)
        );
        assert_eq!(
            reference_apply_nonempty_counter(&mut states, &candidate(2, 1, 4, 1)),
            Ok(())
        );
    }

    #[test]
    fn nonempty_change_advances_counter() {
        let actor = ActorId::from_bytes([5; 32]);
        let first = candidate(5, 1, 1, 2);
        let second = candidate(5, 2, 3, 4);
        let mut states = BTreeMap::new();
        assert_eq!(
            reference_apply_nonempty_counter(&mut states, &first),
            Ok(())
        );
        assert_eq!(
            reference_apply_nonempty_counter(&mut states, &second),
            Ok(())
        );
        assert_eq!(states[&actor].last_sequence, 2);
        assert_eq!(states[&actor].next_op, 7);
        assert_eq!(states[&actor].highest_change, second.change_hash);

        let mut overflow = BTreeMap::from([(
            actor,
            EpochActorState {
                last_sequence: 2,
                next_op: u64::MAX,
                highest_change: second.change_hash,
            },
        )]);
        assert_eq!(
            reference_apply_nonempty_counter(&mut overflow, &candidate(5, 3, u64::MAX, 1)),
            Err(ActorStateError::OperationCounter)
        );
    }

    #[test]
    fn causal_operation_clock_allows_concurrent_actors() {
        let mut base = candidate(1, 1, 1, 1);
        base.change_hash = ChangeHash::from_bytes([1; 32]);
        let mut left = candidate(2, 1, 2, 1);
        left.change_hash = ChangeHash::from_bytes([2; 32]);
        left.dependencies = vec![base.change_hash].into();
        let mut right = candidate(3, 1, 2, 1);
        right.change_hash = ChangeHash::from_bytes([3; 32]);
        right.dependencies = vec![base.change_hash].into();
        let mut merge = candidate(4, 1, 3, 0);
        merge.change_hash = ChangeHash::from_bytes([4; 32]);
        merge.dependencies = vec![left.change_hash, right.change_hash].into();
        let states = initialize_actor_states([merge, right, base, left]);
        assert!(states.is_ok());
    }

    #[test]
    fn causal_operation_counter_neutral_vectors() {
        let vectors: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../fixtures/v1_draft/conformance/causal_operation_counter_v1.json"
        ))
        .unwrap_or_default();
        assert_eq!(
            vectors["schema"],
            "nostr_automerge.causal_operation_counter.v1"
        );
        assert_eq!(
            vectors["requirements"],
            serde_json::json!(["NCRDT-SEQ-001", "NCRDT-SEQ-002"])
        );
        let cases = vectors["cases"].as_array();
        assert!(
            cases.is_some(),
            "neutral causal-counter cases must be an array"
        );
        let Some(cases) = cases else {
            return;
        };
        assert_eq!(cases.len(), 6);
        for case in cases {
            let mut states = BTreeMap::new();
            let mut current_heads = BTreeSet::new();
            let visible = case["visible_actor_states"].as_array();
            assert!(visible.is_some(), "visible actor states must be an array");
            let Some(visible) = visible else {
                return;
            };
            for state in visible {
                let actor =
                    u8::try_from(state["actor"].as_u64().unwrap_or_default()).unwrap_or_default();
                let hash = ChangeHash::from_bytes([actor; 32]);
                states.insert(
                    ActorId::from_bytes([actor; 32]),
                    EpochActorState {
                        last_sequence: state["last_sequence"].as_u64().unwrap_or_default(),
                        next_op: state["exclusive_next_op"].as_u64().unwrap_or_default(),
                        highest_change: hash,
                    },
                );
                current_heads.insert(hash);
            }
            let input = &case["candidate"];
            let actor =
                u8::try_from(input["actor"].as_u64().unwrap_or_default()).unwrap_or_default();
            let mut candidate = candidate(
                actor,
                input["sequence"].as_u64().unwrap_or_default(),
                input["start_op"].as_u64().unwrap_or_default(),
                input["operation_count"].as_u64().unwrap_or_default(),
            );
            candidate.dependencies = current_heads.iter().copied().collect::<Vec<_>>().into();
            let result = if input["empty"].as_bool() == Some(true) {
                reference_apply_empty_counter(&mut states, &candidate, &current_heads)
            } else {
                reference_apply_nonempty_counter(&mut states, &candidate)
            };
            match case["expected"]["result"].as_str() {
                Some("accepted") => {
                    assert_eq!(result, Ok(()), "case {}", case["id"]);
                    assert_eq!(
                        states
                            .get(&ActorId::from_bytes([actor; 32]))
                            .map(|state| state.next_op),
                        case["expected"]["exclusive_next_op"].as_u64(),
                        "case {}",
                        case["id"]
                    );
                }
                Some("operation_counter") => assert_eq!(
                    result,
                    Err(ActorStateError::OperationCounter),
                    "case {}",
                    case["id"]
                ),
                other => {
                    assert!(
                        other.is_some_and(|value| {
                            matches!(value, "accepted" | "operation_counter")
                        }),
                        "unknown neutral-vector result: {}",
                        case["id"]
                    );
                    return;
                }
            }
        }
    }

    #[test]
    fn validate_empty_merge_change_counters() {
        let mut states = BTreeMap::new();
        let first = candidate(1, 1, 1, 2);
        assert_eq!(
            reference_apply_nonempty_counter(&mut states, &first),
            Ok(())
        );
        let first_head = ChangeHash::from_bytes([7; 32]);
        let mut empty = candidate(1, 2, 3, 0);
        empty.change_hash = ChangeHash::from_bytes([2; 32]);
        empty.dependencies = vec![first_head].into();
        assert_eq!(
            reference_apply_empty_counter(&mut states, &empty, &BTreeSet::from([first_head])),
            Ok(())
        );
        assert_eq!(states[&ActorId::from_bytes([1; 32])].next_op, 3);

        let mut second_empty = candidate(1, 3, 3, 0);
        second_empty.change_hash = ChangeHash::from_bytes([3; 32]);
        second_empty.dependencies = vec![empty.change_hash].into();
        assert_eq!(
            reference_apply_empty_counter(
                &mut states,
                &second_empty,
                &BTreeSet::from([empty.change_hash])
            ),
            Ok(())
        );
        let mut wrong_start = candidate(1, 4, 4, 0);
        wrong_start.dependencies = vec![second_empty.change_hash].into();
        assert_eq!(
            reference_apply_empty_counter(
                &mut states.clone(),
                &wrong_start,
                &BTreeSet::from([second_empty.change_hash])
            ),
            Err(ActorStateError::OperationCounter)
        );
        assert_eq!(
            reference_apply_empty_counter(&mut states, &wrong_start, &BTreeSet::new()),
            Err(ActorStateError::DependencyFrontier)
        );
    }

    #[test]
    fn finding_100_actor_predecessor_scan_reproduction() {
        let mut accepted = BTreeMap::new();
        let mut closure = BTreeSet::new();
        let mut previous = None;
        for sequence in 1..=64 {
            let mut change = candidate(1, sequence, sequence, 1);
            change.dependencies = previous.into_iter().collect::<Vec<_>>().into();
            closure.insert(change.change_hash);
            previous = Some(change.change_hash);
            accepted.insert(change.change_hash, change);
        }
        let mut unrelated = candidate(2, 1, 65, 1);
        unrelated.change_hash = ChangeHash::from_bytes([200; 32]);
        unrelated.dependencies = previous.into_iter().collect::<Vec<_>>().into();
        closure.insert(unrelated.change_hash);
        accepted.insert(unrelated.change_hash, unrelated.clone());
        let mut next = candidate(1, 65, 65, 1);
        next.start_op = 66;
        next.dependencies = vec![unrelated.change_hash].into();

        let projection = initialize_actor_states_metered(&closure, &accepted, |_| Ok::<_, ()>(()));
        assert!(projection.is_ok(), "{:?}", projection.as_ref().err());
        let Some(projection) = projection.ok() else {
            return;
        };
        assert!(
            projection
                .actor_sequence_decision_metered(&next, |_| Ok::<_, ()>(()))
                .is_ok(),
            "the actor predecessor is accepted transitively and need not be a direct dependency"
        );

        let source = include_str!("actor_state.rs");
        let production = source
            .split_once("#[cfg(test)]\npub(crate) mod tests")
            .map_or(source, |item| item.0);
        let engine = include_str!("../reference/epoch_engine.rs");
        let engine_production = engine
            .split_once("#[cfg(test)]\nmod tests")
            .map_or(engine, |item| item.0);
        assert!(
            !production.contains("fn validate_actor_predecessor(")
                && !engine_production.contains("validate_actor_predecessor")
                && engine_production
                    .matches(".candidate_semantics_decision_metered(")
                    .count()
                    == 1,
            "unmetered actor predecessor collection remains"
        );
    }

    #[test]
    fn finding_100_causal_next_op_scan_reproduction() {
        let mut accepted = BTreeMap::new();
        let mut closure = BTreeSet::new();
        for actor in 1..=64 {
            let mut change = candidate(actor, 1, 1, u64::from(actor));
            change.change_hash = ChangeHash::from_bytes([actor; 32]);
            closure.insert(change.change_hash);
            accepted.insert(change.change_hash, change);
        }
        let projection = initialize_actor_states_metered(&closure, &accepted, |_| Ok::<_, ()>(()));
        assert!(projection.is_ok());
        let Some(projection) = projection.ok() else {
            return;
        };
        let mut next = candidate(100, 1, 65, 1);
        next.change_hash = ChangeHash::from_bytes([100; 32]);
        assert_eq!(
            projection.causal_next_decision_metered(&next, |_| Ok::<_, ()>(())),
            Ok(66)
        );

        let source = include_str!("actor_state.rs");
        let production = source
            .split_once("#[cfg(test)]\npub(crate) mod tests")
            .map_or(source, |item| item.0);
        let engine = include_str!("../reference/epoch_engine.rs");
        let engine_production = engine
            .split_once("#[cfg(test)]\nmod tests")
            .map_or(engine, |item| item.0);
        assert!(
            !production.contains("fn causal_next_op(states:")
                && !production.contains("legacy_counter_is_valid")
                && !production.contains("pub(crate) fn apply_nonempty_counter")
                && engine_production
                    .matches(".causal_next_decision_metered(")
                    .count()
                    == 0
                && engine_production
                    .matches(".candidate_semantics_decision_metered(")
                    .count()
                    == 1,
            "unmetered causal next-op scan remains"
        );
    }

    #[test]
    fn finding_100_empty_frontier_work_reproduction() {
        let current_heads = BTreeSet::from([
            ChangeHash::from_bytes([10; 32]),
            ChangeHash::from_bytes([20; 32]),
            ChangeHash::from_bytes([30; 32]),
        ]);
        let mut empty = candidate(1, 1, 1, 0);
        empty.dependencies = current_heads.iter().copied().collect::<Vec<_>>().into();
        let branch = BTreeMap::new();
        let accepted = BTreeSet::new();
        let projection = TrustedEpochProjection {
            branch_membership: &branch,
            accepted_closure: &accepted,
            dependencies: BTreeMap::new(),
            frontier_heads: BTreeSet::new(),
            actor_states: BTreeMap::new(),
            writer_contributions: BTreeMap::new(),
            causal_next_op: 1,
        };
        assert!(
            projection
                .empty_frontier_decision_metered(&empty, &current_heads, |_| Ok::<_, ()>(()))
                .is_ok()
        );

        let source = include_str!("actor_state.rs");
        let production = source
            .split_once("#[cfg(test)]\npub(crate) mod tests")
            .map_or(source, |item| item.0);
        let method = production
            .split_once("fn empty_frontier_decision_metered_observed")
            .map(|item| item.1)
            .and_then(|body| body.split_once("pub(crate) fn into_accepted_state_parts"))
            .map_or("", |item| item.0);
        assert!(
            !method.contains(".collect::<")
                && !method.contains(".clone()")
                && !method.contains(".sort")
                && !method.contains(".dedup")
                && include_str!("../reference/epoch_engine.rs")
                    .split_once("#[cfg(test)]\nmod tests")
                    .map_or("", |item| item.0)
                    .matches(".empty_frontier_decision_metered(")
                    .count()
                    == 0,
            "unmetered empty-frontier allocation remains"
        );
    }

    fn assert_v16_source_site_counter(
        operation_enum: &str,
        variant: &str,
        occurrence: usize,
        expected: WorkCounter,
    ) {
        let source = include_str!("actor_state.rs");
        let production = source
            .split_once("#[cfg(test)]\npub(crate) mod tests")
            .map_or(source, |item| item.0);
        if operation_enum == "ProjectionBuildOperation" {
            let registry = production
                .split_once("projection_build_sites! {")
                .map(|item| item.1)
                .and_then(|body| body.split_once("\n}"))
                .map_or("", |item| item.0);
            let needle = format!("=> ({variant}, ");
            let site = registry.match_indices(&needle).nth(occurrence - 1);
            assert!(
                site.is_some(),
                "missing descriptor site {operation_enum}::{variant}#{occurrence}"
            );
            let Some((offset, _)) = site else { return };
            let suffix = &registry[offset + needle.len()..];
            let counter = suffix
                .chars()
                .take_while(|value| value.is_ascii_alphanumeric())
                .collect::<String>();
            assert_eq!(counter, format!("{expected:?}"));
            return;
        }
        if matches!(
            operation_enum,
            "ActorDecisionOperation" | "CausalNextOperation"
        ) {
            let registry_name = if operation_enum == "ActorDecisionOperation" {
                "actor_decision_sites! {"
            } else {
                "causal_next_sites! {"
            };
            let registry = production
                .split_once(registry_name)
                .map(|item| item.1)
                .and_then(|body| body.split_once("\n}"))
                .map_or("", |item| item.0);
            let needle = format!("=> {variant}");
            assert!(
                registry
                    .match_indices(&needle)
                    .nth(occurrence - 1)
                    .is_some(),
                "missing descriptor site {operation_enum}::{variant}#{occurrence}"
            );
            assert_eq!(expected, WorkCounter::GraphNode);
            return;
        }
        let needle = format!("{operation_enum}::{variant}");
        let site = production.match_indices(&needle).nth(occurrence - 1);
        assert!(
            site.is_some(),
            "missing source site {operation_enum}::{variant}#{occurrence}"
        );
        let Some((offset, _)) = site else { return };
        let prefix = &production[..offset];
        let counter_offset = prefix.rfind("WorkCounter::");
        assert!(
            counter_offset.is_some(),
            "missing source counter for {operation_enum}::{variant}#{occurrence}"
        );
        let Some(counter_offset) = counter_offset else {
            return;
        };
        let counter = prefix[counter_offset + "WorkCounter::".len()..]
            .chars()
            .take_while(|value| value.is_ascii_alphanumeric())
            .collect::<String>();
        assert_eq!(counter, format!("{expected:?}"));
    }

    fn assert_v16_projection_build_site(
        family: ProjectionBuildOperation,
        variant: &str,
        occurrence: usize,
        counter: WorkCounter,
    ) {
        assert_v16_source_site_counter("ProjectionBuildOperation", variant, occurrence, counter);
        assert_projection_build_family_exact(family);

        let first = candidate(1, 1, 1, 1);
        let mut second = candidate(1, 2, 2, 1);
        second.change_hash = ChangeHash::from_bytes([2; 32]);
        second.dependencies = vec![first.change_hash].into();
        let accepted = BTreeSet::from([first.change_hash, second.change_hash]);
        let changes = BTreeMap::from([(first.change_hash, first), (second.change_hash, second)]);
        let (_, trace) = observed_projection_build_operations(
            &accepted,
            &changes,
            usize::MAX,
            Completion::BudgetExhausted,
        );
        let mut charges = 0_usize;
        let target = trace.iter().find_map(|entry| match entry {
            BuildTrace::Attempt(_) => None,
            BuildTrace::Charge(_) => {
                charges = charges.saturating_add(1);
                None
            }
            BuildTrace::Operation(site) if site.operation() == family => Some(charges),
            BuildTrace::Operation(_) => None,
        });
        assert!(target.is_some(), "unreachable build family");
        let Some(target) = target else { return };
        let injected = (variant, occurrence);
        let (result, blocked) =
            observed_projection_build_operations(&accepted, &changes, target - 1, &injected);
        assert!(matches!(
            result,
            Err(MeteredActorStateError::Work(error)) if core::ptr::eq(error, &injected)
        ));
        assert!(!blocked.iter().any(
            |entry| matches!(entry, BuildTrace::Operation(site) if site.operation() == family)
        ));
    }

    fn actor_site_fixture() -> (
        BTreeMap<ChangeHash, ChangeCandidate>,
        BTreeSet<ChangeHash>,
        ChangeCandidate,
    ) {
        let first = candidate(1, 1, 1, 1);
        let closure = BTreeSet::from([first.change_hash]);
        let changes = BTreeMap::from([(first.change_hash, first.clone())]);
        let mut next = candidate(1, 2, 2, 1);
        next.change_hash = ChangeHash::from_bytes([2; 32]);
        next.dependencies = vec![first.change_hash].into();
        (changes, closure, next)
    }

    fn assert_v16_actor_site(
        family: ActorDecisionOperation,
        variant: &str,
        occurrence: usize,
        counter: WorkCounter,
    ) {
        assert_v16_source_site_counter("ActorDecisionOperation", variant, occurrence, counter);
        let (changes, closure, next) = actor_site_fixture();
        let projection = initialize_actor_states_metered(&closure, &changes, |_| Ok::<_, ()>(()));
        assert!(projection.is_ok(), "actor fixture");
        let Ok(projection) = projection else { return };
        let (_, trace) =
            observed_actor_sequence(&projection, &next, usize::MAX, Completion::BudgetExhausted);
        let target = trace
            .iter()
            .filter(|entry| matches!(entry, ActorDecisionTrace::Operation(_)))
            .position(
                |entry| matches!(entry, ActorDecisionTrace::Operation(value) if value.operation() == family),
            )
            .map(|index| index + 1);
        assert!(target.is_some(), "unreachable actor family");
        let Some(target) = target else { return };
        for stopped in [Completion::BudgetExhausted, Completion::Cancelled] {
            let (result, blocked) =
                observed_actor_sequence(&projection, &next, target - 1, stopped);
            assert_eq!(result, Err(MeteredActorStateError::Work(stopped)));
            assert!(!blocked.iter().any(
                |entry| matches!(entry, ActorDecisionTrace::Operation(value) if value.operation() == family)
            ));
        }
        for allowance in [target, target + 1] {
            let (_, admitted) =
                observed_actor_sequence(&projection, &next, allowance, Completion::BudgetExhausted);
            assert!(admitted.iter().any(
                |entry| matches!(entry, ActorDecisionTrace::Operation(value) if value.operation() == family)
            ));
        }
        let injected = (variant, occurrence);
        let (result, _) = observed_actor_sequence(&projection, &next, target - 1, &injected);
        assert!(matches!(
            result,
            Err(MeteredActorStateError::Work(error)) if core::ptr::eq(error, &injected)
        ));
    }

    fn assert_v16_causal_site(
        family: CausalNextOperation,
        variant: &str,
        occurrence: usize,
        counter: WorkCounter,
    ) {
        assert_v16_source_site_counter("CausalNextOperation", variant, occurrence, counter);
        assert_causal_consumer_family_exact(family);
    }

    fn assert_v16_frontier_site(
        family: FrontierComparisonOperation,
        variant: &str,
        occurrence: usize,
        counter: WorkCounter,
    ) {
        assert_v16_source_site_counter("FrontierComparisonOperation", variant, occurrence, counter);
        assert_frontier_family_exact(family);
    }

    macro_rules! v16_projection_build_site_proofs {
        ($(($test:ident, $family:ident, $occurrence:expr, $counter:ident)),+ $(,)?) => {
            $(
                #[test]
                fn $test() {
                    assert_v16_projection_build_site(
                        ProjectionBuildOperation::$family,
                        stringify!($family),
                        $occurrence,
                        WorkCounter::$counter,
                    );
                }
            )+
        };
    }

    v16_projection_build_site_proofs!(
        (
            causal_projection_v16_site_projection_construction_source_count_read_01,
            SourceCountRead,
            1,
            GraphNode
        ),
        (
            causal_projection_v16_site_projection_construction_expected_count_comparison_01,
            ExpectedCountComparison,
            1,
            GraphNode
        ),
        (
            causal_projection_v16_site_projection_construction_canonical_source_pull_01,
            CanonicalSourcePull,
            1,
            GraphNode
        ),
        (
            causal_projection_v16_site_projection_construction_canonical_order_compare_01,
            CanonicalOrderCompare,
            1,
            GraphNode
        ),
        (
            causal_projection_v16_site_projection_construction_membership_lookup_01,
            MembershipLookup,
            1,
            GraphNode
        ),
        (
            causal_projection_v16_site_projection_construction_candidate_lookup_01,
            CandidateLookup,
            1,
            GraphNode
        ),
        (
            causal_projection_v16_site_projection_construction_candidate_identity_comparison_01,
            CandidateIdentityComparison,
            1,
            GraphNode
        ),
        (
            causal_projection_v16_site_projection_construction_dependency_count_read_01,
            DependencyCountRead,
            1,
            GraphNode
        ),
        (
            causal_projection_v16_site_projection_construction_dependency_lookup_01,
            DependencyLookup,
            1,
            GraphEdge
        ),
        (
            causal_projection_v16_site_projection_construction_canonical_order_compare_02,
            CanonicalOrderCompare,
            2,
            GraphEdge
        ),
        (
            causal_projection_v16_site_projection_construction_membership_lookup_02,
            MembershipLookup,
            2,
            GraphEdge
        ),
        (
            causal_projection_v16_site_projection_construction_set_insertion_01,
            SetInsertion,
            1,
            GraphEdge
        ),
        (
            causal_projection_v16_site_projection_construction_set_insertion_02,
            SetInsertion,
            2,
            GraphEdge
        ),
        (
            causal_projection_v16_site_projection_construction_map_insertion_01,
            MapInsertion,
            1,
            GraphEdge
        ),
        (
            causal_projection_v16_site_projection_construction_set_insertion_03,
            SetInsertion,
            3,
            GraphEdge
        ),
        (
            causal_projection_v16_site_projection_construction_candidate_readiness_comparison_01,
            CandidateReadinessComparison,
            1,
            GraphNode
        ),
        (
            causal_projection_v16_site_projection_construction_readiness_transition_01,
            ReadinessTransition,
            1,
            GraphNode
        ),
        (
            causal_projection_v16_site_projection_construction_map_insertion_02,
            MapInsertion,
            2,
            GraphNode
        ),
        (
            causal_projection_v16_site_projection_construction_map_insertion_03,
            MapInsertion,
            3,
            GraphNode
        ),
        (
            causal_projection_v16_site_projection_construction_readiness_transition_02,
            ReadinessTransition,
            2,
            GraphNode
        ),
        (
            causal_projection_v16_site_projection_construction_canonical_source_pull_02,
            CanonicalSourcePull,
            2,
            GraphNode
        ),
        (
            causal_projection_v16_site_projection_construction_candidate_lookup_02,
            CandidateLookup,
            2,
            GraphNode
        ),
        (
            causal_projection_v16_site_projection_construction_state_lookup_01,
            StateLookup,
            1,
            GraphNode
        ),
        (
            causal_projection_v16_site_projection_construction_set_insertion_04,
            SetInsertion,
            4,
            GraphNode
        ),
        (
            causal_projection_v16_site_projection_construction_state_lookup_02,
            StateLookup,
            2,
            GraphNode
        ),
        (
            causal_projection_v16_site_projection_construction_checked_arithmetic_01,
            CheckedArithmetic,
            1,
            GraphNode
        ),
        (
            causal_projection_v16_site_projection_construction_canonical_order_compare_03,
            CanonicalOrderCompare,
            3,
            GraphNode
        ),
        (
            causal_projection_v16_site_projection_construction_canonical_order_compare_04,
            CanonicalOrderCompare,
            4,
            GraphNode
        ),
        (
            causal_projection_v16_site_projection_construction_state_lookup_03,
            StateLookup,
            3,
            GraphNode
        ),
        (
            causal_projection_v16_site_projection_construction_canonical_order_compare_05,
            CanonicalOrderCompare,
            5,
            GraphNode
        ),
        (
            causal_projection_v16_site_projection_construction_candidate_kind_comparison_01,
            CandidateKindComparison,
            1,
            GraphNode
        ),
        (
            causal_projection_v16_site_projection_construction_checked_arithmetic_02,
            CheckedArithmetic,
            2,
            GraphNode
        ),
        (
            causal_projection_v16_site_projection_construction_causal_maximum_compare_01,
            CausalMaximumCompare,
            1,
            GraphNode
        ),
        (
            causal_projection_v16_site_projection_construction_map_insertion_04,
            MapInsertion,
            4,
            GraphNode
        ),
        (
            causal_projection_v16_site_projection_construction_map_insertion_05,
            MapInsertion,
            5,
            GraphNode
        ),
        (
            causal_projection_v16_site_projection_construction_map_insertion_06,
            MapInsertion,
            6,
            GraphNode
        ),
        (
            causal_projection_v16_site_projection_construction_checked_arithmetic_03,
            CheckedArithmetic,
            3,
            GraphNode
        ),
        (
            causal_projection_v16_site_projection_construction_state_lookup_04,
            StateLookup,
            4,
            GraphEdge
        ),
        (
            causal_projection_v16_site_projection_construction_state_lookup_05,
            StateLookup,
            5,
            GraphEdge
        ),
        (
            causal_projection_v16_site_projection_construction_dependency_lookup_02,
            DependencyLookup,
            2,
            GraphEdge
        ),
        (
            causal_projection_v16_site_projection_construction_state_lookup_06,
            StateLookup,
            6,
            GraphEdge
        ),
        (
            causal_projection_v16_site_projection_construction_checked_arithmetic_04,
            CheckedArithmetic,
            4,
            GraphEdge
        ),
        (
            causal_projection_v16_site_projection_construction_remaining_state_write_01,
            RemainingStateWrite,
            1,
            GraphEdge
        ),
        (
            causal_projection_v16_site_projection_construction_state_lookup_07,
            StateLookup,
            7,
            GraphEdge
        ),
        (
            causal_projection_v16_site_projection_construction_causal_maximum_compare_02,
            CausalMaximumCompare,
            2,
            GraphEdge
        ),
        (
            causal_projection_v16_site_projection_construction_map_insertion_07,
            MapInsertion,
            7,
            GraphEdge
        ),
        (
            causal_projection_v16_site_projection_construction_readiness_transition_03,
            ReadinessTransition,
            3,
            GraphEdge
        ),
        (
            causal_projection_v16_site_projection_construction_readiness_transition_04,
            ReadinessTransition,
            4,
            GraphNode
        ),
        (
            causal_projection_v16_site_projection_construction_completion_comparison_01,
            CompletionComparison,
            1,
            GraphNode
        ),
        (
            causal_projection_v16_site_projection_construction_result_publication_01,
            ResultPublication,
            1,
            GraphNode
        ),
    );

    macro_rules! v16_direct_site_proofs {
        ($assertion:ident, $operation:ident; $(($test:ident, $family:ident, $counter:ident)),+ $(,)?) => {
            $(
                #[test]
                fn $test() {
                    $assertion($operation::$family, stringify!($family), 1, WorkCounter::$counter);
                }
            )+
        };
    }

    v16_direct_site_proofs!(
        assert_v16_actor_site,
        ActorDecisionOperation;
        (causal_projection_v16_site_actor_sequence_actor_state_read_01, ActorStateRead, GraphNode),
        (causal_projection_v16_site_actor_sequence_predecessor_candidate_read_01, PredecessorCandidateRead, GraphNode),
        (causal_projection_v16_site_actor_sequence_actor_identity_decision_01, ActorIdentityDecision, GraphNode),
        (causal_projection_v16_site_actor_sequence_sequence_relation_decision_01, SequenceRelationDecision, GraphNode),
    );
    v16_direct_site_proofs!(
        assert_v16_causal_site,
        CausalNextOperation;
        (causal_projection_v16_site_causal_counter_consumer_stored_counter_read_01, StoredCounterRead, GraphNode),
        (causal_projection_v16_site_causal_counter_consumer_expected_start_comparison_01, ExpectedStartComparison, GraphNode),
        (causal_projection_v16_site_causal_counter_consumer_checked_advance_01, CheckedAdvance, GraphNode),
    );
    v16_direct_site_proofs!(
        assert_v16_frontier_site,
        FrontierComparisonOperation;
        (causal_projection_v16_site_frontier_comparison_candidate_kind_comparison_01, CandidateKindComparison, GraphNode),
        (causal_projection_v16_site_frontier_comparison_candidate_count_01, CandidateCount, GraphNode),
        (causal_projection_v16_site_frontier_comparison_projection_count_01, ProjectionCount, GraphNode),
        (causal_projection_v16_site_frontier_comparison_base_count_01, BaseCount, GraphNode),
        (causal_projection_v16_site_frontier_comparison_candidate_pull_01, CandidatePull, GraphEdge),
        (causal_projection_v16_site_frontier_comparison_candidate_order_comparison_01, CandidateOrderComparison, GraphEdge),
        (causal_projection_v16_site_frontier_comparison_projection_pull_01, ProjectionPull, GraphNode),
        (causal_projection_v16_site_frontier_comparison_base_pull_01, BasePull, GraphNode),
        (causal_projection_v16_site_frontier_comparison_base_accepted_lookup_01, BaseAcceptedLookup, GraphNode),
        (causal_projection_v16_site_frontier_comparison_expected_source_comparison_01, ExpectedSourceComparison, GraphNode),
        (causal_projection_v16_site_frontier_comparison_frontier_equality_comparison_01, FrontierEqualityComparison, GraphEdge),
    );
}
