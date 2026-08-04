use std::collections::BTreeSet;

use automerge::{
    ActorId, Automerge, ObjType, ROOT, ReadDoc, ScalarValue, TextEncoding,
    marks::{ExpandMark, Mark},
    transaction::{CommitOptions, Transactable},
};

use crate::ProtocolRevision;

use super::encode::qualify_canonical_reencoding;
use super::types::{Action, ObjectKind, Scalar};

#[test]
fn add_complete_automerge_semantic_matrix() {
    let mut document = Automerge::new_with_encoding(TextEncoding::Utf16CodeUnit);
    document.set_actor(ActorId::from([0x11; 32]));
    let text;
    {
        let mut transaction = document.transaction();
        let map = transaction.put_object(ROOT, "map", ObjType::Map);
        let list = transaction.put_object(ROOT, "list", ObjType::List);
        let created_text = transaction.put_object(ROOT, "text", ObjType::Text);
        let table = transaction.put_object(ROOT, "table", ObjType::Table);
        assert!(map.is_ok() && list.is_ok() && created_text.is_ok() && table.is_ok());
        let (map, list, created_text) = match (map, list, created_text, table) {
            (Ok(map), Ok(list), Ok(text), Ok(_table)) => (map, list, text),
            _ => return,
        };
        text = created_text;
        assert!(transaction.put(&map, "nested", "map-value").is_ok());
        assert!(transaction.insert(&list, 0, "list-value").is_ok());
        assert!(transaction.splice_text(&text, 0, 0, "A😀e\u{301}").is_ok());
        assert!(transaction.put(ROOT, "bytes", vec![0, 1, 255]).is_ok());
        assert!(transaction.put(ROOT, "bool", true).is_ok());
        assert!(transaction.put(ROOT, "null", ()).is_ok());
        assert!(transaction.put(ROOT, "int", -7_i64).is_ok());
        assert!(transaction.put(ROOT, "uint", 7_u64).is_ok());
        assert!(
            transaction
                .put(ROOT, "timestamp", ScalarValue::Timestamp(-9))
                .is_ok()
        );
        assert!(
            transaction
                .put(
                    ROOT,
                    "f64",
                    ScalarValue::F64(f64::from_bits(0x3ff0_0000_0000_0001))
                )
                .is_ok()
        );
        assert!(
            transaction
                .put(ROOT, "counter", ScalarValue::Counter(5_i64.into()))
                .is_ok()
        );
        assert!(transaction.put(ROOT, "delete_me", "temporary").is_ok());
        transaction.commit();
    }

    document.set_actor(ActorId::from([0x22; 32]));
    {
        let mut transaction = document.transaction();
        assert!(transaction.delete(ROOT, "delete_me").is_ok());
        assert!(transaction.increment(ROOT, "counter", 3).is_ok());
        assert!(
            transaction
                .mark(
                    &text,
                    Mark::new("bold".to_owned(), true, 1, 3),
                    ExpandMark::Both,
                )
                .is_ok()
        );
        transaction.commit();
    }

    document.set_actor(ActorId::from([0x33; 32]));
    document.empty_commit(CommitOptions::default());

    let changes = document.get_changes(&[]);
    assert_eq!(changes.len(), 3);
    let mut labels = BTreeSet::new();
    let mut decoded = Vec::new();
    for change in &changes {
        let qualified =
            qualify_canonical_reencoding(change.raw_bytes(), ProtocolRevision::draft_v1());
        assert!(qualified.is_ok());
        let qualified = match qualified {
            Ok(value) => value,
            Err(_) => return,
        };
        if qualified.operations.is_empty() {
            labels.insert("empty_merge");
        }
        if qualified.actor.as_bytes() == &[0x22; 32] {
            labels.insert("other_actor");
        }
        for operation in &qualified.operations {
            match &operation.action {
                Action::Make(ObjectKind::Map) => labels.insert("make_map"),
                Action::Make(ObjectKind::List) => labels.insert("make_list"),
                Action::Make(ObjectKind::Text) => labels.insert("make_text"),
                Action::Make(ObjectKind::Table) => labels.insert("make_table"),
                Action::Delete => labels.insert("delete"),
                Action::Increment(_) => labels.insert("increment"),
                Action::MarkBegin { .. } => labels.insert("mark_begin"),
                Action::MarkEnd { .. } => labels.insert("mark_end"),
                Action::Set(value) => match value {
                    Scalar::Bytes(_) => labels.insert("bytes"),
                    Scalar::String(value) if value.contains('😀') => labels.insert("unicode"),
                    Scalar::String(_) => labels.insert("string"),
                    Scalar::Int(_) => labels.insert("int"),
                    Scalar::Uint(_) => labels.insert("uint"),
                    Scalar::F64Bits(0x3ff0_0000_0000_0001) => labels.insert("f64_bits"),
                    Scalar::F64Bits(_) => labels.insert("f64"),
                    Scalar::Counter(_) => labels.insert("counter"),
                    Scalar::Timestamp(_) => labels.insert("timestamp"),
                    Scalar::Boolean(_) => labels.insert("bool"),
                    Scalar::Null => labels.insert("null"),
                },
            };
        }
        decoded.push(qualified);
    }

    let expected: Result<Vec<String>, _> = serde_json::from_str(include_str!(
        "../../../../fixtures/v1_draft/automerge_semantics/matrix.json"
    ));
    assert!(expected.is_ok());
    let expected: BTreeSet<&str> = expected
        .as_deref()
        .unwrap_or_default()
        .iter()
        .map(String::as_str)
        .collect();
    assert_eq!(labels, expected);

    let mut replay = Automerge::new_with_encoding(TextEncoding::Utf16CodeUnit);
    assert!(replay.apply_changes(changes).is_ok());
    assert_eq!(replay.get_heads(), document.get_heads());
    assert_eq!(replay.text(&text).as_deref(), Ok("A😀e\u{301}"));
    assert!(matches!(
        replay.get(ROOT, "counter"),
        Ok(Some((automerge::Value::Scalar(value), _)))
            if matches!(value.as_ref(), ScalarValue::Counter(counter) if i64::from(counter) == 8)
    ));
    assert!(!decoded.is_empty());
}
