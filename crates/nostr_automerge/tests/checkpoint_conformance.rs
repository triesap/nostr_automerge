//! Deterministic committed checkpoint conformance evidence.
#[test]
fn publish_checkpoint_conformance_fixtures_and_report() {
    let fixture = include_str!("../../../fixtures/v1_draft/checkpoints/cases.json");
    let report = include_str!("../../../reports/checkpoint_conformance.json");
    let fixture: serde_json::Value = serde_json::from_str(fixture).unwrap_or_default();
    let report: serde_json::Value = serde_json::from_str(report).unwrap_or_default();
    assert_eq!(fixture["invalid"].as_array().map(Vec::len), Some(10));
    assert_eq!(
        report["schema"],
        "nostr_automerge.checkpoint_conformance.v1"
    );
    assert_eq!(report["result"], "passed");
    assert_eq!(report["signed_carrier_integration"], "passed");
    assert_eq!(report["full_replay_required"], true);
    assert_eq!(report["evaluated_commit"].as_str().map(str::len), Some(40));
    assert_eq!(report["gates"].as_array().map(Vec::len), Some(4));
    assert!(
        report["gates"]
            .as_array()
            .is_some_and(|gates| gates.iter().all(|gate| gate["result"] == "passed"))
    );
    assert_eq!(
        report["sources"].as_object().map(serde_json::Map::len),
        Some(3)
    );
    assert!(
        include_str!("../../../reports/checkpoint_conformance.md")
            .contains("signed descriptor/chunk pipeline")
    );
}
