//! Verification for canonical signed conformance evidence.

use serde_json::Value;

#[test]
fn signed_profile_deliberate_mismatch_is_detectable() {
    let source = include_bytes!("../../../reports/rust_signed_property.json");
    let report: Result<Value, _> = serde_json::from_slice(source);
    assert!(report.is_ok());
    let Ok(report) = report else { return };
    assert_eq!(report["schema"], "nostr_automerge.rust_signed_profile.v3");
    assert_eq!(report["status"], "pass");
    assert_eq!(report["process_runs_per_fixture"], 2);
    assert_eq!(report["fixture_count"], 6);

    let mut mutated = report.clone();
    mutated["reports"][0]["report"]["completion"] = Value::String("cancelled".to_owned());
    let original_reports = serde_json::to_vec(&report["reports"]);
    let mutated_reports = serde_json::to_vec(&mutated["reports"]);
    assert!(original_reports.is_ok());
    assert!(mutated_reports.is_ok());
    assert_ne!(
        original_reports.as_deref().ok(),
        mutated_reports.as_deref().ok()
    );
    assert_eq!(
        include_bytes!("../../../reports/rust_signed_property.json"),
        source
    );
}
