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
pub(crate) struct TrustedEpochView {
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

pub(crate) type AcceptedEpochStateParts = (
    BTreeSet<ChangeHash>,
    BTreeMap<ChangeHash, BTreeSet<ChangeHash>>,
    BTreeMap<ActorId, EpochActorState>,
    BTreeMap<ActorId, ChangeHash>,
);

impl TrustedEpochView {
    pub(crate) const fn is_branch_member(self) -> bool {
        self.branch_member
    }

    pub(crate) const fn is_accepted_member(self) -> bool {
        self.accepted_member
    }

    pub(crate) const fn predecessor(self) -> Option<ChangeHash> {
        self.predecessor
    }

    pub(crate) const fn predecessor_is_direct_dependency(self) -> bool {
        self.predecessor_is_direct_dependency
    }

    pub(crate) const fn actor_identity_matches(self) -> bool {
        self.actor_identity_matches
    }

    pub(crate) const fn expected_sequence(self) -> u64 {
        self.expected_sequence
    }

    pub(crate) const fn sequence_matches(self) -> bool {
        self.sequence_matches
    }

    pub(crate) const fn causal_next_op(self) -> u64 {
        self.causal_next_op
    }

    pub(crate) const fn expected_next_matches(self) -> bool {
        self.expected_next_matches
    }
}

impl TrustedEpochProjection<'_> {
    pub(crate) fn candidate_metered<E>(
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
        let expected_sequence = actor.map_or(Some(1), |state| state.last_sequence.checked_add(1));
        observed(ProjectionLookupOperation::ExpectedSequence);
        let expected_sequence =
            expected_sequence.ok_or(MeteredActorStateError::State(ActorStateError::SequenceGap))?;

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

    pub(crate) fn legacy_counter_is_valid(
        self,
        candidate: &ChangeCandidate,
        base_frontier: &BTreeSet<ChangeHash>,
    ) -> bool {
        let mut states = self.actor_states;
        if candidate.operation_count == 0 {
            let mut current_heads = self.frontier_heads;
            current_heads.extend(base_frontier.difference(self.accepted_closure).copied());
            apply_empty_counter(&mut states, candidate, &current_heads).is_ok()
        } else {
            apply_nonempty_counter(&mut states, candidate).is_ok()
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

pub(crate) fn apply_empty_counter(
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
    let next_op = causal_next_op(states);
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

pub(crate) fn apply_nonempty_counter(
    states: &mut BTreeMap<ActorId, EpochActorState>,
    candidate: &ChangeCandidate,
) -> Result<(), ActorStateError> {
    if candidate.operation_count == 0 {
        return Err(ActorStateError::EmptyChange);
    }
    let last_sequence = states
        .get(&candidate.actor)
        .map_or(0, |state| state.last_sequence);
    let next_op = causal_next_op(states);
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

pub(crate) fn validate_actor_predecessor(
    candidate: &ChangeCandidate,
    closure: &std::collections::BTreeSet<ChangeHash>,
    accepted: &BTreeMap<ChangeHash, ChangeCandidate>,
) -> Result<(), ActorStateError> {
    let same_actor = closure
        .iter()
        .filter_map(|hash| accepted.get(hash))
        .filter(|change| change.actor == candidate.actor)
        .collect::<Vec<_>>();
    if same_actor
        .iter()
        .any(|change| change.sequence >= candidate.sequence)
    {
        return Err(ActorStateError::SequenceRollback);
    }
    if candidate.sequence == 1 {
        return if same_actor.is_empty() {
            Ok(())
        } else {
            Err(ActorStateError::SequenceRollback)
        };
    }
    let expected = candidate.sequence - 1;
    let predecessors = same_actor
        .iter()
        .filter(|change| change.sequence == expected)
        .count();
    match predecessors {
        1 => Ok(()),
        0 => Err(ActorStateError::MissingPredecessor),
        _ => Err(ActorStateError::ParallelPredecessor),
    }
}

fn causal_next_op(states: &BTreeMap<ActorId, EpochActorState>) -> u64 {
    // Automerge operation counters are causal Lamport counters. Actor sequence
    // remains actor-local, while a new change starts after the greatest
    // operation visible in its exact dependency closure.
    states
        .values()
        .map(|state| state.next_op)
        .max()
        .unwrap_or(1)
}

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
    mut charge: impl FnMut(WorkCounter) -> Result<(), E>,
) -> Result<TrustedEpochProjection<'a>, MeteredActorStateError<E>> {
    let member_count = source.member_count();
    if member_count != accepted_closure.len() {
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
        charge(WorkCounter::GraphNode).map_err(MeteredActorStateError::Work)?;
        let Some(hash) = source.next_member() else {
            return Err(MeteredActorStateError::State(
                ActorStateError::NoncanonicalInput,
            ));
        };
        if previous_hash.is_some_and(|previous| previous >= hash) {
            return Err(MeteredActorStateError::State(
                ActorStateError::NoncanonicalInput,
            ));
        }
        previous_hash = Some(hash);
        charge(WorkCounter::GraphNode).map_err(MeteredActorStateError::Work)?;
        if !source.accepted_member(&hash) {
            return Err(MeteredActorStateError::State(
                ActorStateError::NoncanonicalInput,
            ));
        }
        charge(WorkCounter::GraphNode).map_err(MeteredActorStateError::Work)?;
        let Some(candidate) = source.candidate(&hash) else {
            return Err(MeteredActorStateError::State(
                ActorStateError::MissingDependency,
            ));
        };
        if candidate.change_hash != hash {
            return Err(MeteredActorStateError::State(
                ActorStateError::NoncanonicalInput,
            ));
        }
        let mut candidate_dependencies = BTreeSet::new();
        let dependency_count = source.dependency_count(candidate);
        let mut previous_dependency = None;
        for index in 0..dependency_count {
            charge(WorkCounter::GraphEdge).map_err(MeteredActorStateError::Work)?;
            let Some(dependency) = source.dependency(candidate, index) else {
                return Err(MeteredActorStateError::State(
                    ActorStateError::NoncanonicalInput,
                ));
            };
            if previous_dependency.is_some_and(|previous| previous >= dependency) {
                return Err(MeteredActorStateError::State(
                    ActorStateError::NoncanonicalInput,
                ));
            }
            previous_dependency = Some(dependency);
            charge(WorkCounter::GraphEdge).map_err(MeteredActorStateError::Work)?;
            if !source.accepted_member(&dependency) {
                return Err(MeteredActorStateError::State(
                    ActorStateError::MissingDependency,
                ));
            }
            candidate_dependencies.insert(dependency);
            depended_on.insert(dependency);
            dependants.entry(dependency).or_default().insert(hash);
        }
        if candidate_dependencies.is_empty() {
            ready.insert(hash);
        }
        remaining_dependencies.insert(hash, dependency_count);
        dependencies.insert(hash, candidate_dependencies);
    }

    let mut states = BTreeMap::<ActorId, EpochActorState>::new();
    let mut frontier_heads = BTreeSet::new();
    let mut writer_contributions = BTreeMap::new();
    let mut causal_next_by_change = BTreeMap::<ChangeHash, u64>::new();
    let mut processed = 0usize;
    while !ready.is_empty() {
        charge(WorkCounter::GraphNode).map_err(MeteredActorStateError::Work)?;
        let Some(hash) = ready.pop_first() else {
            return Err(MeteredActorStateError::State(
                ActorStateError::DependencyCycle,
            ));
        };
        charge(WorkCounter::GraphNode).map_err(MeteredActorStateError::Work)?;
        let Some(candidate) = source.candidate(&hash) else {
            return Err(MeteredActorStateError::State(
                ActorStateError::MissingDependency,
            ));
        };
        if !depended_on.contains(&hash) {
            frontier_heads.insert(hash);
        }
        let expected_sequence = match states.get(&candidate.actor) {
            Some(state) => state
                .last_sequence
                .checked_add(1)
                .ok_or(MeteredActorStateError::State(ActorStateError::SequenceGap))?,
            None => 1,
        };
        if candidate.sequence < expected_sequence {
            return Err(MeteredActorStateError::State(ActorStateError::Equivocation));
        }
        if candidate.sequence != expected_sequence {
            return Err(MeteredActorStateError::State(ActorStateError::SequenceGap));
        }
        let next_op = causal_next_by_change.get(&hash).copied().unwrap_or(1);
        let advanced =
            if candidate.operation_count == 0 {
                if candidate.start_op != next_op {
                    return Err(MeteredActorStateError::State(
                        ActorStateError::OperationCounter,
                    ));
                }
                next_op
            } else {
                if candidate.start_op != next_op {
                    return Err(MeteredActorStateError::State(
                        ActorStateError::OperationCounter,
                    ));
                }
                next_op.checked_add(candidate.operation_count).ok_or(
                    MeteredActorStateError::State(ActorStateError::OperationCounter),
                )?
            };
        states.insert(
            candidate.actor,
            EpochActorState {
                last_sequence: candidate.sequence,
                next_op: advanced,
                highest_change: candidate.change_hash,
            },
        );
        writer_contributions.insert(candidate.actor, candidate.change_hash);
        causal_next_by_change.insert(candidate.change_hash, advanced);
        processed = processed
            .checked_add(1)
            .ok_or(MeteredActorStateError::State(
                ActorStateError::DependencyCycle,
            ))?;
        if let Some(children) = dependants.get(&hash) {
            let mut child_iter = children.iter();
            for _ in 0..children.len() {
                charge(WorkCounter::GraphEdge).map_err(MeteredActorStateError::Work)?;
                let Some(child) = child_iter.next() else {
                    return Err(MeteredActorStateError::State(
                        ActorStateError::DependencyCycle,
                    ));
                };
                let Some(remaining) = remaining_dependencies.get_mut(child) else {
                    return Err(MeteredActorStateError::State(
                        ActorStateError::MissingDependency,
                    ));
                };
                *remaining = remaining
                    .checked_sub(1)
                    .ok_or(MeteredActorStateError::State(
                        ActorStateError::DependencyCycle,
                    ))?;
                causal_next_by_change
                    .entry(*child)
                    .and_modify(|value| *value = (*value).max(advanced))
                    .or_insert(advanced);
                if *remaining == 0 {
                    ready.insert(*child);
                }
            }
        }
    }
    if processed != member_count {
        return Err(MeteredActorStateError::State(
            ActorStateError::DependencyCycle,
        ));
    }
    let causal_next_op = states
        .values()
        .map(|state| state.next_op)
        .max()
        .unwrap_or(1);
    Ok(TrustedEpochProjection {
        branch_membership: changes,
        accepted_closure,
        dependencies,
        frontier_heads,
        actor_states: states,
        writer_contributions,
        causal_next_op,
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use std::cell::{Cell, RefCell};
    use std::collections::BTreeSet;
    use std::rc::Rc;

    use super::{
        ActorStateError, EpochActorState, EpochProjectionSource, MeteredActorStateError,
        ProjectionLookupOperation, TrustedEpochProjection, TrustedEpochView, apply_empty_counter,
        apply_nonempty_counter, build_trusted_epoch_projection, initialize_actor_states,
        initialize_actor_states_metered, validate_actor_predecessor,
    };
    use crate::graph::change_candidate::ChangeCandidate;
    use crate::{ActorId, ChangeHash, Completion, DevicePublicKey, EventId, WorkCounter};
    use std::collections::BTreeMap;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum SourceOperation {
        PullMember(ChangeHash),
        ReadAcceptedMember(ChangeHash),
        ReadCandidate(ChangeHash),
        PullDependency(ChangeHash, usize, ChangeHash),
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum TraversalTrace {
        Charge(WorkCounter),
        Operation(SourceOperation),
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum LookupTrace {
        Charge(WorkCounter),
        Operation(ProjectionLookupOperation),
    }

    struct ObservedEpochProjectionSource<'a> {
        members: Vec<ChangeHash>,
        cursor: usize,
        accepted_closure: &'a BTreeSet<ChangeHash>,
        changes: &'a BTreeMap<ChangeHash, ChangeCandidate>,
        trace: Rc<RefCell<Vec<TraversalTrace>>>,
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
            charges,
            vec![
                WorkCounter::GraphNode,
                WorkCounter::GraphNode,
                WorkCounter::GraphNode,
                WorkCounter::GraphNode,
                WorkCounter::GraphNode,
            ]
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
                            let counter = match index
                                .checked_sub(2)
                                .and_then(|prior| full_trace.get(prior))
                            {
                                Some(TraversalTrace::Operation(SourceOperation::PullMember(
                                    member,
                                ))) if member == hash => Some(WorkCounter::GraphNode),
                                Some(TraversalTrace::Operation(
                                    SourceOperation::PullDependency(_, _, dependency),
                                )) if dependency == hash => Some(WorkCounter::GraphEdge),
                                _ => None,
                            };
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
            assert!(trace.is_empty());
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
        assert_shape::<super::TrustedEpochView>();

        let source = include_str!("actor_state.rs");
        let production = source
            .split_once("#[cfg(test)]")
            .map_or(source, |item| item.0);
        assert_eq!(production.matches("Ok(TrustedEpochProjection {").count(), 1);
        assert!(!production.contains("pub struct TrustedEpochProjection"));
        assert!(!production.contains("pub(crate) struct TrustedEpochProjectionParts"));
        assert!(!production.contains("&mut TrustedEpochProjection"));
        assert!(!production.contains("pub(crate) dependencies:"));
        assert!(!production.contains("pub(crate) actor_states:"));
    }

    #[test]
    fn validate_actor_predecessor_sequence() {
        let first = candidate(1, 1, 1, 1);
        let mut second = candidate(1, 2, 2, 1);
        second.change_hash = ChangeHash::from_bytes([2; 32]);
        let accepted = BTreeMap::from([(first.change_hash, first.clone())]);
        assert_eq!(
            validate_actor_predecessor(&second, &BTreeSet::from([first.change_hash]), &accepted),
            Ok(())
        );
        assert_eq!(
            validate_actor_predecessor(&second, &BTreeSet::new(), &accepted),
            Err(ActorStateError::MissingPredecessor)
        );
        let mut conflict = first.clone();
        conflict.change_hash = ChangeHash::from_bytes([8; 32]);
        let conflicts = BTreeMap::from([
            (first.change_hash, first.clone()),
            (conflict.change_hash, conflict),
        ]);
        assert_eq!(
            validate_actor_predecessor(
                &second,
                &BTreeSet::from([first.change_hash, ChangeHash::from_bytes([8; 32])]),
                &conflicts,
            ),
            Err(ActorStateError::ParallelPredecessor)
        );
        assert_eq!(
            validate_actor_predecessor(&first, &BTreeSet::from([first.change_hash]), &accepted),
            Err(ActorStateError::SequenceRollback)
        );
    }

    #[test]
    fn validate_next_op_for_nonempty_changes() {
        let actor = ActorId::from_bytes([1; 32]);
        let mut states = BTreeMap::new();
        let first = candidate(1, 1, 1, 2);
        assert_eq!(apply_nonempty_counter(&mut states, &first), Ok(()));
        assert_eq!(states[&actor].next_op, 3);
        let second = candidate(1, 2, 3, 1);
        assert_eq!(apply_nonempty_counter(&mut states, &second), Ok(()));

        let mut gap = candidate(1, 3, 5, 1);
        assert_eq!(
            apply_nonempty_counter(&mut states.clone(), &gap),
            Err(ActorStateError::OperationCounter)
        );
        gap.start_op = 2;
        assert_eq!(
            apply_nonempty_counter(&mut states.clone(), &gap),
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
            apply_nonempty_counter(&mut overflow_states, &overflow),
            Err(ActorStateError::OperationCounter)
        );
        assert_eq!(
            apply_nonempty_counter(&mut states, &candidate(2, 1, 4, 1)),
            Ok(())
        );
    }

    #[test]
    fn nonempty_change_advances_counter() {
        let actor = ActorId::from_bytes([5; 32]);
        let first = candidate(5, 1, 1, 2);
        let second = candidate(5, 2, 3, 4);
        let mut states = BTreeMap::new();
        assert_eq!(apply_nonempty_counter(&mut states, &first), Ok(()));
        assert_eq!(apply_nonempty_counter(&mut states, &second), Ok(()));
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
            apply_nonempty_counter(&mut overflow, &candidate(5, 3, u64::MAX, 1)),
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
                apply_empty_counter(&mut states, &candidate, &current_heads)
            } else {
                apply_nonempty_counter(&mut states, &candidate)
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
        assert_eq!(apply_nonempty_counter(&mut states, &first), Ok(()));
        let first_head = ChangeHash::from_bytes([7; 32]);
        let mut empty = candidate(1, 2, 3, 0);
        empty.change_hash = ChangeHash::from_bytes([2; 32]);
        empty.dependencies = vec![first_head].into();
        assert_eq!(
            apply_empty_counter(&mut states, &empty, &BTreeSet::from([first_head])),
            Ok(())
        );
        assert_eq!(states[&ActorId::from_bytes([1; 32])].next_op, 3);

        let mut second_empty = candidate(1, 3, 3, 0);
        second_empty.change_hash = ChangeHash::from_bytes([3; 32]);
        second_empty.dependencies = vec![empty.change_hash].into();
        assert_eq!(
            apply_empty_counter(
                &mut states,
                &second_empty,
                &BTreeSet::from([empty.change_hash])
            ),
            Ok(())
        );
        let mut wrong_start = candidate(1, 4, 4, 0);
        wrong_start.dependencies = vec![second_empty.change_hash].into();
        assert_eq!(
            apply_empty_counter(
                &mut states.clone(),
                &wrong_start,
                &BTreeSet::from([second_empty.change_hash])
            ),
            Err(ActorStateError::OperationCounter)
        );
        assert_eq!(
            apply_empty_counter(&mut states, &wrong_start, &BTreeSet::new()),
            Err(ActorStateError::DependencyFrontier)
        );
    }

    #[test]
    #[ignore = "open remediation-v12 resource-accounting reproduction"]
    fn finding_100_actor_predecessor_scan_reproduction() {
        let mut accepted = BTreeMap::new();
        let mut closure = BTreeSet::new();
        for sequence in 1..=64 {
            let change = candidate(1, sequence, sequence, 1);
            closure.insert(change.change_hash);
            accepted.insert(change.change_hash, change);
        }
        let mut unrelated = candidate(2, 1, 1, 1);
        unrelated.change_hash = ChangeHash::from_bytes([200; 32]);
        closure.insert(unrelated.change_hash);
        accepted.insert(unrelated.change_hash, unrelated.clone());
        let mut next = candidate(1, 65, 65, 1);
        next.dependencies = vec![unrelated.change_hash].into();

        assert_eq!(
            validate_actor_predecessor(&next, &closure, &accepted),
            Ok(()),
            "the actor predecessor is accepted transitively and need not be a direct dependency"
        );

        let source = include_str!("actor_state.rs");
        assert!(
            !source.contains(".collect::<Vec<_>>()"),
            "unmetered actor predecessor collection remains"
        );
    }

    #[test]
    #[ignore = "open remediation-v12 resource-accounting reproduction"]
    fn finding_100_causal_next_op_scan_reproduction() {
        let mut states = BTreeMap::new();
        for actor in 1..=64 {
            states.insert(
                ActorId::from_bytes([actor; 32]),
                EpochActorState {
                    last_sequence: 1,
                    next_op: u64::from(actor) + 1,
                    highest_change: ChangeHash::from_bytes([actor; 32]),
                },
            );
        }
        let next = candidate(100, 1, 65, 1);
        assert_eq!(apply_nonempty_counter(&mut states, &next), Ok(()));

        let source = include_str!("actor_state.rs");
        assert!(
            !source.contains("fn causal_next_op(states: &BTreeMap<ActorId, EpochActorState>)"),
            "unmetered causal next-op scan remains"
        );
    }

    #[test]
    #[ignore = "open remediation-v12 resource-accounting reproduction"]
    fn finding_100_empty_frontier_work_reproduction() {
        let current_heads = BTreeSet::from([
            ChangeHash::from_bytes([10; 32]),
            ChangeHash::from_bytes([20; 32]),
            ChangeHash::from_bytes([30; 32]),
        ]);
        let mut empty = candidate(1, 1, 1, 0);
        empty.dependencies = current_heads.iter().copied().collect::<Vec<_>>().into();
        assert_eq!(
            apply_empty_counter(&mut BTreeMap::new(), &empty, &current_heads),
            Ok(())
        );

        let source = include_str!("actor_state.rs");
        assert!(
            !source.contains(".collect::<std::collections::BTreeSet<_>>()"),
            "unmetered empty-frontier allocation remains"
        );
    }
}
