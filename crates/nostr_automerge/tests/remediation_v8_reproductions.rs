//! Ignored expected-failure reproductions for the remediation-v8 source review.
//!
//! The repository-owned harness runs each test individually and requires its
//! exact diagnostic at the bound baseline. The closing checkpoint enables the
//! assertion only after the corresponding implementation is green.

#[test]
fn finding_066_branch_results_reach_final_claim_reduction() {
    let evaluation = include_str!("../src/reference/evaluate.rs");
    let reducer = include_str!("../src/engine/reference_evaluator.rs");
    assert!(
        evaluation.contains("branch_change_dispositions")
            && reducer.contains("referenced_branch_change_disposition"),
        "FINDING_066 reproduced: final reduction cannot query branch-local change outcomes"
    );
}

#[test]
fn finding_067_control_work_is_coordinate_scoped() {
    let indexes = include_str!("../src/evidence/indexes.rs");
    let view = include_str!("../src/evidence/document_view.rs");
    assert!(
        indexes.contains("control_children_by_coordinate_parent")
            && view.contains("control_children("),
        "FINDING_067 reproduced: target control work lacks coordinate-scoped parent edges"
    );
}

#[test]
fn finding_068_interrupted_report_work_is_settled_by_pass() {
    let source = include_str!("../src/engine/reference_evaluator.rs");
    assert!(
        source.contains("InterruptedReportPass")
            && source.contains("prepare_interrupted_batch_report"),
        "FINDING_068 reproduced: interrupted report work occurs after coarse settlement"
    );
}

#[test]
fn finding_069_change_carriers_have_event_dispositions() {
    let source = include_str!("../src/engine/reference_evaluator.rs");
    assert!(
        source.contains("change_carrier_dispositions"),
        "FINDING_069 reproduced: change carriers have no dynamic Event dispositions"
    );
}

#[test]
fn finding_070_local_nip_contains_reconciled_branch_rules() {
    let source = include_str!("../../../spec/NIP_DRAFT.md");
    assert!(
        source.contains("## Branch-local change outcomes"),
        "FINDING_070 reproduced: the local NIP lacks reconciled branch-local outcomes"
    );
}

#[test]
#[ignore = "expected to fail at the remediation-v8 baseline"]
fn finding_071_distribution_contains_180_scenarios() {
    let manifest = include_str!("../../../fixtures/distribution/manifest_v8.json");
    let value: serde_json::Value =
        serde_json::from_str(manifest).unwrap_or(serde_json::Value::Null);
    assert_eq!(
        value
            .get("fixture_count")
            .and_then(serde_json::Value::as_u64),
        Some(180),
        "FINDING_071 reproduced: signed distribution does not contain 180 scenarios"
    );
}
