use std::borrow::Borrow;
use std::collections::{BTreeMap, BTreeSet};

use super::change_candidate::ChangeCandidate;
use crate::{CancellationCheck, ChangeHash, WorkBudget, WorkCounter};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Schedule {
    pub(crate) ordered: Vec<ChangeHash>,
    pub(crate) pending: BTreeSet<ChangeHash>,
    pub(crate) missing_dependencies: BTreeSet<ChangeHash>,
    pub(crate) cyclic: BTreeSet<ChangeHash>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ScheduleError {
    BudgetExhausted,
    Cancelled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScheduleOperation {
    ScheduleConstruction,
    RemainingMapConstruction,
    CandidatePull,
    RemainingInsert,
    CandidateHashSetConstruction,
    RemainingKeyPull,
    CandidateHashInsert,
    UnresolvedMapConstruction,
    DependantMapConstruction,
    RemainingEntryPull,
    DependencyPull,
    AcceptedLookup,
    UnresolvedIncrement,
    CandidateLookup,
    DependantLookup,
    DependantBucketInsert,
    DependantInsert,
    UnresolvedInsert,
    ReadySetConstruction,
    UnresolvedPull,
    ReadinessComparison,
    ReadyInsert,
    OrderedVecConstruction,
    ReadyPeek,
    ReadyTieComparison,
    ReadyPop,
    RemainingRemove,
    OrderedPush,
    ChildrenLookup,
    ChildPull,
    UnresolvedLookup,
    UnresolvedDecrement,
    MissingSetConstruction,
    RemainingValuePull,
    MissingCandidateLookup,
    MissingAcceptedLookup,
    MissingInsert,
    PendingSetConstruction,
    MissingLookup,
    PendingInsert,
    BlockedStackConstruction,
    PendingPull,
    BlockedPush,
    BlockedPull,
    RemainingLookup,
    PendingLookup,
    CyclicSetConstruction,
    CyclicCandidatePull,
    CyclicPendingLookup,
    CyclicInsert,
    ResultPublication,
}

pub(crate) fn schedule_candidates(
    candidates: impl IntoIterator<Item = ChangeCandidate>,
    accepted_base: impl Borrow<BTreeSet<ChangeHash>>,
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
) -> Result<Schedule, ScheduleError> {
    schedule_candidates_observed(
        candidates,
        accepted_base,
        |counter| charge_schedule_work(budget, cancellation, counter),
        |_| {},
    )
}

fn schedule_candidates_observed<E>(
    candidates: impl IntoIterator<Item = ChangeCandidate>,
    accepted_base: impl Borrow<BTreeSet<ChangeHash>>,
    mut charge: impl FnMut(WorkCounter) -> Result<(), E>,
    mut observed: impl FnMut(ScheduleOperation),
) -> Result<Schedule, E> {
    let accepted_base = accepted_base.borrow();
    schedule_operation(
        WorkCounter::GraphNode,
        ScheduleOperation::ScheduleConstruction,
        &mut charge,
        &mut observed,
        || (),
    )?;
    let mut remaining = schedule_operation(
        WorkCounter::GraphNode,
        ScheduleOperation::RemainingMapConstruction,
        &mut charge,
        &mut observed,
        BTreeMap::new,
    )?;
    let mut candidates = candidates.into_iter();
    loop {
        let candidate = schedule_operation(
            WorkCounter::GraphNode,
            ScheduleOperation::CandidatePull,
            &mut charge,
            &mut observed,
            || candidates.next(),
        )?;
        let Some(candidate) = candidate else { break };
        schedule_operation(
            WorkCounter::GraphNode,
            ScheduleOperation::RemainingInsert,
            &mut charge,
            &mut observed,
            || remaining.insert(candidate.change_hash, candidate),
        )?;
    }
    let mut candidate_hashes = schedule_operation(
        WorkCounter::GraphNode,
        ScheduleOperation::CandidateHashSetConstruction,
        &mut charge,
        &mut observed,
        BTreeSet::new,
    )?;
    let mut remaining_keys = remaining.keys();
    for _ in 0..remaining.len() {
        let hash = schedule_operation(
            WorkCounter::GraphNode,
            ScheduleOperation::RemainingKeyPull,
            &mut charge,
            &mut observed,
            || remaining_keys.next().copied(),
        )?;
        let Some(hash) = hash else { break };
        schedule_operation(
            WorkCounter::GraphNode,
            ScheduleOperation::CandidateHashInsert,
            &mut charge,
            &mut observed,
            || candidate_hashes.insert(hash),
        )?;
    }
    let mut unresolved = schedule_operation(
        WorkCounter::GraphNode,
        ScheduleOperation::UnresolvedMapConstruction,
        &mut charge,
        &mut observed,
        BTreeMap::new,
    )?;
    let mut dependants = schedule_operation(
        WorkCounter::GraphEdge,
        ScheduleOperation::DependantMapConstruction,
        &mut charge,
        &mut observed,
        BTreeMap::<ChangeHash, BTreeSet<ChangeHash>>::new,
    )?;
    let mut remaining_entries = remaining.iter();
    for _ in 0..remaining.len() {
        let entry = schedule_operation(
            WorkCounter::GraphNode,
            ScheduleOperation::RemainingEntryPull,
            &mut charge,
            &mut observed,
            || remaining_entries.next(),
        )?;
        let Some((hash, candidate)) = entry else {
            break;
        };
        let mut count = 0_usize;
        let mut dependencies = candidate.dependencies.iter();
        for _ in 0..candidate.dependencies.len() {
            let dependency = schedule_operation(
                WorkCounter::GraphEdge,
                ScheduleOperation::DependencyPull,
                &mut charge,
                &mut observed,
                || dependencies.next().copied(),
            )?;
            let Some(dependency) = dependency else { break };
            let accepted = schedule_operation(
                WorkCounter::GraphEdge,
                ScheduleOperation::AcceptedLookup,
                &mut charge,
                &mut observed,
                || accepted_base.contains(&dependency),
            )?;
            if !accepted {
                schedule_operation(
                    WorkCounter::GraphEdge,
                    ScheduleOperation::UnresolvedIncrement,
                    &mut charge,
                    &mut observed,
                    || count = count.saturating_add(1),
                )?;
            }
            let is_candidate = schedule_operation(
                WorkCounter::GraphEdge,
                ScheduleOperation::CandidateLookup,
                &mut charge,
                &mut observed,
                || candidate_hashes.contains(&dependency),
            )?;
            if is_candidate {
                let has_bucket = schedule_operation(
                    WorkCounter::GraphEdge,
                    ScheduleOperation::DependantLookup,
                    &mut charge,
                    &mut observed,
                    || dependants.contains_key(&dependency),
                )?;
                if !has_bucket {
                    schedule_operation(
                        WorkCounter::GraphEdge,
                        ScheduleOperation::DependantBucketInsert,
                        &mut charge,
                        &mut observed,
                        || dependants.insert(dependency, BTreeSet::new()),
                    )?;
                }
                let children = schedule_operation(
                    WorkCounter::GraphEdge,
                    ScheduleOperation::DependantLookup,
                    &mut charge,
                    &mut observed,
                    || dependants.get_mut(&dependency),
                )?;
                if let Some(children) = children {
                    schedule_operation(
                        WorkCounter::GraphEdge,
                        ScheduleOperation::DependantInsert,
                        &mut charge,
                        &mut observed,
                        || children.insert(*hash),
                    )?;
                }
            }
        }
        schedule_operation(
            WorkCounter::GraphNode,
            ScheduleOperation::UnresolvedInsert,
            &mut charge,
            &mut observed,
            || unresolved.insert(*hash, count),
        )?;
    }
    let mut ready = schedule_operation(
        WorkCounter::GraphNode,
        ScheduleOperation::ReadySetConstruction,
        &mut charge,
        &mut observed,
        BTreeSet::new,
    )?;
    let mut unresolved_items = unresolved.iter();
    for _ in 0..unresolved.len() {
        let item = schedule_operation(
            WorkCounter::GraphNode,
            ScheduleOperation::UnresolvedPull,
            &mut charge,
            &mut observed,
            || unresolved_items.next().map(|(hash, count)| (*hash, *count)),
        )?;
        let Some((hash, count)) = item else { break };
        let is_ready = schedule_operation(
            WorkCounter::GraphNode,
            ScheduleOperation::ReadinessComparison,
            &mut charge,
            &mut observed,
            || count == 0,
        )?;
        if is_ready {
            schedule_operation(
                WorkCounter::GraphNode,
                ScheduleOperation::ReadyInsert,
                &mut charge,
                &mut observed,
                || ready.insert(hash),
            )?;
        }
    }
    let mut ordered = schedule_operation(
        WorkCounter::GraphNode,
        ScheduleOperation::OrderedVecConstruction,
        &mut charge,
        &mut observed,
        Vec::new,
    )?;
    loop {
        let first = schedule_operation(
            WorkCounter::GraphNode,
            ScheduleOperation::ReadyPeek,
            &mut charge,
            &mut observed,
            || ready.first().copied(),
        )?;
        let Some(first) = first else { break };
        schedule_operation(
            WorkCounter::GraphNode,
            ScheduleOperation::ReadyTieComparison,
            &mut charge,
            &mut observed,
            || ready.iter().nth(1).is_none_or(|next| first < *next),
        )?;
        let hash = schedule_operation(
            WorkCounter::GraphNode,
            ScheduleOperation::ReadyPop,
            &mut charge,
            &mut observed,
            || ready.pop_first(),
        )?;
        let Some(hash) = hash else { break };
        schedule_operation(
            WorkCounter::GraphNode,
            ScheduleOperation::RemainingRemove,
            &mut charge,
            &mut observed,
            || remaining.remove(&hash),
        )?;
        schedule_operation(
            WorkCounter::GraphNode,
            ScheduleOperation::OrderedPush,
            &mut charge,
            &mut observed,
            || ordered.push(hash),
        )?;
        let children = schedule_operation(
            WorkCounter::GraphEdge,
            ScheduleOperation::ChildrenLookup,
            &mut charge,
            &mut observed,
            || dependants.get(&hash),
        )?;
        let Some(children) = children else { continue };
        let mut children = children.iter();
        let child_count = children.len();
        for _ in 0..child_count {
            let child = schedule_operation(
                WorkCounter::GraphEdge,
                ScheduleOperation::ChildPull,
                &mut charge,
                &mut observed,
                || children.next().copied(),
            )?;
            let Some(child) = child else { break };
            let count = schedule_operation(
                WorkCounter::GraphNode,
                ScheduleOperation::UnresolvedLookup,
                &mut charge,
                &mut observed,
                || unresolved.get_mut(&child),
            )?;
            let Some(count) = count else { continue };
            schedule_operation(
                WorkCounter::GraphNode,
                ScheduleOperation::UnresolvedDecrement,
                &mut charge,
                &mut observed,
                || *count = count.saturating_sub(1),
            )?;
            let is_ready = schedule_operation(
                WorkCounter::GraphNode,
                ScheduleOperation::ReadinessComparison,
                &mut charge,
                &mut observed,
                || *count == 0,
            )?;
            if is_ready {
                schedule_operation(
                    WorkCounter::GraphNode,
                    ScheduleOperation::ReadyInsert,
                    &mut charge,
                    &mut observed,
                    || ready.insert(child),
                )?;
            }
        }
    }
    let mut missing_dependencies = schedule_operation(
        WorkCounter::GraphEdge,
        ScheduleOperation::MissingSetConstruction,
        &mut charge,
        &mut observed,
        BTreeSet::new,
    )?;
    let mut remaining_values = remaining.values();
    for _ in 0..remaining.len() {
        let candidate = schedule_operation(
            WorkCounter::GraphNode,
            ScheduleOperation::RemainingValuePull,
            &mut charge,
            &mut observed,
            || remaining_values.next(),
        )?;
        let Some(candidate) = candidate else { break };
        let mut dependencies = candidate.dependencies.iter();
        for _ in 0..candidate.dependencies.len() {
            let dependency = schedule_operation(
                WorkCounter::GraphEdge,
                ScheduleOperation::DependencyPull,
                &mut charge,
                &mut observed,
                || dependencies.next().copied(),
            )?;
            let Some(dependency) = dependency else { break };
            let candidate_known = schedule_operation(
                WorkCounter::GraphEdge,
                ScheduleOperation::MissingCandidateLookup,
                &mut charge,
                &mut observed,
                || candidate_hashes.contains(&dependency),
            )?;
            let accepted = schedule_operation(
                WorkCounter::GraphEdge,
                ScheduleOperation::MissingAcceptedLookup,
                &mut charge,
                &mut observed,
                || accepted_base.contains(&dependency),
            )?;
            if !candidate_known && !accepted {
                schedule_operation(
                    WorkCounter::GraphEdge,
                    ScheduleOperation::MissingInsert,
                    &mut charge,
                    &mut observed,
                    || missing_dependencies.insert(dependency),
                )?;
            }
        }
    }
    let mut pending = schedule_operation(
        WorkCounter::GraphNode,
        ScheduleOperation::PendingSetConstruction,
        &mut charge,
        &mut observed,
        BTreeSet::new,
    )?;
    let mut remaining_entries = remaining.iter();
    for _ in 0..remaining.len() {
        let entry = schedule_operation(
            WorkCounter::GraphNode,
            ScheduleOperation::RemainingEntryPull,
            &mut charge,
            &mut observed,
            || remaining_entries.next(),
        )?;
        let Some((hash, candidate)) = entry else {
            break;
        };
        let mut is_pending = false;
        let mut dependencies = candidate.dependencies.iter();
        for _ in 0..candidate.dependencies.len() {
            let dependency = schedule_operation(
                WorkCounter::GraphEdge,
                ScheduleOperation::DependencyPull,
                &mut charge,
                &mut observed,
                || dependencies.next().copied(),
            )?;
            let Some(dependency) = dependency else { break };
            is_pending = schedule_operation(
                WorkCounter::GraphEdge,
                ScheduleOperation::MissingLookup,
                &mut charge,
                &mut observed,
                || missing_dependencies.contains(&dependency),
            )?;
            if is_pending {
                break;
            }
        }
        if is_pending {
            schedule_operation(
                WorkCounter::GraphNode,
                ScheduleOperation::PendingInsert,
                &mut charge,
                &mut observed,
                || pending.insert(*hash),
            )?;
        }
    }
    let mut blocked = schedule_operation(
        WorkCounter::GraphNode,
        ScheduleOperation::BlockedStackConstruction,
        &mut charge,
        &mut observed,
        Vec::new,
    )?;
    let mut pending_items = pending.iter();
    for _ in 0..pending.len() {
        let hash = schedule_operation(
            WorkCounter::GraphNode,
            ScheduleOperation::PendingPull,
            &mut charge,
            &mut observed,
            || pending_items.next().copied(),
        )?;
        let Some(hash) = hash else { break };
        schedule_operation(
            WorkCounter::GraphNode,
            ScheduleOperation::BlockedPush,
            &mut charge,
            &mut observed,
            || blocked.push(hash),
        )?;
    }
    loop {
        let hash = schedule_operation(
            WorkCounter::GraphNode,
            ScheduleOperation::BlockedPull,
            &mut charge,
            &mut observed,
            || blocked.pop(),
        )?;
        let Some(hash) = hash else { break };
        let children = schedule_operation(
            WorkCounter::GraphEdge,
            ScheduleOperation::ChildrenLookup,
            &mut charge,
            &mut observed,
            || dependants.get(&hash),
        )?;
        let Some(children) = children else { continue };
        let mut children = children.iter();
        let child_count = children.len();
        for _ in 0..child_count {
            let child = schedule_operation(
                WorkCounter::GraphEdge,
                ScheduleOperation::ChildPull,
                &mut charge,
                &mut observed,
                || children.next().copied(),
            )?;
            let Some(child) = child else { break };
            let remains = schedule_operation(
                WorkCounter::GraphNode,
                ScheduleOperation::RemainingLookup,
                &mut charge,
                &mut observed,
                || remaining.contains_key(&child),
            )?;
            let already_pending = schedule_operation(
                WorkCounter::GraphNode,
                ScheduleOperation::PendingLookup,
                &mut charge,
                &mut observed,
                || pending.contains(&child),
            )?;
            if remains && !already_pending {
                schedule_operation(
                    WorkCounter::GraphNode,
                    ScheduleOperation::PendingInsert,
                    &mut charge,
                    &mut observed,
                    || pending.insert(child),
                )?;
                schedule_operation(
                    WorkCounter::GraphNode,
                    ScheduleOperation::BlockedPush,
                    &mut charge,
                    &mut observed,
                    || blocked.push(child),
                )?;
            }
        }
    }
    let mut cyclic = schedule_operation(
        WorkCounter::GraphNode,
        ScheduleOperation::CyclicSetConstruction,
        &mut charge,
        &mut observed,
        BTreeSet::new,
    )?;
    let mut remaining_keys = remaining.keys();
    for _ in 0..remaining.len() {
        let hash = schedule_operation(
            WorkCounter::GraphNode,
            ScheduleOperation::CyclicCandidatePull,
            &mut charge,
            &mut observed,
            || remaining_keys.next().copied(),
        )?;
        let Some(hash) = hash else { break };
        let is_pending = schedule_operation(
            WorkCounter::GraphNode,
            ScheduleOperation::CyclicPendingLookup,
            &mut charge,
            &mut observed,
            || pending.contains(&hash),
        )?;
        if !is_pending {
            schedule_operation(
                WorkCounter::GraphNode,
                ScheduleOperation::CyclicInsert,
                &mut charge,
                &mut observed,
                || cyclic.insert(hash),
            )?;
        }
    }
    schedule_operation(
        WorkCounter::GraphNode,
        ScheduleOperation::ResultPublication,
        &mut charge,
        &mut observed,
        || Schedule {
            ordered,
            pending,
            missing_dependencies,
            cyclic,
        },
    )
}

fn schedule_operation<T, E>(
    counter: WorkCounter,
    operation: ScheduleOperation,
    charge: &mut impl FnMut(WorkCounter) -> Result<(), E>,
    observed: &mut impl FnMut(ScheduleOperation),
    target: impl FnOnce() -> T,
) -> Result<T, E> {
    charge(counter)?;
    let value = target();
    observed(operation);
    Ok(value)
}

fn charge_schedule_work(
    budget: &mut WorkBudget,
    cancellation: &impl CancellationCheck,
    counter: WorkCounter,
) -> Result<(), ScheduleError> {
    if cancellation.is_cancelled() {
        return Err(ScheduleError::Cancelled);
    }
    budget
        .charge(counter, 1)
        .map_err(|_| ScheduleError::BudgetExhausted)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{
        ScheduleError, ScheduleOperation, schedule_candidates, schedule_candidates_observed,
    };
    use crate::graph::actor_state::tests::candidate;
    use crate::{ChangeHash, NeverCancelled, WorkBudget, WorkCounter};

    #[test]
    fn implement_deterministic_candidate_scheduling() {
        let mut low = candidate(1, 1, 1, 1);
        low.change_hash = ChangeHash::from_bytes([1; 32]);
        let mut high = candidate(2, 1, 1, 1);
        high.change_hash = ChangeHash::from_bytes([2; 32]);
        let mut dependant = candidate(1, 2, 2, 1);
        dependant.change_hash = ChangeHash::from_bytes([3; 32]);
        dependant.dependencies = vec![high.change_hash].into();
        let evaluate = |candidates| {
            schedule_candidates(
                candidates,
                BTreeSet::new(),
                &mut WorkBudget::new(0, 10_000),
                &NeverCancelled,
            )
        };
        let first = evaluate(vec![dependant.clone(), high.clone(), low.clone()]);
        let second = evaluate(vec![low.clone(), high.clone(), dependant.clone()]);
        assert_eq!(first, second);
        assert_eq!(
            first.map(|schedule| schedule.ordered),
            Ok(vec![
                low.change_hash,
                high.change_hash,
                dependant.change_hash
            ])
        );
        let mut measured = WorkBudget::new(0, 10_000);
        let schedule = schedule_candidates(
            [high.clone(), dependant.clone()],
            BTreeSet::new(),
            &mut measured,
            &NeverCancelled,
        );
        assert!(schedule.is_ok());
        assert!(measured.consumed().get(WorkCounter::GraphNode) > 4);
        assert!(measured.consumed().get(WorkCounter::GraphEdge) > 2);
        let missing_hash = ChangeHash::from_bytes([9; 32]);
        let mut missing = high.clone();
        missing.dependencies = vec![missing_hash].into();
        let missing_schedule = evaluate(vec![missing]).map(|schedule| {
            (
                schedule.pending,
                schedule.missing_dependencies,
                schedule.cyclic,
            )
        });
        assert_eq!(
            missing_schedule,
            Ok((
                BTreeSet::from([high.change_hash]),
                BTreeSet::from([missing_hash]),
                BTreeSet::new(),
            ))
        );
        let mut cycle_low = low.clone();
        cycle_low.dependencies = vec![high.change_hash].into();
        let mut cycle_high = high.clone();
        cycle_high.dependencies = vec![low.change_hash].into();
        assert_eq!(
            evaluate(vec![cycle_high, cycle_low])
                .map(|schedule| (schedule.pending, schedule.cyclic)),
            Ok((
                BTreeSet::new(),
                BTreeSet::from([low.change_hash, high.change_hash]),
            ))
        );
        let mut fan_out = candidate(3, 1, 1, 1);
        fan_out.change_hash = ChangeHash::from_bytes([4; 32]);
        fan_out.dependencies = vec![low.change_hash].into();
        let mut fan_in = candidate(4, 1, 1, 1);
        fan_in.change_hash = ChangeHash::from_bytes([5; 32]);
        fan_in.dependencies = vec![dependant.change_hash, fan_out.change_hash].into();
        assert_eq!(
            evaluate(vec![
                fan_in.clone(),
                fan_out.clone(),
                dependant.clone(),
                high.clone(),
                low.clone(),
            ])
            .map(|schedule| schedule.ordered),
            Ok(vec![
                ChangeHash::from_bytes([1; 32]),
                ChangeHash::from_bytes([2; 32]),
                ChangeHash::from_bytes([3; 32]),
                ChangeHash::from_bytes([4; 32]),
                ChangeHash::from_bytes([5; 32]),
            ])
        );
        assert_eq!(
            schedule_candidates(
                [low.clone()],
                BTreeSet::new(),
                &mut WorkBudget::new(0, 0),
                &NeverCancelled,
            ),
            Err(ScheduleError::BudgetExhausted)
        );
        assert_eq!(
            schedule_candidates([low], BTreeSet::new(), &mut WorkBudget::new(0, 10), &|| {
                true
            },),
            Err(ScheduleError::Cancelled)
        );
    }

    #[test]
    fn dependency_cycle_is_invalid() {
        let evaluate = |candidates| {
            schedule_candidates(
                candidates,
                BTreeSet::new(),
                &mut WorkBudget::new(0, 100),
                &NeverCancelled,
            )
        };
        let mut left = candidate(1, 1, 1, 1);
        left.change_hash = ChangeHash::from_bytes([1; 32]);
        let mut right = candidate(2, 1, 1, 1);
        right.change_hash = ChangeHash::from_bytes([2; 32]);
        left.dependencies = vec![right.change_hash].into();
        right.dependencies = vec![left.change_hash].into();
        let mut descendant = candidate(3, 1, 1, 1);
        descendant.change_hash = ChangeHash::from_bytes([3; 32]);
        descendant.dependencies = vec![right.change_hash].into();
        assert_eq!(
            evaluate(vec![descendant.clone(), right.clone(), left.clone()])
                .map(|schedule| schedule.cyclic),
            Ok(BTreeSet::from([
                left.change_hash,
                right.change_hash,
                descendant.change_hash,
            ]))
        );

        let mut third = candidate(3, 1, 1, 1);
        third.change_hash = ChangeHash::from_bytes([4; 32]);
        right.dependencies = vec![third.change_hash].into();
        third.dependencies = vec![left.change_hash].into();
        assert_eq!(
            evaluate(vec![third.clone(), right, left.clone()]).map(|schedule| schedule.cyclic),
            Ok(BTreeSet::from([
                left.change_hash,
                ChangeHash::from_bytes([2; 32]),
                third.change_hash,
            ]))
        );

        let mut self_cycle = candidate(4, 1, 1, 1);
        self_cycle.change_hash = ChangeHash::from_bytes([5; 32]);
        self_cycle.dependencies = vec![self_cycle.change_hash].into();
        assert_eq!(
            evaluate(vec![self_cycle.clone()]).map(|schedule| schedule.cyclic),
            Ok(BTreeSet::from([self_cycle.change_hash]))
        );
    }

    #[test]
    fn scheduling_charges_immediately_before_every_operation_and_preserves_typed_stops() {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        enum Stop {
            BudgetExhausted,
            Cancelled,
        }
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        enum Trace {
            Charge(WorkCounter),
            Operation(ScheduleOperation),
        }
        let make = |value: u8, dependencies: Vec<ChangeHash>| {
            let mut item = candidate(value, 1, 1, 1);
            item.change_hash = ChangeHash::from_bytes([value; 32]);
            item.dependencies = dependencies.into();
            item
        };
        let missing = ChangeHash::from_bytes([99; 32]);
        let accepted = ChangeHash::from_bytes([100; 32]);
        let inputs = vec![
            make(1, vec![]),
            make(2, vec![]),
            make(3, vec![ChangeHash::from_bytes([1; 32])]),
            make(4, vec![ChangeHash::from_bytes([1; 32])]),
            make(
                5,
                vec![
                    ChangeHash::from_bytes([3; 32]),
                    ChangeHash::from_bytes([4; 32]),
                ],
            ),
            make(6, vec![missing]),
            make(7, vec![ChangeHash::from_bytes([6; 32])]),
            make(8, vec![ChangeHash::from_bytes([9; 32])]),
            make(9, vec![ChangeHash::from_bytes([8; 32])]),
            make(10, vec![accepted]),
        ];
        let accepted_base = BTreeSet::from([accepted]);
        let trace = std::cell::RefCell::new(Vec::new());
        let ample = schedule_candidates_observed(
            inputs.clone(),
            &accepted_base,
            |counter| {
                trace.borrow_mut().push(Trace::Charge(counter));
                Ok::<_, Stop>(())
            },
            |operation| trace.borrow_mut().push(Trace::Operation(operation)),
        );
        assert!(ample.is_ok());
        let Ok(ample) = ample else { return };
        let trace = trace.into_inner();
        assert!(!trace.is_empty() && trace.len().is_multiple_of(2));
        for pair in trace.chunks_exact(2) {
            assert!(matches!(pair, [Trace::Charge(_), Trace::Operation(_)]));
        }
        let operations = trace
            .iter()
            .filter_map(|item| match item {
                Trace::Charge(_) => None,
                Trace::Operation(operation) => Some(*operation),
            })
            .collect::<Vec<_>>();
        for required in [
            ScheduleOperation::ScheduleConstruction,
            ScheduleOperation::RemainingMapConstruction,
            ScheduleOperation::CandidatePull,
            ScheduleOperation::RemainingInsert,
            ScheduleOperation::CandidateHashSetConstruction,
            ScheduleOperation::RemainingKeyPull,
            ScheduleOperation::CandidateHashInsert,
            ScheduleOperation::UnresolvedMapConstruction,
            ScheduleOperation::DependantMapConstruction,
            ScheduleOperation::RemainingEntryPull,
            ScheduleOperation::DependencyPull,
            ScheduleOperation::AcceptedLookup,
            ScheduleOperation::UnresolvedIncrement,
            ScheduleOperation::CandidateLookup,
            ScheduleOperation::DependantLookup,
            ScheduleOperation::DependantBucketInsert,
            ScheduleOperation::DependantInsert,
            ScheduleOperation::UnresolvedInsert,
            ScheduleOperation::ReadySetConstruction,
            ScheduleOperation::UnresolvedPull,
            ScheduleOperation::ReadinessComparison,
            ScheduleOperation::ReadyInsert,
            ScheduleOperation::OrderedVecConstruction,
            ScheduleOperation::ReadyPeek,
            ScheduleOperation::ReadyTieComparison,
            ScheduleOperation::ReadyPop,
            ScheduleOperation::RemainingRemove,
            ScheduleOperation::OrderedPush,
            ScheduleOperation::ChildrenLookup,
            ScheduleOperation::ChildPull,
            ScheduleOperation::UnresolvedLookup,
            ScheduleOperation::UnresolvedDecrement,
            ScheduleOperation::MissingSetConstruction,
            ScheduleOperation::RemainingValuePull,
            ScheduleOperation::MissingCandidateLookup,
            ScheduleOperation::MissingAcceptedLookup,
            ScheduleOperation::MissingInsert,
            ScheduleOperation::PendingSetConstruction,
            ScheduleOperation::MissingLookup,
            ScheduleOperation::PendingInsert,
            ScheduleOperation::BlockedStackConstruction,
            ScheduleOperation::PendingPull,
            ScheduleOperation::BlockedPush,
            ScheduleOperation::BlockedPull,
            ScheduleOperation::RemainingLookup,
            ScheduleOperation::PendingLookup,
            ScheduleOperation::CyclicSetConstruction,
            ScheduleOperation::CyclicCandidatePull,
            ScheduleOperation::CyclicPendingLookup,
            ScheduleOperation::CyclicInsert,
            ScheduleOperation::ResultPublication,
        ] {
            assert!(operations.contains(&required), "missing {required:?}");
        }

        for allowance in 0..operations.len() {
            for stop in [Stop::BudgetExhausted, Stop::Cancelled] {
                let mut successful = 0_usize;
                let mut observed = Vec::new();
                let result = schedule_candidates_observed(
                    inputs.clone(),
                    &accepted_base,
                    |_| {
                        if successful == allowance {
                            return Err(stop);
                        }
                        successful += 1;
                        Ok(())
                    },
                    |operation| observed.push(operation),
                );
                assert_eq!(result, Err(stop), "{stop:?} at {allowance}");
                assert_eq!(successful, allowance);
                assert_eq!(observed, operations[..allowance]);
            }
        }
        let reverse = schedule_candidates_observed(
            inputs.into_iter().rev(),
            &accepted_base,
            |_| Ok::<_, Stop>(()),
            |_| {},
        );
        assert_eq!(reverse, Ok(ample));
    }

    #[test]
    fn finding_100_schedule_readiness_work_reproduction() {
        let mut inputs = (1..=64)
            .rev()
            .map(|value| {
                let mut item = candidate(value, 1, 1, 1);
                item.change_hash = ChangeHash::from_bytes([value; 32]);
                item
            })
            .collect::<Vec<_>>();
        let expected = (1..=64)
            .map(|value| ChangeHash::from_bytes([value; 32]))
            .collect::<Vec<_>>();
        let scheduled = schedule_candidates(
            inputs.drain(..),
            BTreeSet::new(),
            &mut WorkBudget::new(0, 10_000),
            &NeverCancelled,
        );
        assert_eq!(scheduled.map(|value| value.ordered), Ok(expected));

        let source = include_str!("schedule.rs");
        let prohibited = [
            ["remaining.keys().copied().", "collect::<BTreeSet<_>>()"].concat(),
            [".collect::<", "BTreeSet<_>>();"].concat(),
            ["pending.iter().copied().", "collect::<Vec<_>>()"].concat(),
        ];
        assert!(
            prohibited.iter().all(|token| !source.contains(token)),
            "unmetered schedule readiness and pop preparation remains"
        );
    }

    #[test]
    fn finding_100_schedule_publication_work_reproduction() {
        let item = candidate(1, 1, 1, 1);
        let result = schedule_candidates(
            [item.clone()],
            BTreeSet::new(),
            &mut WorkBudget::new(0, 1_000),
            &NeverCancelled,
        );
        assert_eq!(
            result.map(|value| value.ordered),
            Ok(vec![item.change_hash])
        );

        let source = include_str!("schedule.rs");
        assert!(
            source.contains("ScheduleOperation::RemainingInsert")
                && source.contains("ScheduleOperation::OrderedPush")
                && source.contains("ScheduleOperation::ResultPublication")
                && source.contains("charge(counter)?;\n    let value = target();"),
            "unmetered schedule insertion and result publication remains"
        );
    }
}
