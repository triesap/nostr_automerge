use std::collections::BTreeSet;

use serde_json::Value;

use super::select::select_child;
use crate::EventId;

fn event(value: u64) -> EventId {
    EventId::from_bytes([u8::try_from(value).unwrap_or_default(); 32])
}

#[test]
fn add_complete_control_scenario_permutation_suite() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../../fixtures/v1_draft/controls/scenarios.json"
    ))
    .unwrap_or(Value::Null);
    let scenarios = fixture
        .get("scenarios")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    assert_eq!(scenarios.len(), 10);
    let names = scenarios
        .iter()
        .filter_map(|scenario| scenario.get("name").and_then(Value::as_str))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        names,
        BTreeSet::from([
            "acl_transition",
            "freeze",
            "genesis_fork",
            "missing_evidence",
            "reorganization",
            "retained_frontier",
            "successor_continuity",
            "terminal",
            "transition_matrix",
            "unauthorized_child",
        ])
    );

    for scenario in scenarios {
        let valid = scenario
            .get("valid_candidates")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|value| value.as_u64().map(event))
            .collect::<Vec<_>>();
        let expected = scenario.get("selected").and_then(Value::as_u64).map(event);
        let pending = scenario
            .get("pending")
            .and_then(Value::as_array)
            .map_or(0, Vec::len);
        for seed in 0..32 {
            let mut permuted = valid.clone();
            if !permuted.is_empty() {
                let length = permuted.len();
                permuted.rotate_left(seed % length);
                if seed % 2 == 1 {
                    permuted.reverse();
                }
            }
            assert_eq!(select_child(permuted).selected, expected);
            assert_eq!(
                scenario
                    .get("pending")
                    .and_then(Value::as_array)
                    .map_or(0, Vec::len),
                pending
            );
        }
    }
}
