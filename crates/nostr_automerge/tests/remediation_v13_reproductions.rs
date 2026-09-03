//! Expected-failure reproductions for the causal-projection follow-up.

const ACTOR_STATE_SOURCE: &str = include_str!("../src/graph/actor_state.rs");

fn projection_builder_source() -> Option<&'static str> {
    ACTOR_STATE_SOURCE
        .split_once("fn build_trusted_epoch_projection_observed")
        .and_then(|(_, suffix)| suffix.split_once("#[cfg(test)]"))
        .map(|(body, _)| body)
}

#[test]
fn finding_104_projection_causal_maximum_has_no_final_state_scan() {
    let source = projection_builder_source();
    assert!(source.is_some(), "reviewed projection source boundary");
    let Some(source) = source else { return };
    assert!(!source.contains("states\n        .values()"));
    assert!(!source.contains(".map(|state| state.next_op)"));
}

#[test]
fn finding_108_projection_operations_use_one_closed_boundary() {
    let source = projection_builder_source();
    assert!(source.is_some(), "reviewed projection source boundary");
    let Some(source) = source else { return };
    assert!(source.contains("ProjectionBuildSite"));
    assert!(source.contains("perform_projection_build_operation"));
    assert!(!source.contains("while !ready.is_empty()"));
    assert!(!source.contains("charge(WorkCounter::"));
}
