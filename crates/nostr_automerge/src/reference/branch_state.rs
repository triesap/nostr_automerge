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

#[derive(Debug, PartialEq, Eq)]
struct DeltaNode<K, V> {
    parent: Option<Arc<DeltaNode<K, V>>>,
    local: BTreeMap<K, V>,
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

    pub(crate) fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

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
}

impl<K: Ord, V> From<BTreeMap<K, V>> for PersistentDeltaMap<K, V> {
    fn from(local: BTreeMap<K, V>) -> Self {
        Self::from_local(local)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::collections::BTreeMap;

    use super::PersistentDeltaMap;

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
}
