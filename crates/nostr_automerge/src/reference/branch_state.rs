use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct PersistentDeltaMap<K, V> {
    tail: Option<Arc<DeltaNode<K, V>>>,
}

impl<K, V> Clone for PersistentDeltaMap<K, V> {
    fn clone(&self) -> Self {
        Self {
            tail: self.tail.clone(),
        }
    }
}

impl<K, V> Drop for PersistentDeltaMap<K, V> {
    fn drop(&mut self) {
        let mut cursor = self.tail.take();
        while let Some(node) = cursor {
            match Arc::try_unwrap(node) {
                Ok(mut owned) => cursor = owned.parent.take(),
                Err(shared) => {
                    drop(shared);
                    break;
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct DeltaNode<K, V> {
    parent: Option<Arc<DeltaNode<K, V>>>,
    local: BTreeMap<K, V>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PersistentDeltaWork {
    PreparedItem,
    LookupNode,
    AcceptedInsert,
}

impl<K, V> Default for PersistentDeltaMap<K, V> {
    fn default() -> Self {
        Self { tail: None }
    }
}

impl<K: Ord, V> PersistentDeltaMap<K, V> {
    pub(crate) fn from_local(local: BTreeMap<K, V>) -> Self {
        if local.is_empty() {
            return Self::default();
        }
        Self {
            tail: Some(Arc::new(DeltaNode {
                parent: None,
                local,
            })),
        }
    }

    #[cfg(test)]
    pub(crate) fn get(&self, key: &K) -> Option<&V> {
        let mut cursor = self.tail.as_deref();
        while let Some(node) = cursor {
            if let Some(value) = node.local.get(key) {
                return Some(value);
            }
            cursor = node.parent.as_deref();
        }
        None
    }

    pub(crate) fn get_metered<E>(
        &self,
        key: &K,
        mut visit: impl FnMut() -> Result<(), E>,
    ) -> Result<Option<&V>, E> {
        let mut cursor = self.tail.as_deref();
        while let Some(node) = cursor {
            visit()?;
            if let Some(value) = node.local.get(key) {
                return Ok(Some(value));
            }
            cursor = node.parent.as_deref();
        }
        Ok(None)
    }

    #[cfg(test)]
    pub(crate) fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    pub(crate) fn contains_key_metered<E>(
        &self,
        key: &K,
        visit: impl FnMut() -> Result<(), E>,
    ) -> Result<bool, E> {
        self.get_metered(key, visit).map(|value| value.is_some())
    }

    #[cfg(test)]
    pub(crate) fn extend_local(&self, mut local: BTreeMap<K, V>) -> Self
    where
        V: PartialEq,
    {
        local.retain(|key, value| self.get(key) != Some(value));
        if local.is_empty() {
            return self.clone();
        }
        Self {
            tail: Some(Arc::new(DeltaNode {
                parent: self.tail.clone(),
                local,
            })),
        }
    }

    pub(crate) fn extend_prepared_metered<E>(
        &self,
        mut prepared: BTreeMap<K, V>,
        mut work: impl FnMut(PersistentDeltaWork) -> Result<(), E>,
    ) -> Result<Self, E>
    where
        V: PartialEq,
    {
        let mut accepted = BTreeMap::new();
        while !prepared.is_empty() {
            work(PersistentDeltaWork::PreparedItem)?;
            let Some((key, value)) = prepared.pop_first() else {
                continue;
            };
            let inherited = self.get_metered(&key, || work(PersistentDeltaWork::LookupNode))?;
            if inherited == Some(&value) {
                continue;
            }
            work(PersistentDeltaWork::AcceptedInsert)?;
            accepted.insert(key, value);
        }
        if accepted.is_empty() {
            return Ok(self.clone());
        }
        Ok(Self {
            tail: Some(Arc::new(DeltaNode {
                parent: self.tail.clone(),
                local: accepted,
            })),
        })
    }

    pub(crate) fn materialize_metered<E>(
        &self,
        mut visit: impl FnMut() -> Result<(), E>,
    ) -> Result<BTreeMap<K, V>, E>
    where
        K: Copy,
        V: Copy,
    {
        let mut nodes = Vec::new();
        let mut cursor = self.tail.as_deref();
        while let Some(node) = cursor {
            visit()?;
            nodes.push(node);
            cursor = node.parent.as_deref();
        }
        let mut result = BTreeMap::new();
        while let Some(node) = nodes.pop() {
            for (key, value) in &node.local {
                visit()?;
                result.insert(*key, *value);
            }
        }
        Ok(result)
    }

    #[cfg(test)]
    fn shares_parent_with(&self, parent: &Self) -> bool {
        match (&self.tail, &parent.tail) {
            (Some(child), Some(parent)) => child
                .parent
                .as_ref()
                .is_some_and(|retained| Arc::ptr_eq(retained, parent)),
            (Some(child), None) => child.parent.is_none(),
            _ => false,
        }
    }

    #[cfg(test)]
    fn local_len(&self) -> usize {
        self.tail.as_ref().map_or(0, |tail| tail.local.len())
    }

    #[cfg(test)]
    fn shares_tail_with(&self, other: &Self) -> bool {
        match (&self.tail, &other.tail) {
            (Some(left), Some(right)) => Arc::ptr_eq(left, right),
            (None, None) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
impl<K: Ord, V> From<BTreeMap<K, V>> for PersistentDeltaMap<K, V> {
    fn from(local: BTreeMap<K, V>) -> Self {
        Self::from_local(local)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::cmp::Ordering;
    use std::collections::BTreeMap;
    use std::process::Command;
    use std::rc::Rc;

    use super::{PersistentDeltaMap, PersistentDeltaWork};

    #[derive(Debug)]
    struct DropProbe {
        identity: u8,
        drops: Rc<Cell<usize>>,
    }

    impl Drop for DropProbe {
        fn drop(&mut self) {
            self.drops.set(self.drops.get() + 1);
        }
    }

    impl PartialEq for DropProbe {
        fn eq(&self, other: &Self) -> bool {
            self.identity == other.identity
        }
    }

    impl Eq for DropProbe {}

    #[test]
    fn delta_chain_shares_parent_and_materializes_in_override_order() {
        let parent = PersistentDeltaMap::from_local(BTreeMap::from([(1_u8, 10_u8), (2, 20)]));
        let child = parent.extend_local(BTreeMap::from([(2, 21), (3, 30)]));
        assert!(child.shares_parent_with(&parent));
        assert_eq!(child.get(&1), Some(&10));
        assert_eq!(child.get(&2), Some(&21));
        assert_eq!(child.get(&3), Some(&30));
        assert_eq!(child.local_len(), 2);

        let visits = Cell::new(0_u64);
        let materialized = child.materialize_metered(|| {
            visits.set(visits.get() + 1);
            Ok::<(), ()>(())
        });
        assert_eq!(
            materialized,
            Ok(BTreeMap::from([(1, 10), (2, 21), (3, 30)]))
        );
        assert_eq!(visits.get(), 6);
        for boundary in 0..visits.get() {
            let observed = Cell::new(0_u64);
            let stopped = child.materialize_metered(|| {
                if observed.get() == boundary {
                    return Err(());
                }
                observed.set(observed.get() + 1);
                Ok(())
            });
            assert_eq!(stopped, Err(()));
            assert_eq!(observed.get(), boundary);
        }
    }

    #[test]
    fn metered_lookup_counts_only_nodes_actually_visited() {
        let mut state = PersistentDeltaMap::from_local(BTreeMap::from([(0_u8, 10_u8)]));
        for value in 1_u8..=3 {
            state = state.extend_local(BTreeMap::from([(value, value + 10)]));
        }

        for (key, expected, expected_visits) in [
            (3, Some(&13), 1_u64),
            (1, Some(&11), 3),
            (0, Some(&10), 4),
            (4, None, 4),
        ] {
            let visits = Cell::new(0_u64);
            let result = state.get_metered(&key, || {
                visits.set(visits.get() + 1);
                Ok::<(), ()>(())
            });
            assert_eq!(result, Ok(expected));
            assert_eq!(visits.get(), expected_visits);
        }

        let injected = Rc::new("lookup stop");
        let result = state.get_metered(&0, || Err(Rc::clone(&injected)));
        assert!(result.is_err(), "metered lookup unexpectedly completed");
        let Err(returned) = result else { return };
        assert!(Rc::ptr_eq(&returned, &injected));
    }

    #[test]
    fn metered_membership_reuses_lookup_without_hidden_scans() {
        let mut state = PersistentDeltaMap::from_local(BTreeMap::from([(0_u8, 10_u8)]));
        for value in 1_u8..=3 {
            state = state.extend_local(BTreeMap::from([(value, value + 10)]));
        }

        for (key, expected, expected_visits) in [(3, true, 1_u64), (0, true, 4), (4, false, 4)] {
            let visits = Cell::new(0_u64);
            let complete = state.contains_key_metered(&key, || {
                visits.set(visits.get() + 1);
                Ok::<(), Rc<&'static str>>(())
            });
            assert_eq!(complete, Ok(expected));
            assert_eq!(visits.get(), expected_visits);

            for boundary in 0..expected_visits {
                let observed = Cell::new(0_u64);
                let injected = Rc::new("membership stop");
                let stopped = state.contains_key_metered(&key, || {
                    if observed.get() == boundary {
                        return Err(Rc::clone(&injected));
                    }
                    observed.set(observed.get() + 1);
                    Ok(())
                });
                assert!(stopped.is_err());
                let Err(returned) = stopped else { continue };
                assert!(Rc::ptr_eq(&returned, &injected));
                assert_eq!(observed.get(), boundary);
            }
        }
    }

    #[test]
    fn metered_extension_publishes_only_after_all_owned_work() {
        let root = PersistentDeltaMap::from_local(BTreeMap::from([(0_u8, 10_u8)]));
        let parent = root.extend_local(BTreeMap::from([(1, 11)]));
        let prepared = BTreeMap::from([(0, 10), (1, 12), (2, 20)]);
        let expected_work = [
            PersistentDeltaWork::PreparedItem,
            PersistentDeltaWork::LookupNode,
            PersistentDeltaWork::LookupNode,
            PersistentDeltaWork::PreparedItem,
            PersistentDeltaWork::LookupNode,
            PersistentDeltaWork::AcceptedInsert,
            PersistentDeltaWork::PreparedItem,
            PersistentDeltaWork::LookupNode,
            PersistentDeltaWork::LookupNode,
            PersistentDeltaWork::AcceptedInsert,
        ];

        let observed = std::cell::RefCell::new(Vec::new());
        let extended = parent.extend_prepared_metered(prepared.clone(), |stage| {
            observed.borrow_mut().push(stage);
            Ok::<(), Rc<&'static str>>(())
        });
        assert!(extended.is_ok());
        assert_eq!(&*observed.borrow(), &expected_work);
        let Ok(extended) = extended else { return };
        assert!(extended.shares_parent_with(&parent));
        assert_eq!(extended.local_len(), 2);
        assert_eq!(extended.get(&0), Some(&10));
        assert_eq!(extended.get(&1), Some(&12));
        assert_eq!(extended.get(&2), Some(&20));

        for boundary in 0..expected_work.len() {
            let observed = std::cell::RefCell::new(Vec::new());
            let injected = Rc::new("extension stop");
            let stopped = parent.extend_prepared_metered(prepared.clone(), |stage| {
                if observed.borrow().len() == boundary {
                    return Err(Rc::clone(&injected));
                }
                observed.borrow_mut().push(stage);
                Ok(())
            });
            assert!(stopped.is_err());
            let Err(returned) = stopped else { continue };
            assert!(Rc::ptr_eq(&returned, &injected));
            assert_eq!(&*observed.borrow(), &expected_work[..boundary]);
            assert_eq!(parent.get(&0), Some(&10));
            assert_eq!(parent.get(&1), Some(&11));
            assert_eq!(parent.get(&2), None);
        }
    }

    #[test]
    fn deep_persistent_boundaries_are_exact_and_cancellable() {
        const DEPTH: u8 = 64;
        let mut state = PersistentDeltaMap::from_local(BTreeMap::from([(0_u8, 0_u8)]));
        for key in 1..DEPTH {
            state = state.extend_local(BTreeMap::from([(key, key)]));
        }

        for (key, expected, expected_work) in [
            (DEPTH - 1, Some(&(DEPTH - 1)), 1_usize),
            (0, Some(&0), usize::from(DEPTH)),
            (DEPTH, None, usize::from(DEPTH)),
        ] {
            for capacity in 0..=expected_work + 1 {
                let observed = Cell::new(0_usize);
                let injected = Rc::new("deep lookup stop");
                let result = state.get_metered(&key, || {
                    if observed.get() == capacity {
                        return Err(Rc::clone(&injected));
                    }
                    observed.set(observed.get() + 1);
                    Ok(())
                });
                if capacity < expected_work {
                    assert!(result.is_err());
                    let Err(returned) = result else { continue };
                    assert!(Rc::ptr_eq(&returned, &injected));
                    assert_eq!(observed.get(), capacity);
                } else {
                    assert_eq!(result, Ok(expected));
                    assert_eq!(observed.get(), expected_work);
                }
            }
        }

        let lookup_stages = vec![PersistentDeltaWork::LookupNode; usize::from(DEPTH)];
        for (key, value, accepted) in [(0, 99, true), (0, 0, false), (DEPTH, DEPTH, true)] {
            let mut expected_stages = Vec::with_capacity(usize::from(DEPTH) + 2);
            expected_stages.push(PersistentDeltaWork::PreparedItem);
            expected_stages.extend_from_slice(&lookup_stages);
            if accepted {
                expected_stages.push(PersistentDeltaWork::AcceptedInsert);
            }
            for capacity in 0..=expected_stages.len() + 1 {
                let observed = std::cell::RefCell::new(Vec::new());
                let injected = Rc::new("deep extension stop");
                let result =
                    state.extend_prepared_metered(BTreeMap::from([(key, value)]), |stage| {
                        if observed.borrow().len() == capacity {
                            return Err(Rc::clone(&injected));
                        }
                        observed.borrow_mut().push(stage);
                        Ok(())
                    });
                if capacity < expected_stages.len() {
                    assert!(result.is_err());
                    let Err(returned) = result else { continue };
                    assert!(Rc::ptr_eq(&returned, &injected));
                    assert_eq!(&*observed.borrow(), &expected_stages[..capacity]);
                } else {
                    assert!(result.is_ok());
                    let Ok(extended) = result else { continue };
                    assert_eq!(&*observed.borrow(), &expected_stages);
                    assert_eq!(extended.get(&key), Some(&value));
                    assert_eq!(extended.shares_tail_with(&state), !accepted);
                    assert_eq!(extended.local_len(), 1);
                }
            }
        }
    }

    #[derive(Clone, Debug)]
    struct CountedKey {
        value: u16,
        comparisons: Rc<Cell<usize>>,
    }

    impl PartialEq for CountedKey {
        fn eq(&self, other: &Self) -> bool {
            self.cmp(other) == Ordering::Equal
        }
    }

    impl Eq for CountedKey {}

    impl PartialOrd for CountedKey {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Ord for CountedKey {
        fn cmp(&self, other: &Self) -> Ordering {
            self.comparisons.set(self.comparisons.get() + 1);
            self.value.cmp(&other.value)
        }
    }

    #[test]
    fn finding_096_deep_persistent_lookup_is_internally_metered() {
        let comparisons = Rc::new(Cell::new(0));
        let key = |value| CountedKey {
            value,
            comparisons: Rc::clone(&comparisons),
        };
        let mut state = PersistentDeltaMap::from_local(BTreeMap::from([(key(0), 1_u8)]));
        for value in 1_u16..=64 {
            state = state.extend_local(BTreeMap::from([(key(value), 1)]));
        }
        comparisons.set(0);
        let visits = Cell::new(0_usize);
        let result = state.get_metered(&key(0), || {
            visits.set(visits.get() + 1);
            Ok::<(), ()>(())
        });
        assert_eq!(result, Ok(Some(&1)));
        assert_eq!(
            (visits.get(), comparisons.get()),
            (65, 65),
            "every retained-node lookup must have one immediately preceding work observation"
        );
    }

    #[test]
    fn constrained_stack_wide_delta_fork_preserves_shared_parent_teardown() {
        let worker = std::thread::Builder::new().stack_size(64 * 1024).spawn(|| {
            let root = PersistentDeltaMap::from_local(BTreeMap::from([(0_u32, 0_u32)]));
            let mut branches = Vec::new();
            for value in 1_u32..=10_000 {
                let branch = root.extend_local(BTreeMap::from([(value, value)]));
                assert!(branch.shares_parent_with(&root));
                branches.push(branch);
            }
            drop(root);
            drop(branches);
        });
        assert!(
            worker.is_ok(),
            "construct constrained-stack worker: {worker:?}"
        );
        let Ok(worker) = worker else { return };
        assert!(
            worker.join().is_ok(),
            "wide shared delta fork must not recurse through the shared parent"
        );
    }

    #[test]
    fn clone_drop_permutations_release_each_delta_value_once() {
        let root_drops = Rc::new(Cell::new(0));
        let child_drops = Rc::new(Cell::new(0));
        let root = PersistentDeltaMap::from_local(BTreeMap::from([(
            0_u8,
            DropProbe {
                identity: 0,
                drops: Rc::clone(&root_drops),
            },
        )]));
        let child = root.extend_local(BTreeMap::from([(
            1,
            DropProbe {
                identity: 1,
                drops: Rc::clone(&child_drops),
            },
        )]));
        let first = child.clone();
        let second = child.clone();

        drop(root);
        drop(second);
        assert_eq!((root_drops.get(), child_drops.get()), (0, 0));
        drop(child);
        assert_eq!((root_drops.get(), child_drops.get()), (0, 0));
        drop(first);
        assert_eq!((root_drops.get(), child_drops.get()), (1, 1));
    }

    #[test]
    fn stopped_and_panicking_delta_construction_releases_unpublished_values_once() {
        let expected = [
            PersistentDeltaWork::PreparedItem,
            PersistentDeltaWork::LookupNode,
            PersistentDeltaWork::AcceptedInsert,
        ];
        for stop_at in 0..expected.len() {
            let root_drops = Rc::new(Cell::new(0));
            let prepared_drops = Rc::new(Cell::new(0));
            let root = PersistentDeltaMap::from_local(BTreeMap::from([(
                0_u8,
                DropProbe {
                    identity: 0,
                    drops: Rc::clone(&root_drops),
                },
            )]));
            let prepared = BTreeMap::from([(
                1,
                DropProbe {
                    identity: 1,
                    drops: Rc::clone(&prepared_drops),
                },
            )]);
            let observed = std::cell::RefCell::new(Vec::new());
            let injected = Rc::new("typed stop");
            let stopped = root.extend_prepared_metered(prepared, |stage| {
                observed.borrow_mut().push(stage);
                if observed.borrow().len() == stop_at + 1 {
                    return Err(Rc::clone(&injected));
                }
                Ok(())
            });
            assert!(stopped.is_err());
            let Err(returned) = stopped else { continue };
            assert!(Rc::ptr_eq(&returned, &injected));
            assert_eq!(&*observed.borrow(), &expected[..=stop_at]);
            assert_eq!(prepared_drops.get(), 1);
            assert_eq!(root_drops.get(), 0);
            assert_eq!(root.get(&0).map(|value| value.identity), Some(0));
            drop(root);
            assert_eq!((root_drops.get(), prepared_drops.get()), (1, 1));
        }

        let root_drops = Rc::new(Cell::new(0));
        let prepared_drops = Rc::new(Cell::new(0));
        let root = PersistentDeltaMap::from_local(BTreeMap::from([(
            0_u8,
            DropProbe {
                identity: 0,
                drops: Rc::clone(&root_drops),
            },
        )]));
        let prepared = BTreeMap::from([(
            1,
            DropProbe {
                identity: 1,
                drops: Rc::clone(&prepared_drops),
            },
        )]);
        let observed = std::cell::RefCell::new(Vec::new());
        let injected = std::sync::Arc::new("unexpected panic");
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = root.extend_prepared_metered(prepared, |stage| {
                observed.borrow_mut().push(stage);
                if observed.borrow().len() == 2 {
                    std::panic::resume_unwind(Box::new(std::sync::Arc::clone(&injected)));
                }
                Ok::<(), Rc<&'static str>>(())
            });
        }));
        assert!(panic.is_err());
        let Err(payload) = panic else { return };
        let returned = payload.downcast_ref::<std::sync::Arc<&'static str>>();
        assert!(returned.is_some_and(|value| std::sync::Arc::ptr_eq(value, &injected)));
        assert_eq!(&*observed.borrow(), &expected[..2]);
        assert_eq!(prepared_drops.get(), 1);
        assert_eq!(root_drops.get(), 0);
        drop(root);
        assert_eq!((root_drops.get(), prepared_drops.get()), (1, 1));
    }

    #[test]
    fn constrained_stack_delta_drop_stops_at_a_retained_shared_prefix() {
        let worker = std::thread::Builder::new().stack_size(64 * 1024).spawn(|| {
            let mut state = PersistentDeltaMap::from_local(BTreeMap::from([(0_u8, 0_u32)]));
            for value in 1_u32..=5_000 {
                state = state.extend_local(BTreeMap::from([(0, value)]));
            }
            let retained = state.clone();
            for value in 5_001_u32..=10_000 {
                state = state.extend_local(BTreeMap::from([(0, value)]));
            }
            drop(state);
            assert_eq!(retained.get(&0), Some(&5_000));
            drop(retained);
        });
        assert!(
            worker.is_ok(),
            "construct constrained-stack worker: {worker:?}"
        );
        let Ok(worker) = worker else { return };
        assert!(
            worker.join().is_ok(),
            "shared delta prefix teardown must stay bounded"
        );
    }

    #[test]
    fn deep_unique_delta_teardown_is_bounded_stack() {
        const TEST_NAME: &str =
            "reference::branch_state::tests::deep_unique_delta_teardown_is_bounded_stack";
        const CHILD_ENV: &str = "NOSTR_AUTOMERGE_FINDING_099_CHILD";
        if std::env::var_os(CHILD_ENV).is_some() {
            let child = std::thread::Builder::new().stack_size(64 * 1024).spawn(|| {
                let mut state = PersistentDeltaMap::from_local(BTreeMap::from([(0_u8, 0_usize)]));
                for value in 1_usize..=100_000 {
                    state = state.extend_local(BTreeMap::from([(0, value)]));
                }
                drop(state);
            });
            assert!(
                child.is_ok(),
                "construct constrained-stack thread: {child:?}"
            );
            let Ok(handle) = child else { return };
            assert!(handle.join().is_ok());
            return;
        }

        let executable = std::env::current_exe();
        assert!(
            executable.is_ok(),
            "resolve current test executable: {executable:?}"
        );
        let Ok(executable) = executable else { return };
        let output = Command::new(executable)
            .args(["--exact", TEST_NAME])
            .env(CHILD_ENV, "1")
            .output();
        assert!(
            output.is_ok(),
            "execute isolated teardown reproduction: {output:?}"
        );
        let Ok(output) = output else { return };
        assert!(
            output.status.success(),
            "FINDING_099 reproduced: deep uniquely owned persistent teardown exceeded the constrained stack"
        );
    }
}
