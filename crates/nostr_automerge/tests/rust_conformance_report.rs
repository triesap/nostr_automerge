//! Validation for generated executable Rust conformance evidence.

#[test]
fn publish_full_rust_conformance_report() {
    let report: serde_json::Value =
        serde_json::from_str(include_str!("../../../reports/rust_conformance.json"))
            .unwrap_or_default();
    assert_eq!(report["schema"], "nostr_automerge.rust_conformance.v1");
    assert_eq!(report["completion"], "complete");
    assert_eq!(report["failures"].as_array().map(Vec::len), Some(0));
    assert_eq!(report["commit"].as_str().map(str::len), Some(40));
    assert_eq!(
        report["family_source_sha256"]
            .as_object()
            .map(serde_json::Map::len),
        Some(6)
    );
    assert_eq!(report["test_gates"].as_array().map(Vec::len), Some(4));
    assert_eq!(
        report["fixture_report_sha256"]
            .as_object()
            .map(serde_json::Map::len),
        report["fixture_count"]
            .as_u64()
            .and_then(|value| usize::try_from(value).ok())
    );
    assert!(
        report["requirement_count"]
            .as_u64()
            .is_some_and(|count| count > 0)
    );
}
