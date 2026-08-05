//! Deterministic committed checkpoint conformance evidence.
#[test]
fn publish_checkpoint_conformance_fixtures_and_report() {
    let fixture = include_str!("../../../fixtures/v1_draft/checkpoints/cases.json");
    let report = include_str!("../../../reports/checkpoint_conformance.json");
    let fixture: serde_json::Value = serde_json::from_str(fixture).unwrap_or_default();
    let report: serde_json::Value = serde_json::from_str(report).unwrap_or_default();
    assert_eq!(fixture["invalid"].as_array().map(Vec::len), Some(10));
    assert_eq!(report["result"], "limited_pass");
    assert_eq!(report["signed_carrier_integration"], "not_completed");
    assert_eq!(report["full_replay_required"], true);
    assert!(
        !include_str!("../../../reports/checkpoint_conformance.md").contains("missing-history")
    );
}
