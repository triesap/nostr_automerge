use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum PathElement {
    Key(String),
    Index(u64),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ObjectType {
    Map,
    List,
    Table,
    Text,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum MarkValue {
    Null,
    Bool(bool),
    String(String),
}

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
    Object {
        object_type: ObjectType,
        object_id: Vec<u8>,
    },
    Mark {
        name: String,
        value: MarkValue,
        start: u64,
        end: u64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ExpectedValue {
    Value(TypedValue),
    Conflicts(Vec<TypedValue>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TypedAssertion {
    pub(crate) path: Vec<PathElement>,
    pub(crate) expected: ExpectedValue,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct OpaqueDocumentView {
    values: BTreeMap<Vec<PathElement>, Vec<TypedValue>>,
}

impl OpaqueDocumentView {
    pub(crate) fn from_typed_values(
        values: impl IntoIterator<Item = (Vec<PathElement>, Vec<TypedValue>)>,
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
            ExpectedValue::Conflicts(expected) => expected.len() >= 2 && actual == expected,
        }
    }

    pub(crate) fn assert_all(&self, assertions: &[TypedAssertion]) -> bool {
        assertions.iter().all(|assertion| self.assert(assertion))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ExpectedValue, MarkValue, ObjectType, OpaqueDocumentView, PathElement, TypedAssertion,
        TypedValue,
    };

    fn key(value: &str) -> PathElement {
        PathElement::Key(value.to_owned())
    }

    #[test]
    fn evaluate_primitive_state_assertions() {
        let entries = vec![
            ("null", TypedValue::Null),
            ("bool", TypedValue::Bool(true)),
            ("i64", TypedValue::I64(i64::MIN)),
            ("u64", TypedValue::U64(u64::MAX)),
            ("nan", TypedValue::F64Bits(0x7ff8_0000_0000_0042)),
            ("negative_zero", TypedValue::F64Bits((-0.0_f64).to_bits())),
            ("string", TypedValue::String("scalar".to_owned())),
            ("text", TypedValue::Text("text".to_owned())),
            ("bytes", TypedValue::Bytes(vec![0, 255])),
            ("timestamp", TypedValue::Timestamp(-1)),
            ("counter", TypedValue::Counter(7)),
        ];
        let view = OpaqueDocumentView::from_typed_values(
            entries
                .iter()
                .map(|(path, value)| (vec![key(path)], vec![value.clone()])),
        );
        let assertions = entries
            .iter()
            .map(|(path, value)| TypedAssertion {
                path: vec![key(path)],
                expected: ExpectedValue::Value(value.clone()),
            })
            .collect::<Vec<_>>();
        assert!(view.assert_all(&assertions));
        assert!(!view.assert(&TypedAssertion {
            path: vec![key("nan")],
            expected: ExpectedValue::Value(TypedValue::F64Bits(f64::NAN.to_bits())),
        }));
        assert!(!view.assert(&TypedAssertion {
            path: vec![key("negative_zero")],
            expected: ExpectedValue::Value(TypedValue::F64Bits(0.0_f64.to_bits())),
        }));
    }

    #[test]
    fn implement_object_text_mark_and_conflict_assertions() {
        let nested = vec![key("root"), key("items"), PathElement::Index(1)];
        let object = TypedValue::Object {
            object_type: ObjectType::List,
            object_id: vec![0x42; 16],
        };
        let mark = TypedValue::Mark {
            name: "bold".to_owned(),
            value: MarkValue::Bool(true),
            start: 0,
            end: 2,
        };
        let conflicts = vec![
            TypedValue::String("left".to_owned()),
            TypedValue::String("right".to_owned()),
        ];
        let view = OpaqueDocumentView::from_typed_values([
            (vec![key("root")], vec![object.clone()]),
            (nested.clone(), vec![TypedValue::Text("🙂".to_owned())]),
            (vec![key("mark")], vec![mark.clone()]),
            (vec![key("conflict")], conflicts.clone()),
            (
                vec![key("map")],
                vec![TypedValue::Object {
                    object_type: ObjectType::Map,
                    object_id: vec![1],
                }],
            ),
            (
                vec![key("table")],
                vec![TypedValue::Object {
                    object_type: ObjectType::Table,
                    object_id: vec![2],
                }],
            ),
            (
                vec![key("text_object")],
                vec![TypedValue::Object {
                    object_type: ObjectType::Text,
                    object_id: vec![3],
                }],
            ),
        ]);
        assert!(view.assert(&TypedAssertion {
            path: nested,
            expected: ExpectedValue::Value(TypedValue::Text("🙂".to_owned()))
        }));
        assert!(view.assert(&TypedAssertion {
            path: vec![key("mark")],
            expected: ExpectedValue::Value(mark)
        }));
        assert!(view.assert(&TypedAssertion {
            path: vec![key("conflict")],
            expected: ExpectedValue::Conflicts(conflicts.clone())
        }));
        assert!(!view.assert(&TypedAssertion {
            path: vec![key("missing")],
            expected: ExpectedValue::Value(object)
        }));
        assert!(!view.assert(&TypedAssertion {
            path: vec![key("conflict")],
            expected: ExpectedValue::Conflicts(vec![conflicts[0].clone()])
        }));
    }
}
