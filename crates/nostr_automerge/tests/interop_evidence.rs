//! Verification for committed independent interoperability evidence.

use serde_json::Value;

#[test]
fn record_independent_typescript_differential_agreement() {
    let report: Result<Value, _> =
        serde_json::from_str(include_str!("../../../reports/interop_differential.json"));
    assert!(report.is_ok());
    let Ok(report) = report else { return };
    assert_eq!(report["schema"], "nostr_automerge.local_interop.v1");
    assert_eq!(report["status"], "local_differential_pass");
    assert_eq!(report["fixture_count"], 5);
    assert_eq!(report["canonical_report_bytes"], "identical");
    assert_eq!(report["deliberate_mismatch"], "detected");
    assert_eq!(report["ci_policy"], "local_act_pass");
    assert_eq!(report["mismatches"].as_array().map(Vec::len), Some(0));
    assert_eq!(
        report["corpus_sha256"],
        "e1c96aa1046df5108c713d6484857d2030cb73ab1ac668f3aac28821f71779d4"
    );
    assert_eq!(
        report["evaluated_typescript_commit"].as_str().map(str::len),
        Some(40)
    );
    let narrative = include_str!("../../../reports/interop_differential.md");
    assert!(narrative.contains("No workflow definition"));
    assert!(narrative.contains("is tracked"));
}
