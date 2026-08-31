//! Expected-defect reproductions for the v15 operation-ownership follow-up.

const ACTOR_STATE: &str = include_str!("../src/graph/actor_state.rs");
const INVENTORY: &str =
    include_str!("../../../reports/causal_projection_consumer_inventory_v15.json");
const MUTATIONS: &str =
    include_str!("../../../scripts/run_causal_projection_behavior_mutations_v15.py");

fn projection_builder() -> &'static str {
    let Some((_, after_declaration)) =
        ACTOR_STATE.split_once("fn build_trusted_epoch_projection_observed")
    else {
        return "";
    };
    let Some((builder, _)) = after_declaration.split_once("#[cfg(test)]") else {
        return "";
    };
    builder
}

#[test]
fn f113_candidate_identity_comparison_is_owned() {
    assert!(projection_builder().contains("ProjectionBuildOperation::CandidateIdentityComparison"));
}

#[test]
fn f113_dependency_count_read_is_owned() {
    assert!(projection_builder().contains("ProjectionBuildOperation::DependencyCountRead"));
}

#[test]
fn f113_candidate_readiness_comparison_is_owned() {
    assert!(
        projection_builder().contains("ProjectionBuildOperation::CandidateReadinessComparison")
    );
}

#[test]
fn f113_candidate_kind_comparison_is_owned() {
    assert!(projection_builder().contains("ProjectionBuildOperation::CandidateKindComparison"));
}

#[test]
fn f113_remaining_state_write_is_owned() {
    assert!(projection_builder().contains("ProjectionBuildOperation::RemainingStateWrite"));
}

#[test]
fn f113_terminal_completion_comparison_is_owned() {
    assert!(projection_builder().contains("ProjectionBuildOperation::CompletionComparison"));
}

#[test]
fn f113_initial_count_read_and_comparison_are_separate() {
    let builder = projection_builder();
    assert!(builder.contains("ProjectionBuildOperation::SourceCountRead"));
    assert!(builder.contains("ProjectionBuildOperation::ExpectedCountComparison"));
}

#[test]
fn f114_active_rust_families_are_reachable() {
    assert!(!ACTOR_STATE.contains("SharedReferenceClone"));
    assert!(!ACTOR_STATE.contains("ConstantCandidateValidation"));
}

#[test]
fn f114_candidate_consumer_has_distinct_inventory_rows() {
    for operation in [
        "StoredCounterRead",
        "ExpectedStartComparison",
        "CheckedAdvance",
    ] {
        assert!(
            INVENTORY.contains(operation),
            "missing candidate operation {operation}"
        );
    }
}

#[test]
fn f115_mutations_change_resource_behavior() {
    for mutation in [
        "charge_deletion",
        "double_operation_after_one_charge",
        "state_write_before_charge",
        "post_stop_target_action",
        "early_publication",
    ] {
        assert!(
            MUTATIONS.contains(mutation),
            "missing behavioral mutation {mutation}"
        );
    }
}
