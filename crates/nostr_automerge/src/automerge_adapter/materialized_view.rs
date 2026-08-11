use core::fmt;

use crate::{CancellationCheck, WorkBudget, WorkCounter};
use automerge::{
    Automerge, LoadOptions, ObjId, ObjType, OnPartialLoad, ROOT, ReadDoc, ScalarValue,
    StringMigration, TextEncoding, Value, VerificationMode,
};

/// One deterministic element in a materialized document path.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MaterializedPathElement {
    /// A map or table property.
    Key(String),
    /// A list position under UTF-16 document semantics.
    Index(u64),
    /// The exact conflicting composite branch selected for following descendants.
    Branch {
        /// Stable identity of the object containing the conflicting property.
        parent_object_id: String,
        /// Stable identity of the operation that selected this child value.
        operation_id: String,
        /// Stable identity of the selected child object.
        child_object_id: String,
    },
}

impl MaterializedPathElement {
    /// Creates one exact conflicting-object branch element.
    #[must_use]
    pub fn branch(
        parent_object_id: impl Into<String>,
        operation_id: impl Into<String>,
        child_object_id: impl Into<String>,
    ) -> Self {
        Self::Branch {
            parent_object_id: parent_object_id.into(),
            operation_id: operation_id.into(),
            child_object_id: child_object_id.into(),
        }
    }

    /// Returns the three stable identities when this is a branch element.
    #[must_use]
    pub fn branch_identity(&self) -> Option<(&str, &str, &str)> {
        let Self::Branch {
            parent_object_id,
            operation_id,
            child_object_id,
        } = self
        else {
            return None;
        };
        Some((parent_object_id, operation_id, child_object_id))
    }
}

/// Exact Automerge mark boundary expansion semantics.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MaterializedMarkExpansion {
    /// Neither boundary expands.
    None,
    /// Only insertion at the start boundary inherits the mark.
    Before,
    /// Only insertion at the end boundary inherits the mark.
    After,
    /// Both boundaries expand.
    Both,
}

/// Exact Automerge composite object type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MaterializedObjectType {
    /// Map object.
    Map,
    /// List object.
    List,
    /// Table object.
    Table,
    /// UTF-16 text object.
    Text,
}

/// Exact non-object Automerge value without JSON coercion.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MaterializedScalar {
    /// Null.
    Null,
    /// Boolean.
    Bool(bool),
    /// Signed 64-bit integer.
    I64(i64),
    /// Unsigned 64-bit integer.
    U64(u64),
    /// IEEE-754 bits, preserving NaNs and signed zero.
    F64Bits(u64),
    /// UTF-8 string scalar.
    String(String),
    /// Arbitrary bytes.
    Bytes(Vec<u8>),
    /// Signed millisecond timestamp.
    Timestamp(i64),
    /// Signed counter value.
    Counter(i64),
}

/// One exact projected Automerge value.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MaterializedValue {
    /// Scalar value.
    Scalar(MaterializedScalar),
    /// Composite object identity and type.
    Object {
        /// Object type.
        object_type: MaterializedObjectType,
        /// Stable Automerge external object identifier.
        object_id: String,
    },
    /// Complete UTF-16 text value and stable object identity.
    Text {
        /// Stable Automerge external object identifier.
        object_id: String,
        /// Complete text content.
        value: String,
    },
}

/// One conflicting value with the operation identity that created it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MaterializedConflict {
    operation_id: String,
    value: MaterializedValue,
}

impl MaterializedConflict {
    /// Returns the stable Automerge operation identity.
    #[must_use]
    pub fn operation_id(&self) -> &str {
        &self.operation_id
    }

    /// Returns the exact projected value.
    #[must_use]
    pub const fn value(&self) -> &MaterializedValue {
        &self.value
    }
}

/// All deterministic values present at one materialized path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaterializedEntry {
    path: Vec<MaterializedPathElement>,
    conflicts: Vec<MaterializedConflict>,
}

impl MaterializedEntry {
    /// Returns the canonical path.
    #[must_use]
    pub fn path(&self) -> &[MaterializedPathElement] {
        &self.path
    }

    /// Returns every value ordered by stable operation identity.
    #[must_use]
    pub fn conflicts(&self) -> &[MaterializedConflict] {
        &self.conflicts
    }
}

/// One projected mark range on a UTF-16 text path.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MaterializedMark {
    path: Vec<MaterializedPathElement>,
    name: String,
    value: MaterializedScalar,
    start: u64,
    end: u64,
    expansion: MaterializedMarkExpansion,
}

impl MaterializedMark {
    /// Returns the marked text path.
    #[must_use]
    pub fn path(&self) -> &[MaterializedPathElement] {
        &self.path
    }

    /// Returns the mark name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the exact mark value.
    #[must_use]
    pub const fn value(&self) -> &MaterializedScalar {
        &self.value
    }

    /// Returns the inclusive UTF-16 start position.
    #[must_use]
    pub const fn start(&self) -> u64 {
        self.start
    }

    /// Returns the exclusive UTF-16 end position.
    #[must_use]
    pub const fn end(&self) -> u64 {
        self.end
    }

    /// Returns the exact boundary expansion mode.
    #[must_use]
    pub const fn expansion(&self) -> MaterializedMarkExpansion {
        self.expansion
    }
}

/// Immutable exact projection of a materialized Automerge document.
#[derive(Clone, PartialEq, Eq)]
pub struct MaterializedDocumentView {
    canonical_bytes: Vec<u8>,
    entries: Vec<MaterializedEntry>,
    marks: Vec<MaterializedMark>,
}

impl MaterializedDocumentView {
    pub(crate) fn empty() -> Result<Self, ProjectionError> {
        Self::from_canonical_bytes(Automerge::new().save_nocompress())
    }

    #[cfg(test)]
    pub(crate) fn empty_for_test() -> Result<Self, ProjectionError> {
        Self::empty()
    }

    pub(crate) fn from_canonical_bytes(canonical_bytes: Vec<u8>) -> Result<Self, ProjectionError> {
        Self::project_canonical_bytes(canonical_bytes, None)
    }

    pub(crate) fn from_canonical_bytes_metered(
        canonical_bytes: Vec<u8>,
        budget: &mut WorkBudget,
        cancellation: &impl CancellationCheck,
    ) -> Result<Self, ProjectionError> {
        if cancellation.is_cancelled() {
            return Err(ProjectionError::Cancelled);
        }
        let bytes = u64::try_from(canonical_bytes.len()).map_err(|_| ProjectionError::Budget)?;
        budget
            .charge(WorkCounter::DecodeByte, bytes)
            .map_err(|_| ProjectionError::Budget)?;
        if cancellation.is_cancelled() {
            return Err(ProjectionError::Cancelled);
        }
        budget
            .charge(WorkCounter::ApplyChange, 1)
            .map_err(|_| ProjectionError::Budget)?;
        Self::project_canonical_bytes(
            canonical_bytes,
            Some(ProjectionMeter {
                budget,
                cancellation,
            }),
        )
    }

    fn project_canonical_bytes(
        canonical_bytes: Vec<u8>,
        mut meter: Option<ProjectionMeter<'_>>,
    ) -> Result<Self, ProjectionError> {
        let options = LoadOptions::new()
            .text_encoding(TextEncoding::Utf16CodeUnit)
            .migrate_strings(StringMigration::NoMigration)
            .on_partial_load(OnPartialLoad::Error)
            .verification_mode(VerificationMode::Check);
        let document = Automerge::load_with_options(&canonical_bytes, options)
            .map_err(|_| ProjectionError::Invalid)?;
        let mut entries = Vec::new();
        let mut marks = Vec::new();
        project_document_iterative(&document, &mut entries, &mut marks, &mut meter)?;
        charge_projection(
            &mut meter,
            WorkCounter::Assertion,
            projection_sort_work(entries.len())?,
        )?;
        entries.sort_by(|left, right| left.path.cmp(&right.path));
        charge_projection(
            &mut meter,
            WorkCounter::Assertion,
            projection_sort_work(marks.len())?,
        )?;
        marks.sort();
        Ok(Self {
            canonical_bytes,
            entries,
            marks,
        })
    }

    /// Returns the size of the materialized canonical Automerge state.
    #[must_use]
    pub fn byte_len(&self) -> usize {
        self.canonical_bytes.len()
    }

    /// Returns true when the materialized state has an empty byte encoding.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.canonical_bytes.is_empty()
    }

    /// Returns every projected path in canonical path order.
    #[must_use]
    pub fn entries(&self) -> &[MaterializedEntry] {
        &self.entries
    }

    /// Returns every projected UTF-16 mark in canonical order.
    #[must_use]
    pub fn marks(&self) -> &[MaterializedMark] {
        &self.marks
    }

    /// Returns the UTF-16 code-unit length for one unconflicted text path.
    #[must_use]
    pub fn text_utf16_len(&self, path: &[MaterializedPathElement]) -> Option<u64> {
        let entry = self.entries.iter().find(|entry| entry.path == path)?;
        let [conflict] = entry.conflicts.as_slice() else {
            return None;
        };
        let MaterializedValue::Text { value, .. } = &conflict.value else {
            return None;
        };
        u64::try_from(value.encode_utf16().count()).ok()
    }
}

impl fmt::Debug for MaterializedDocumentView {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MaterializedDocumentView")
            .field("byte_len", &self.byte_len())
            .field("entry_count", &self.entries.len())
            .field("mark_count", &self.marks.len())
            .finish()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProjectionError {
    Invalid,
    Budget,
    Cancelled,
}

struct ProjectionObject {
    object: ObjId,
    object_type: MaterializedObjectType,
    path: Vec<MaterializedPathElement>,
}

struct ProjectionMeter<'a> {
    budget: &'a mut WorkBudget,
    cancellation: &'a dyn CancellationCheck,
}

impl ProjectionMeter<'_> {
    fn charge(&mut self, counter: WorkCounter, amount: u64) -> Result<(), ProjectionError> {
        if self.cancellation.is_cancelled() {
            return Err(ProjectionError::Cancelled);
        }
        self.budget
            .charge(counter, amount)
            .map_err(|_| ProjectionError::Budget)
    }
}

fn charge_projection(
    meter: &mut Option<ProjectionMeter<'_>>,
    counter: WorkCounter,
    amount: u64,
) -> Result<(), ProjectionError> {
    if let Some(meter) = meter {
        meter.charge(counter, amount)?;
    }
    Ok(())
}

fn projection_sort_work(len: usize) -> Result<u64, ProjectionError> {
    if len < 2 {
        return Ok(0);
    }
    let len = u64::try_from(len).map_err(|_| ProjectionError::Budget)?;
    Ok(len.saturating_mul(u64::from(len.ilog2()) + 1))
}

fn project_document_iterative(
    document: &Automerge,
    entries: &mut Vec<MaterializedEntry>,
    marks: &mut Vec<MaterializedMark>,
    meter: &mut Option<ProjectionMeter<'_>>,
) -> Result<(), ProjectionError> {
    let mut pending = vec![ProjectionObject {
        object: ROOT,
        object_type: MaterializedObjectType::Map,
        path: Vec::new(),
    }];
    while let Some(current) = pending.pop() {
        charge_projection(meter, WorkCounter::Assertion, 1)?;
        match current.object_type {
            MaterializedObjectType::Map | MaterializedObjectType::Table => {
                let mut keys = Vec::new();
                for key in document.keys(&current.object) {
                    charge_projection(meter, WorkCounter::Assertion, 1)?;
                    keys.push(key);
                }
                keys.sort();
                keys.dedup();
                for key in keys {
                    charge_projection(
                        meter,
                        WorkCounter::Assertion,
                        u64::try_from(current.path.len())
                            .ok()
                            .and_then(|length| length.checked_add(1))
                            .ok_or(ProjectionError::Budget)?,
                    )?;
                    let mut next = current.path.clone();
                    next.push(MaterializedPathElement::Key(key.clone()));
                    project_property(
                        document,
                        &current.object,
                        key,
                        next,
                        entries,
                        &mut pending,
                        meter,
                    )?;
                }
            }
            MaterializedObjectType::List => {
                for index in 0..document.length(&current.object) {
                    charge_projection(
                        meter,
                        WorkCounter::Assertion,
                        u64::try_from(current.path.len())
                            .ok()
                            .and_then(|length| length.checked_add(1))
                            .ok_or(ProjectionError::Budget)?,
                    )?;
                    let mut next = current.path.clone();
                    next.push(MaterializedPathElement::Index(
                        u64::try_from(index).map_err(|_| ProjectionError::Invalid)?,
                    ));
                    project_property(
                        document,
                        &current.object,
                        index,
                        next,
                        entries,
                        &mut pending,
                        meter,
                    )?;
                }
            }
            MaterializedObjectType::Text => {
                for mark in document
                    .marks(&current.object)
                    .map_err(|_| ProjectionError::Invalid)?
                {
                    charge_projection(meter, WorkCounter::Assertion, 1)?;
                    marks.push(MaterializedMark {
                        path: current.path.clone(),
                        name: mark.name.to_string(),
                        value: scalar(&mark.value)?,
                        start: u64::try_from(mark.start).map_err(|_| ProjectionError::Invalid)?,
                        end: u64::try_from(mark.end).map_err(|_| ProjectionError::Invalid)?,
                        expansion: MaterializedMarkExpansion::None,
                    });
                }
            }
        }
    }
    Ok(())
}

fn project_property(
    document: &Automerge,
    object: &ObjId,
    property: impl Into<automerge::Prop> + Clone,
    path: Vec<MaterializedPathElement>,
    entries: &mut Vec<MaterializedEntry>,
    pending: &mut Vec<ProjectionObject>,
    meter: &mut Option<ProjectionMeter<'_>>,
) -> Result<(), ProjectionError> {
    let mut conflicts = Vec::new();
    let mut objects = Vec::new();
    let values = document
        .get_all(object, property)
        .map_err(|_| ProjectionError::Invalid)?;
    charge_projection(
        meter,
        WorkCounter::Assertion,
        u64::try_from(values.len()).map_err(|_| ProjectionError::Budget)?,
    )?;
    let has_conflicts = values.len() > 1;
    for (value, id) in values {
        if let Value::Object(kind) = &value {
            objects.push((id.to_string(), object_type(*kind), id.clone()));
        }
        conflicts.push(MaterializedConflict {
            operation_id: id.to_string(),
            value: value_at(document, value, &id, meter)?,
        });
    }
    charge_projection(
        meter,
        WorkCounter::Assertion,
        projection_sort_work(conflicts.len())?,
    )?;
    conflicts.sort_by(|left, right| left.operation_id.cmp(&right.operation_id));
    entries.push(MaterializedEntry {
        path: path.clone(),
        conflicts,
    });
    objects.sort_by(|left, right| left.0.cmp(&right.0));
    for (operation_id, kind, id) in objects.into_iter().rev() {
        let mut child_path = path.clone();
        if has_conflicts {
            child_path.push(MaterializedPathElement::branch(
                object.to_string(),
                operation_id,
                id.to_string(),
            ));
        }
        pending.push(ProjectionObject {
            object: id,
            object_type: kind,
            path: child_path,
        });
    }
    Ok(())
}

fn value_at(
    document: &Automerge,
    value: Value<'_>,
    id: &ObjId,
    meter: &mut Option<ProjectionMeter<'_>>,
) -> Result<MaterializedValue, ProjectionError> {
    charge_projection(meter, WorkCounter::Assertion, 1)?;
    match value {
        Value::Scalar(value) => Ok(MaterializedValue::Scalar(scalar(value.as_ref())?)),
        Value::Object(ObjType::Text) => {
            let text_work = u64::try_from(document.length(id))
                .ok()
                .and_then(|length| length.checked_add(1))
                .ok_or(ProjectionError::Budget)?;
            charge_projection(meter, WorkCounter::Assertion, text_work)?;
            Ok(MaterializedValue::Text {
                object_id: id.to_string(),
                value: document.text(id).map_err(|_| ProjectionError::Invalid)?,
            })
        }
        Value::Object(kind) => Ok(MaterializedValue::Object {
            object_type: object_type(kind),
            object_id: id.to_string(),
        }),
    }
}

const fn object_type(value: ObjType) -> MaterializedObjectType {
    match value {
        ObjType::Map => MaterializedObjectType::Map,
        ObjType::List => MaterializedObjectType::List,
        ObjType::Table => MaterializedObjectType::Table,
        ObjType::Text => MaterializedObjectType::Text,
    }
}

fn scalar(value: &ScalarValue) -> Result<MaterializedScalar, ProjectionError> {
    match value {
        ScalarValue::Bytes(value) => Ok(MaterializedScalar::Bytes(value.clone())),
        ScalarValue::Str(value) => Ok(MaterializedScalar::String(value.to_string())),
        ScalarValue::Int(value) => Ok(MaterializedScalar::I64(*value)),
        ScalarValue::Uint(value) => Ok(MaterializedScalar::U64(*value)),
        ScalarValue::F64(value) => Ok(MaterializedScalar::F64Bits(value.to_bits())),
        ScalarValue::Counter(value) => Ok(MaterializedScalar::Counter(i64::from(value))),
        ScalarValue::Timestamp(value) => Ok(MaterializedScalar::Timestamp(*value)),
        ScalarValue::Boolean(value) => Ok(MaterializedScalar::Bool(*value)),
        ScalarValue::Null => Ok(MaterializedScalar::Null),
        ScalarValue::Unknown { .. } => Err(ProjectionError::Invalid),
    }
}

#[cfg(test)]
mod tests {
    use automerge::transaction::Transactable;
    use automerge::{
        ActorId, Automerge, ObjType, ROOT, ScalarValue, TextEncoding,
        marks::{ExpandMark, Mark},
    };

    use super::{
        MaterializedDocumentView, MaterializedPathElement, MaterializedScalar, MaterializedValue,
        ProjectionError,
    };
    use crate::{NeverCancelled, WorkBudget, WorkCounter};

    #[test]
    fn snapshot_projection_load_is_metered_before_decode() {
        let bytes = Automerge::new().save_nocompress();
        let mut exhausted = WorkBudget::new(0, 10);
        assert_eq!(
            MaterializedDocumentView::from_canonical_bytes_metered(
                bytes.clone(),
                &mut exhausted,
                &NeverCancelled,
            ),
            Err(ProjectionError::Budget)
        );
        assert_eq!(exhausted.consumed().get(WorkCounter::DecodeByte), 0);
        let mut cancelled = WorkBudget::new(u64::MAX, 10);
        assert_eq!(
            MaterializedDocumentView::from_canonical_bytes_metered(bytes, &mut cancelled, &|| true,),
            Err(ProjectionError::Cancelled)
        );
        assert_eq!(cancelled.consumed().get(WorkCounter::DecodeByte), 0);
    }

    #[test]
    fn project_every_scalar_without_json_coercion() {
        let mut document = Automerge::new_with_encoding(TextEncoding::Utf16CodeUnit);
        document.set_actor(ActorId::from([1; 32]));
        let float_bits = 0x7ff8_0000_0000_0042_u64;
        {
            let mut tx = document.transaction();
            assert!(tx.put(ROOT, "null", ()).is_ok());
            assert!(tx.put(ROOT, "bool", true).is_ok());
            assert!(tx.put(ROOT, "i64", i64::MIN).is_ok());
            assert!(tx.put(ROOT, "u64", u64::MAX).is_ok());
            assert!(
                tx.put(ROOT, "float", ScalarValue::F64(f64::from_bits(float_bits)))
                    .is_ok()
            );
            assert!(tx.put(ROOT, "negative_zero", -0.0_f64).is_ok());
            assert!(tx.put(ROOT, "bytes", vec![0, 255]).is_ok());
            assert!(tx.put(ROOT, "string", "exact").is_ok());
            assert!(
                tx.put(ROOT, "timestamp", ScalarValue::Timestamp(i64::MIN))
                    .is_ok()
            );
            assert!(
                tx.put(ROOT, "counter", ScalarValue::Counter((-7_i64).into()))
                    .is_ok()
            );
            tx.commit();
        }
        let canonical = document.save_nocompress();
        let view = MaterializedDocumentView::from_canonical_bytes(canonical.clone());
        assert!(view.is_ok());
        let Ok(view) = view else { return };
        assert_eq!(view.byte_len(), canonical.len());
        assert!(!view.is_empty());
        assert_eq!(view.entries().len(), 10);
        assert!(view.marks().is_empty());
        assert_eq!(
            format!("{view:?}"),
            format!(
                "MaterializedDocumentView {{ byte_len: {}, entry_count: 10, mark_count: 0 }}",
                canonical.len()
            )
        );
        for (key, expected) in [
            ("null", MaterializedScalar::Null),
            ("bool", MaterializedScalar::Bool(true)),
            ("i64", MaterializedScalar::I64(i64::MIN)),
            ("u64", MaterializedScalar::U64(u64::MAX)),
            ("float", MaterializedScalar::F64Bits(float_bits)),
            (
                "negative_zero",
                MaterializedScalar::F64Bits((-0.0_f64).to_bits()),
            ),
            ("bytes", MaterializedScalar::Bytes(vec![0, 255])),
            ("string", MaterializedScalar::String("exact".to_owned())),
            ("timestamp", MaterializedScalar::Timestamp(i64::MIN)),
            ("counter", MaterializedScalar::Counter(-7)),
        ] {
            assert!(
                view.entries().iter().any(|entry| {
                    entry.path() == [MaterializedPathElement::Key(key.to_owned())]
                        && matches!(
                            entry.conflicts(),
                            [conflict]
                                if conflict.value() == &MaterializedValue::Scalar(expected.clone())
                        )
                }),
                "{key}"
            );
        }
    }

    #[test]
    fn project_structured_objects_and_indexes_deterministically() {
        let mut document = Automerge::new_with_encoding(TextEncoding::Utf16CodeUnit);
        document.set_actor(ActorId::from([2; 32]));
        {
            let mut tx = document.transaction();
            let map = tx.put_object(ROOT, "map", ObjType::Map);
            let list = tx.put_object(ROOT, "list", ObjType::List);
            let table = tx.put_object(ROOT, "table", ObjType::Table);
            assert!(map.is_ok() && list.is_ok() && table.is_ok());
            let (Ok(map), Ok(list), Ok(table)) = (map, list, table) else {
                return;
            };
            assert!(tx.put(&map, "nested", "value").is_ok());
            assert!(tx.insert(&list, 0, "zero").is_ok());
            assert!(tx.insert(&list, 1, "one").is_ok());
            let _ = table;
            tx.commit();
        }
        let first = MaterializedDocumentView::from_canonical_bytes(document.save_nocompress());
        let second = MaterializedDocumentView::from_canonical_bytes(document.save_nocompress());
        assert!(first.is_ok() && second.is_ok());
        let (Ok(first), Ok(second)) = (first, second) else {
            return;
        };
        assert_eq!(first.entries(), second.entries());
        for path in [
            vec![MaterializedPathElement::Key("map".to_owned())],
            vec![
                MaterializedPathElement::Key("map".to_owned()),
                MaterializedPathElement::Key("nested".to_owned()),
            ],
            vec![MaterializedPathElement::Key("list".to_owned())],
            vec![
                MaterializedPathElement::Key("list".to_owned()),
                MaterializedPathElement::Index(0),
            ],
            vec![
                MaterializedPathElement::Key("list".to_owned()),
                MaterializedPathElement::Index(1),
            ],
            vec![MaterializedPathElement::Key("table".to_owned())],
        ] {
            assert!(first.entries().iter().any(|entry| entry.path() == path));
        }
        let object_ids = first
            .entries()
            .iter()
            .flat_map(|entry| entry.conflicts())
            .filter_map(|conflict| match conflict.value() {
                MaterializedValue::Object { object_id, .. }
                | MaterializedValue::Text { object_id, .. } => Some(object_id),
                MaterializedValue::Scalar(_) => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(object_ids.len(), 3);
        assert!(object_ids.iter().all(|identity| !identity.is_empty()));
    }

    #[test]
    fn project_text_with_utf16_code_unit_semantics() {
        let mut document = Automerge::new_with_encoding(TextEncoding::Utf16CodeUnit);
        document.set_actor(ActorId::from([3; 32]));
        {
            let mut tx = document.transaction();
            let text = tx.put_object(ROOT, "text", ObjType::Text);
            assert!(text.is_ok());
            let Ok(text) = text else { return };
            assert!(tx.splice_text(&text, 0, 0, "A😀e\u{301}").is_ok());
            tx.commit();
        }
        let view = MaterializedDocumentView::from_canonical_bytes(document.save_nocompress());
        assert!(view.is_ok());
        let Ok(view) = view else { return };
        let path = [MaterializedPathElement::Key("text".to_owned())];
        assert_eq!(view.text_utf16_len(&path), Some(5));
        assert!(view.entries().iter().any(|entry| {
            entry.path() == path
                && matches!(
                    entry.conflicts(),
                    [conflict]
                        if matches!(
                            conflict.value(),
                            MaterializedValue::Text { value, .. } if value == "A😀e\u{301}"
                        )
                )
        }));
    }

    #[test]
    fn text_projection_charges_utf16_units_before_materialization() {
        let mut document = Automerge::new_with_encoding(TextEncoding::Utf16CodeUnit);
        document.set_actor(ActorId::from([13; 32]));
        {
            let mut tx = document.transaction();
            let text = tx.put_object(ROOT, "text", ObjType::Text);
            let Ok(text) = text else { return };
            assert!(tx.splice_text(&text, 0, 0, "A😀").is_ok());
            tx.commit();
        }
        let bytes = document.save_nocompress();
        let mut exact = WorkBudget::new(u64::MAX, u64::MAX);
        let projected = MaterializedDocumentView::from_canonical_bytes_metered(
            bytes.clone(),
            &mut exact,
            &NeverCancelled,
        );
        assert!(projected.is_ok());
        let assertion_work = exact.consumed().get(WorkCounter::Assertion);
        assert!(assertion_work >= 4);

        let mut exhausted = WorkBudget::new(u64::MAX, assertion_work);
        assert_eq!(
            MaterializedDocumentView::from_canonical_bytes_metered(
                bytes,
                &mut exhausted,
                &NeverCancelled,
            ),
            Err(ProjectionError::Budget)
        );
        assert!(exhausted.consumed().get(WorkCounter::Assertion) < assertion_work);
    }

    #[test]
    fn project_all_conflicts_with_stable_operation_identity() {
        let change = |actor: u8, value: &str| {
            let mut document = Automerge::new_with_encoding(TextEncoding::Utf16CodeUnit);
            document.set_actor(ActorId::from([actor; 32]));
            {
                let mut tx = document.transaction();
                assert!(tx.put(ROOT, "conflict", value).is_ok());
                tx.commit();
            }
            document.get_changes(&[])[0].clone()
        };
        let left = change(4, "left");
        let right = change(5, "right");
        let project = |changes| {
            let mut document = Automerge::new_with_encoding(TextEncoding::Utf16CodeUnit);
            assert!(document.apply_changes(changes).is_ok());
            MaterializedDocumentView::from_canonical_bytes(document.save_nocompress())
        };
        let first = project([left.clone(), right.clone()]);
        let second = project([right, left]);
        assert!(first.is_ok() && second.is_ok());
        let (Ok(first), Ok(second)) = (first, second) else {
            return;
        };
        assert_eq!(first.entries(), second.entries());
        let entry = first
            .entries()
            .iter()
            .find(|entry| entry.path() == [MaterializedPathElement::Key("conflict".to_owned())]);
        assert!(entry.is_some());
        let Some(entry) = entry else { return };
        assert_eq!(entry.conflicts().len(), 2);
        assert!(
            entry
                .conflicts()
                .windows(2)
                .all(|pair| pair[0].operation_id() < pair[1].operation_id())
        );
        let values = entry
            .conflicts()
            .iter()
            .map(|conflict| conflict.value().clone())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            values,
            std::collections::BTreeSet::from([
                MaterializedValue::Scalar(MaterializedScalar::String("left".to_owned())),
                MaterializedValue::Scalar(MaterializedScalar::String("right".to_owned())),
            ])
        );
    }

    #[test]
    fn nested_conflicting_maps_retain_distinct_branch_paths() {
        let change = |actor: u8, value: &str| {
            let mut document = Automerge::new_with_encoding(TextEncoding::Utf16CodeUnit);
            document.set_actor(ActorId::from([actor; 32]));
            {
                let mut tx = document.transaction();
                let map = tx.put_object(ROOT, "conflict", ObjType::Map);
                let Ok(map) = map else { return None };
                if tx.put(&map, "same", value).is_err() {
                    return None;
                }
                tx.commit();
            }
            document.get_changes(&[]).first().cloned()
        };
        let (Some(left), Some(right)) = (change(20, "left"), change(21, "right")) else {
            return;
        };
        let mut document = Automerge::new_with_encoding(TextEncoding::Utf16CodeUnit);
        assert!(document.apply_changes([right, left]).is_ok());
        let view = MaterializedDocumentView::from_canonical_bytes(document.save_nocompress());
        let Ok(view) = view else { return };
        let descendants = view
            .entries()
            .iter()
            .filter(|entry| {
                matches!(
                    entry.path(),
                    [MaterializedPathElement::Key(key), MaterializedPathElement::Branch { .. }, MaterializedPathElement::Key(child)]
                        if key == "conflict" && child == "same"
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(descendants.len(), 2);
        assert_ne!(descendants[0].path(), descendants[1].path());
        let values = descendants
            .iter()
            .flat_map(|entry| entry.conflicts())
            .map(|conflict| conflict.value().clone())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            values,
            std::collections::BTreeSet::from([
                MaterializedValue::Scalar(MaterializedScalar::String("left".to_owned())),
                MaterializedValue::Scalar(MaterializedScalar::String("right".to_owned())),
            ])
        );
    }

    #[test]
    fn nested_conflicting_lists_retain_distinct_index_branches() {
        let change = |actor: u8, value: &str| {
            let mut document = Automerge::new_with_encoding(TextEncoding::Utf16CodeUnit);
            document.set_actor(ActorId::from([actor; 32]));
            {
                let mut tx = document.transaction();
                let list = tx.put_object(ROOT, "conflict", ObjType::List);
                let Ok(list) = list else { return None };
                if tx.insert(&list, 0, value).is_err() {
                    return None;
                }
                tx.commit();
            }
            document.get_changes(&[]).first().cloned()
        };
        let (Some(left), Some(right)) = (change(22, "left"), change(23, "right")) else {
            return;
        };
        let mut document = Automerge::new_with_encoding(TextEncoding::Utf16CodeUnit);
        assert!(document.apply_changes([left, right]).is_ok());
        let view = MaterializedDocumentView::from_canonical_bytes(document.save_nocompress());
        let Ok(view) = view else { return };
        let descendants = view
            .entries()
            .iter()
            .filter(|entry| {
                matches!(
                    entry.path(),
                    [MaterializedPathElement::Key(key), MaterializedPathElement::Branch { .. }, MaterializedPathElement::Index(0)]
                        if key == "conflict"
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(descendants.len(), 2);
        assert_ne!(descendants[0].path(), descendants[1].path());
    }

    #[test]
    fn conflicting_text_and_marks_retain_their_text_branch() {
        let change = |actor: u8, value: &str| {
            let mut document = Automerge::new_with_encoding(TextEncoding::Utf16CodeUnit);
            document.set_actor(ActorId::from([actor; 32]));
            {
                let mut tx = document.transaction();
                let text = tx.put_object(ROOT, "conflict", ObjType::Text);
                let Ok(text) = text else { return None };
                if tx.splice_text(&text, 0, 0, value).is_err()
                    || tx
                        .mark(
                            &text,
                            Mark::new("branch".to_owned(), true, 0, value.len()),
                            ExpandMark::Both,
                        )
                        .is_err()
                {
                    return None;
                }
                tx.commit();
            }
            document.get_changes(&[]).first().cloned()
        };
        let (Some(left), Some(right)) = (change(24, "left"), change(25, "right")) else {
            return;
        };
        let mut document = Automerge::new_with_encoding(TextEncoding::Utf16CodeUnit);
        assert!(document.apply_changes([right, left]).is_ok());
        let view = MaterializedDocumentView::from_canonical_bytes(document.save_nocompress());
        let Ok(view) = view else { return };
        let text = view
            .entries()
            .iter()
            .find(|entry| entry.path() == [MaterializedPathElement::Key("conflict".to_owned())]);
        assert!(text.is_some_and(|entry| entry.conflicts().len() == 2));
        assert_eq!(view.marks().len(), 2);
        assert!(view.marks().iter().all(|mark| matches!(
            mark.path(),
            [MaterializedPathElement::Key(key), MaterializedPathElement::Branch { .. }]
                if key == "conflict"
        )));
        assert_ne!(view.marks()[0].path(), view.marks()[1].path());
    }

    #[test]
    fn project_real_marks_with_utf16_ranges() {
        let mut document = Automerge::new_with_encoding(TextEncoding::Utf16CodeUnit);
        document.set_actor(ActorId::from([6; 32]));
        let text;
        {
            let mut tx = document.transaction();
            let created = tx.put_object(ROOT, "text", ObjType::Text);
            assert!(created.is_ok());
            let Ok(created) = created else { return };
            text = created;
            assert!(tx.splice_text(&text, 0, 0, "A😀e\u{301}").is_ok());
            tx.commit();
        }
        document.set_actor(ActorId::from([7; 32]));
        {
            let mut tx = document.transaction();
            assert!(
                tx.mark(
                    &text,
                    Mark::new("bold".to_owned(), true, 1, 3),
                    ExpandMark::Both,
                )
                .is_ok()
            );
            tx.commit();
        }
        let view = MaterializedDocumentView::from_canonical_bytes(document.save_nocompress());
        assert!(view.is_ok());
        let Ok(view) = view else { return };
        assert_eq!(view.marks().len(), 1);
        let mark = &view.marks()[0];
        assert_eq!(
            mark.path(),
            [MaterializedPathElement::Key("text".to_owned())]
        );
        assert_eq!(mark.name(), "bold");
        assert_eq!(mark.value(), &MaterializedScalar::Bool(true));
        assert_eq!((mark.start(), mark.end()), (1, 3));
        assert_eq!(view.text_utf16_len(mark.path()), Some(5));
    }

    #[test]
    fn deeply_nested_projection_uses_an_explicit_stack() {
        let mut document = Automerge::new_with_encoding(TextEncoding::Utf16CodeUnit);
        document.set_actor(ActorId::from([8; 32]));
        {
            let mut tx = document.transaction();
            let first = tx.put_object(ROOT, "root", ObjType::Map);
            let Ok(mut parent) = first else { return };
            for _ in 0..2_048 {
                let child = tx.put_object(&parent, "child", ObjType::Map);
                let Ok(child) = child else { return };
                parent = child;
            }
            assert!(tx.put(&parent, "value", true).is_ok());
            tx.commit();
        }
        let view = MaterializedDocumentView::from_canonical_bytes(document.save_nocompress());
        assert!(view.is_ok_and(|view| view.entries().len() == 2_050));
    }
}
