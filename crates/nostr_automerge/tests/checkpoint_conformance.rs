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
        "nostr_automerge.checkpoint_conformance.v2"
    );
    assert_eq!(report["result"], "passed");
    assert_eq!(report["signed_carrier_integration"], "passed");
    assert_eq!(report["full_replay_required"], true);
    assert_eq!(report["evaluated_commit"].as_str().map(str::len), Some(40));
    assert_eq!(report["gates"].as_array().map(Vec::len), Some(12));
    assert!(
        report["gates"]
            .as_array()
            .is_some_and(|gates| gates.iter().all(|gate| gate["result"] == "passed"))
    );
    assert_eq!(
        report["sources"].as_object().map(serde_json::Map::len),
        Some(3)
    );
    let evidence = report["evidence"].as_array().cloned().unwrap_or_default();
    assert_eq!(evidence.len(), 31);
    assert!(evidence.iter().all(|row| {
        row["result"] == "passed"
            && row["result_sha256"].as_str().map(str::len) == Some(64)
            && row["fixture_id"].as_str().is_some()
            && row["expected_status"].as_str().is_some()
            && row["public_engine_test"].as_str().is_some()
    }));
    assert!(
        include_str!("../../../reports/checkpoint_conformance.md")
            .contains("signed descriptor/chunk pipeline")
    );
}
