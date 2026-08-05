use core::fmt;

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
}

/// Immutable exact projection of a materialized Automerge document.
#[derive(Clone, PartialEq, Eq)]
pub struct MaterializedDocumentView {
    canonical_bytes: Vec<u8>,
    entries: Vec<MaterializedEntry>,
    marks: Vec<MaterializedMark>,
}

impl MaterializedDocumentView {
    pub(crate) fn from_canonical_bytes(canonical_bytes: Vec<u8>) -> Result<Self, ProjectionError> {
        let options = LoadOptions::new()
            .text_encoding(TextEncoding::Utf16CodeUnit)
            .migrate_strings(StringMigration::NoMigration)
            .on_partial_load(OnPartialLoad::Error)
            .verification_mode(VerificationMode::Check);
        let document =
            Automerge::load_with_options(&canonical_bytes, options).map_err(|_| ProjectionError)?;
        let mut entries = Vec::new();
        let mut marks = Vec::new();
        project_object(
            &document,
            &ROOT,
            MaterializedObjectType::Map,
            &[],
            &mut entries,
            &mut marks,
        )?;
        entries.sort_by(|left, right| left.path.cmp(&right.path));
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
pub(crate) struct ProjectionError;

fn project_object(
    document: &Automerge,
    object: &ObjId,
    object_type: MaterializedObjectType,
    path: &[MaterializedPathElement],
    entries: &mut Vec<MaterializedEntry>,
    marks: &mut Vec<MaterializedMark>,
) -> Result<(), ProjectionError> {
    match object_type {
        MaterializedObjectType::Map | MaterializedObjectType::Table => {
            let mut keys = document.keys(object).collect::<Vec<_>>();
            keys.sort();
            keys.dedup();
            for key in keys {
                let mut next = path.to_vec();
                next.push(MaterializedPathElement::Key(key.clone()));
                project_property(document, object, key, next, entries, marks)?;
            }
        }
        MaterializedObjectType::List => {
            for index in 0..document.length(object) {
                let mut next = path.to_vec();
                next.push(MaterializedPathElement::Index(
                    u64::try_from(index).map_err(|_| ProjectionError)?,
                ));
                project_property(document, object, index, next, entries, marks)?;
            }
        }
        MaterializedObjectType::Text => {
            for mark in document.marks(object).map_err(|_| ProjectionError)? {
                marks.push(MaterializedMark {
                    path: path.to_vec(),
                    name: mark.name.to_string(),
                    value: scalar(&mark.value)?,
                    start: u64::try_from(mark.start).map_err(|_| ProjectionError)?,
                    end: u64::try_from(mark.end).map_err(|_| ProjectionError)?,
                });
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
    marks: &mut Vec<MaterializedMark>,
) -> Result<(), ProjectionError> {
    let mut conflicts = Vec::new();
    let mut objects = Vec::new();
    for (value, id) in document
        .get_all(object, property)
        .map_err(|_| ProjectionError)?
    {
        if let Value::Object(kind) = &value {
            objects.push((object_type(*kind), id.clone()));
        }
        conflicts.push(MaterializedConflict {
            operation_id: id.to_string(),
            value: value_at(document, value, &id)?,
        });
    }
    conflicts.sort_by(|left, right| left.operation_id.cmp(&right.operation_id));
    entries.push(MaterializedEntry {
        path: path.clone(),
        conflicts,
    });
    for (kind, id) in objects {
        project_object(document, &id, kind, &path, entries, marks)?;
    }
    Ok(())
}

fn value_at(
    document: &Automerge,
    value: Value<'_>,
    id: &ObjId,
) -> Result<MaterializedValue, ProjectionError> {
    match value {
        Value::Scalar(value) => Ok(MaterializedValue::Scalar(scalar(value.as_ref())?)),
        Value::Object(ObjType::Text) => Ok(MaterializedValue::Text {
            object_id: id.to_string(),
            value: document.text(id).map_err(|_| ProjectionError)?,
        }),
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
        ScalarValue::Unknown { .. } => Err(ProjectionError),
    }
}
