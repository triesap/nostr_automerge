use automerge::{
    ActorId, Automerge, Change, LoadOptions, OnPartialLoad, StringMigration, TextEncoding,
    VerificationMode,
};
use std::collections::{BTreeMap, BTreeSet};

use crate::ChangeHash;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DocumentLoadError;

#[derive(Clone)]
pub(crate) struct Document {
    inner: Automerge,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AppliedDocument {
    pub(crate) heads: BTreeSet<ChangeHash>,
    pub(crate) canonical_bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExactApplyError {
    ClosureMismatch,
    Decode,
    HashMismatch,
    Application,
    Heads,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AuthoringOperation {
    PutString {
        key: String,
        value: String,
    },
    CreateList {
        key: String,
        values: Vec<String>,
    },
    CreateText {
        key: String,
        value: String,
    },
    CreateCounter {
        key: String,
        value: i64,
        increment: i64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AdapterAuthoredChange {
    pub(crate) raw: Vec<u8>,
    pub(crate) hash: ChangeHash,
    pub(crate) operation_count: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AdapterAuthoringError {
    Empty,
    Operation,
    Missing,
    Hash,
    Limit,
}

pub(crate) fn apply_exact_closure(
    closure: &BTreeMap<ChangeHash, Vec<u8>>,
    ordered: &[ChangeHash],
    candidate_hash: ChangeHash,
    candidate_raw: &[u8],
    candidate_dependencies: &BTreeSet<ChangeHash>,
) -> Result<AppliedDocument, ExactApplyError> {
    if ordered.iter().copied().collect::<BTreeSet<_>>() != closure.keys().copied().collect() {
        return Err(ExactApplyError::ClosureMismatch);
    }
    let mut document = Automerge::new_with_encoding(TextEncoding::Utf16CodeUnit);
    for hash in ordered {
        let raw = closure.get(hash).ok_or(ExactApplyError::ClosureMismatch)?;
        let change = Change::try_from(raw.as_slice()).map_err(|_| ExactApplyError::Decode)?;
        if change.hash().as_ref() != hash.as_bytes() {
            return Err(ExactApplyError::HashMismatch);
        }
        document
            .apply_changes([change])
            .map_err(|_| ExactApplyError::Application)?;
    }
    let before = heads(&document)?;
    let candidate = Change::try_from(candidate_raw).map_err(|_| ExactApplyError::Decode)?;
    if candidate.hash().as_ref() != candidate_hash.as_bytes() {
        return Err(ExactApplyError::HashMismatch);
    }
    document
        .apply_changes([candidate])
        .map_err(|_| ExactApplyError::Application)?;
    let actual = heads(&document)?;
    let mut expected = before;
    for dependency in candidate_dependencies {
        expected.remove(dependency);
    }
    expected.insert(candidate_hash);
    if actual != expected {
        return Err(ExactApplyError::Heads);
    }
    Ok(AppliedDocument {
        heads: actual,
        canonical_bytes: document.save_nocompress(),
    })
}

pub(crate) fn materialize_history(
    raw_changes: &BTreeMap<ChangeHash, Vec<u8>>,
    ordered: &[ChangeHash],
) -> Result<AppliedDocument, ExactApplyError> {
    if ordered.iter().copied().collect::<BTreeSet<_>>() != raw_changes.keys().copied().collect() {
        return Err(ExactApplyError::ClosureMismatch);
    }
    let mut document = Automerge::new_with_encoding(TextEncoding::Utf16CodeUnit);
    for hash in ordered {
        let raw = raw_changes
            .get(hash)
            .ok_or(ExactApplyError::ClosureMismatch)?;
        let change = Change::try_from(raw.as_slice()).map_err(|_| ExactApplyError::Decode)?;
        if change.hash().as_ref() != hash.as_bytes() {
            return Err(ExactApplyError::HashMismatch);
        }
        document
            .apply_changes([change])
            .map_err(|_| ExactApplyError::Application)?;
    }
    Ok(AppliedDocument {
        heads: heads(&document)?,
        canonical_bytes: document.save_nocompress(),
    })
}

fn heads(document: &Automerge) -> Result<BTreeSet<ChangeHash>, ExactApplyError> {
    document
        .get_heads()
        .into_iter()
        .map(|hash| {
            let bytes: [u8; 32] = hash
                .as_ref()
                .try_into()
                .map_err(|_| ExactApplyError::HashMismatch)?;
            Ok(ChangeHash::from_bytes(bytes))
        })
        .collect()
}

impl Document {
    pub(crate) fn new_utf16() -> Self {
        Self {
            inner: Automerge::new_with_encoding(TextEncoding::Utf16CodeUnit),
        }
    }

    pub(crate) fn is_utf16(&self) -> bool {
        self.inner.text_encoding() == TextEncoding::Utf16CodeUnit
    }

    pub(crate) fn replace_unused_actor(&mut self, actor: &[u8]) {
        self.inner.set_actor(ActorId::from(actor));
    }

    pub(crate) fn actor_bytes(&self) -> Vec<u8> {
        self.inner.get_actor().to_bytes().to_vec()
    }

    pub(crate) fn semantic_heads(&self) -> Result<BTreeSet<ChangeHash>, ExactApplyError> {
        heads(&self.inner)
    }

    pub(crate) fn canonical_bytes(&self) -> Vec<u8> {
        self.inner.save_nocompress()
    }

    pub(crate) fn author_operations(
        &mut self,
        operations: &[AuthoringOperation],
    ) -> Result<AdapterAuthoredChange, AdapterAuthoringError> {
        use automerge::{
            ObjType, ROOT, ReadDoc, ScalarValue, transaction::CommitOptions,
            transaction::Transactable,
        };

        if operations.is_empty() {
            return Err(AdapterAuthoringError::Empty);
        }
        let limits = crate::ProtocolRevision::draft_v1().limits();
        if u64::try_from(operations.len()).map_err(|_| AdapterAuthoringError::Limit)?
            > limits.change_operations.get()
        {
            return Err(AdapterAuthoringError::Limit);
        }
        let mut staged = self.inner.clone();
        let mut transaction = staged.transaction();
        for operation in operations {
            match operation {
                AuthoringOperation::PutString { key, value } => {
                    transaction
                        .put(ROOT, key, value.as_str())
                        .map_err(|_| AdapterAuthoringError::Operation)?;
                }
                AuthoringOperation::CreateList { key, values } => {
                    let list = transaction
                        .put_object(ROOT, key, ObjType::List)
                        .map_err(|_| AdapterAuthoringError::Operation)?;
                    for (index, value) in values.iter().enumerate() {
                        transaction
                            .insert(&list, index, value.as_str())
                            .map_err(|_| AdapterAuthoringError::Operation)?;
                    }
                }
                AuthoringOperation::CreateText { key, value } => {
                    let text = transaction
                        .put_object(ROOT, key, ObjType::Text)
                        .map_err(|_| AdapterAuthoringError::Operation)?;
                    transaction
                        .splice_text(&text, 0, 0, value)
                        .map_err(|_| AdapterAuthoringError::Operation)?;
                }
                AuthoringOperation::CreateCounter {
                    key,
                    value,
                    increment,
                } => {
                    transaction
                        .put(ROOT, key, ScalarValue::counter(*value))
                        .map_err(|_| AdapterAuthoringError::Operation)?;
                    if *increment != 0 {
                        transaction
                            .increment(ROOT, key, *increment)
                            .map_err(|_| AdapterAuthoringError::Operation)?;
                    }
                }
            }
        }
        let (hash, _) = transaction.commit_with(CommitOptions::default().with_time(0));
        let hash = hash.ok_or(AdapterAuthoringError::Empty)?;
        let change = staged
            .get_change_by_hash(&hash)
            .ok_or(AdapterAuthoringError::Missing)?;
        let raw = change.raw_bytes().to_vec();
        if u64::try_from(raw.len()).map_err(|_| AdapterAuthoringError::Limit)?
            > limits.change_bytes.get()
            || u64::try_from(change.len()).map_err(|_| AdapterAuthoringError::Limit)?
                > limits.change_operations.get()
            || u64::try_from(change.deps().len()).map_err(|_| AdapterAuthoringError::Limit)?
                > limits.change_dependencies.get()
        {
            return Err(AdapterAuthoringError::Limit);
        }
        let bytes: [u8; 32] = hash.0;
        let operation_count =
            u64::try_from(change.len()).map_err(|_| AdapterAuthoringError::Limit)?;
        self.inner = staged;
        Ok(AdapterAuthoredChange {
            raw,
            hash: ChangeHash::from_bytes(bytes),
            operation_count,
        })
    }

    pub(crate) fn author_empty_change(
        &mut self,
    ) -> Result<AdapterAuthoredChange, AdapterAuthoringError> {
        use automerge::{ReadDoc, transaction::CommitOptions};

        let limits = crate::ProtocolRevision::draft_v1().limits();
        if u64::try_from(self.inner.get_heads().len()).map_err(|_| AdapterAuthoringError::Limit)?
            > limits.change_dependencies.get()
        {
            return Err(AdapterAuthoringError::Limit);
        }
        let mut staged = self.inner.clone();
        let hash = staged.empty_commit(CommitOptions::default().with_time(0));
        let change = staged
            .get_change_by_hash(&hash)
            .ok_or(AdapterAuthoringError::Missing)?;
        let raw = change.raw_bytes().to_vec();
        if u64::try_from(raw.len()).map_err(|_| AdapterAuthoringError::Limit)?
            > limits.change_bytes.get()
        {
            return Err(AdapterAuthoringError::Limit);
        }
        let bytes = hash.0;
        self.inner = staged;
        Ok(AdapterAuthoredChange {
            raw,
            hash: ChangeHash::from_bytes(bytes),
            operation_count: 0,
        })
    }

    #[cfg(test)]
    pub(crate) fn author_test_change(&mut self) -> Option<Vec<u8>> {
        use automerge::{ROOT, ReadDoc, transaction::CommitOptions, transaction::Transactable};

        let mut transaction = self.inner.transaction();
        transaction.put(ROOT, "metadata", true).ok()?;
        let (hash, _) = transaction.commit_with(CommitOptions::default().with_time(0));
        let hash = hash?;
        self.inner
            .get_change_by_hash(&hash)
            .map(|change| change.raw_bytes().to_vec())
    }

    pub(crate) fn load_utf16(bytes: &[u8]) -> Result<Self, DocumentLoadError> {
        let options = LoadOptions::new()
            .text_encoding(TextEncoding::Utf16CodeUnit)
            .migrate_strings(StringMigration::NoMigration)
            .on_partial_load(OnPartialLoad::Error)
            .verification_mode(VerificationMode::Check);
        Automerge::load_with_options(bytes, options)
            .map(|inner| Self { inner })
            .map_err(|_| DocumentLoadError)
    }
}

#[cfg(test)]
mod tests {
    use automerge::{ActorId, ROOT, ReadDoc, transaction::Transactable};

    use super::Document;

    #[test]
    fn construction_is_explicitly_utf16() {
        assert!(Document::new_utf16().is_utf16());
    }

    #[test]
    fn load_with_utf_16_no_migration_and_no_partial_state() {
        let mut source = Document::new_utf16();
        {
            let mut tx = source.inner.transaction();
            assert!(tx.put(ROOT, "value", "raw string").is_ok());
            tx.commit();
        }
        let expected_heads = source.inner.get_heads();
        let saved = source.inner.save_nocompress();

        let loaded = Document::load_utf16(&saved);
        assert!(loaded.is_ok());
        let loaded = match loaded {
            Ok(document) => document,
            Err(_) => return,
        };
        assert!(loaded.is_utf16());
        assert_eq!(loaded.inner.get_heads(), expected_heads);
        assert!(matches!(
            loaded.inner.get(ROOT, "value"),
            Ok(Some((automerge::Value::Scalar(value), _)))
                if matches!(value.as_ref(), automerge::ScalarValue::Str(text) if text == "raw string")
        ));

        let truncated = &saved[..saved.len().saturating_sub(1)];
        assert!(Document::load_utf16(truncated).is_err());
    }

    fn fixed_change() -> (ActorId, ActorId, Vec<u8>) {
        let mut document = Document::new_utf16();
        let unused_actor = document.inner.get_actor().clone();
        let derived_actor = ActorId::from([0x42; 32]);
        document.replace_unused_actor(derived_actor.to_bytes());
        {
            let mut tx = document.inner.transaction();
            assert!(tx.put(ROOT, "key", "value").is_ok());
            tx.commit();
        }
        let changes = document.inner.get_changes(&[]);
        assert_eq!(changes.len(), 1);
        assert!(changes[0].actors().all(|actor| actor == &derived_actor));
        (unused_actor, derived_actor, changes[0].raw_bytes().to_vec())
    }

    #[test]
    fn prove_derived_actor_replaces_unused_random_actor() {
        let (first_unused, derived_actor, first_bytes) = fixed_change();
        let (second_unused, second_derived, second_bytes) = fixed_change();

        assert_ne!(first_unused, derived_actor);
        assert_ne!(second_unused, second_derived);
        assert_eq!(derived_actor, second_derived);
        assert_eq!(first_bytes, second_bytes);
    }
}
