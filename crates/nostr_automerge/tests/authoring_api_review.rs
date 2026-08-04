//! Authoring public-surface review evidence.

#[test]
fn publish_authoring_api_review_report() {
    let report = include_str!("../../../reports/authoring_api_review.md");
    for heading in [
        "## Public surface",
        "## State safety",
        "## Semver assessment",
        "## Evidence reviewed",
        "## Approved gaps",
    ] {
        assert!(report.contains(heading));
    }
    let public = include_str!("../src/authoring/mod.rs");
    for forbidden in ["automerge::", "secp256k1::", "tokio::", "reqwest::"] {
        assert!(!public.contains(forbidden));
    }
}
