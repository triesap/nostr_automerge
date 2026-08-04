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

    pub(crate) fn assert_all(&self, assertions: &[TypedAssertion]) -> bool {
        assertions.iter().all(|assertion| self.assert(assertion))
    }
}

#[cfg(test)]
mod tests {
    use super::{ExpectedValue, OpaqueDocumentView, TypedAssertion, TypedValue};

    #[test]
    fn evaluate_primitive_state_assertions() {
        let nan = f64::from_bits(0x7ff8_0000_0000_0042).to_bits();
        let negative_zero = (-0.0_f64).to_bits();
        let entries = vec![
            ("null", TypedValue::Null),
            ("bool", TypedValue::Bool(true)),
            ("i64", TypedValue::I64(i64::MIN)),
            ("u64", TypedValue::U64(u64::MAX)),
            ("nan", TypedValue::F64Bits(nan)),
            ("negative_zero", TypedValue::F64Bits(negative_zero)),
            ("string", TypedValue::String("scalar".to_owned())),
            ("text", TypedValue::Text("text".to_owned())),
            ("bytes", TypedValue::Bytes(vec![0, 255])),
            ("timestamp", TypedValue::Timestamp(-1)),
            ("counter", TypedValue::Counter(7)),
        ];
        let view = OpaqueDocumentView::from_typed_values(
            entries
                .iter()
                .map(|(path, value)| (vec![(*path).to_owned()], vec![value.clone()])),
        );
        let assertions = entries
            .iter()
            .map(|(path, value)| TypedAssertion {
                path: vec![(*path).to_owned()],
                expected: ExpectedValue::Value(value.clone()),
            })
            .collect::<Vec<_>>();
        assert!(view.assert_all(&assertions));
        for assertion in &assertions {
            let mut negative = assertion.clone();
            negative.expected = ExpectedValue::Value(TypedValue::Null);
            if assertion.expected != negative.expected {
                assert!(!view.assert(&negative));
            }
        }
        assert!(!view.assert(&TypedAssertion {
            path: vec!["nan".to_owned()],
            expected: ExpectedValue::Value(TypedValue::F64Bits(f64::NAN.to_bits())),
        }));
        assert!(!view.assert(&TypedAssertion {
            path: vec!["negative_zero".to_owned()],
            expected: ExpectedValue::Value(TypedValue::F64Bits(0.0_f64.to_bits())),
        }));
    }
}
