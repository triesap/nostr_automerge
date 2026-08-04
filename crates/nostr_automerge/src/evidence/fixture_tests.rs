use serde_json::Value;

use super::indexes::{
    ChangeIndexRecord, ControlIndexRecord, IndexValidity, index_changes, index_controls,
};
use crate::{ActorId, ChangeHash, EventId};

fn byte(object: &serde_json::Map<String, Value>, key: &str) -> u8 {
    object
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|value| u8::try_from(value).ok())
        .unwrap_or_default()
}

fn validity(object: &serde_json::Map<String, Value>) -> IndexValidity {
    match object.get("validity").and_then(Value::as_str) {
        Some("valid") => IndexValidity::Valid,
        Some("pending") => IndexValidity::Pending,
        _ => IndexValidity::Invalid,
    }
}

#[test]
fn add_carrier_evidence_integration_fixtures() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../../fixtures/v1_draft/carriers/evidence_scenario.json"
    ))
    .unwrap_or(Value::Null);
    let events = fixture
        .get("events")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let expected = fixture.get("expected").and_then(Value::as_object);
    assert!(expected.is_some());
    let expected = match expected {
        Some(expected) => expected,
        None => return,
    };
    assert_eq!(
        events.len() as u64,
        expected
            .get("categories")
            .and_then(Value::as_u64)
            .unwrap_or_default()
    );

    let evaluate = |ordered: &[Value]| {
        let mut controls = Vec::new();
        let mut changes = Vec::new();
        for event in ordered {
            let Some(object) = event.as_object() else {
                continue;
            };
            match object.get("category").and_then(Value::as_str) {
                Some("control") => controls.push(ControlIndexRecord {
                    event_id: EventId::from_bytes([byte(object, "event"); 32]),
                    parent: object.get("parent").and_then(Value::as_u64).map(|parent| {
                        EventId::from_bytes([u8::try_from(parent).unwrap_or_default(); 32])
                    }),
                    validity: validity(object),
                }),
                Some("change") => changes.push(ChangeIndexRecord {
                    event_id: EventId::from_bytes([byte(object, "event"); 32]),
                    change_hash: ChangeHash::from_bytes([byte(object, "hash"); 32]),
                    control_id: EventId::from_bytes([byte(object, "control"); 32]),
                    actor: ActorId::from_bytes([byte(object, "actor"); 32]),
                    dependencies: Vec::new(),
                    validity: validity(object),
                }),
                _ => {}
            }
        }
        (index_controls(controls), index_changes(changes))
    };

    let baseline = evaluate(&events);
    for seed in 0..32 {
        let mut permuted = events.clone();
        if !permuted.is_empty() {
            let length = permuted.len();
            permuted.rotate_left(seed % length);
            if seed % 2 == 1 {
                permuted.reverse();
            }
        }
        assert_eq!(baseline, evaluate(&permuted));
    }

    let (controls, changes) = baseline;
    assert_eq!(controls.controls_by_id.len(), 2);
    assert_eq!(controls.invalid.len(), 1);
    assert_eq!(changes.valid_carriers_by_hash.len(), 1);
    let hash = ChangeHash::from_bytes([9; 32]);
    assert_eq!(changes.valid_carriers_by_hash[&hash].len(), 2);
    assert_eq!(changes.invalid_carriers_by_hash[&hash].len(), 1);
    assert_eq!(
        changes.preferred_valid_carrier[&hash],
        EventId::from_bytes([5; 32])
    );
}
