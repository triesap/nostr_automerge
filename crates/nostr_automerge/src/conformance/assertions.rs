use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum TypedValue {
    Null,
    Bool(bool),
    I64(i64),
    U64(u64),
    F64Bits(u64),
    String(String),
    Text(String),
    Bytes(Vec<u8>),
    Timestamp(i64),
    Counter(i64),
    List,
    Map,
    Table,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ExpectedValue {
    Value(TypedValue),
    Conflicts(Vec<TypedValue>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TypedAssertion {
    pub(crate) path: Vec<String>,
    pub(crate) expected: ExpectedValue,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct OpaqueDocumentView {
    values: BTreeMap<Vec<String>, Vec<TypedValue>>,
}

impl OpaqueDocumentView {
    pub(crate) fn from_typed_values(
        values: impl IntoIterator<Item = (Vec<String>, Vec<TypedValue>)>,
    ) -> Self {
        Self {
            values: values.into_iter().collect(),
        }
    }

    pub(crate) fn assert(&self, assertion: &TypedAssertion) -> bool {
        let Some(actual) = self.values.get(&assertion.path) else {
            return false;
        };
        match &assertion.expected {
            ExpectedValue::Value(expected) => actual.as_slice() == [expected.clone()],
            ExpectedValue::Conflicts(expected) => actual == expected,
        }
    }
}
