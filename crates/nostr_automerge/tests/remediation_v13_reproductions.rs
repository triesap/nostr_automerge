//! Expected-failure reproductions for the causal-projection follow-up.

const ACTOR_STATE_SOURCE: &str = include_str!("../src/graph/actor_state.rs");

fn projection_builder_source() -> &'static str {
    ACTOR_STATE_SOURCE
        .split_once("fn build_trusted_epoch_projection_observed")
        .expect("reviewed source contains projection builder")
        .1
        .split_once("#[cfg(test)]")
        .expect("reviewed source contains test boundary")
        .0
}

#[test]
#[ignore = "FINDING_104 remains open until step_1430"]
fn finding_104_projection_causal_maximum_has_no_final_state_scan() {
    let source = projection_builder_source();
    assert!(!source.contains("states\n        .values()"));
    assert!(!source.contains(".map(|state| state.next_op)"));
}

#[test]
#[ignore = "FINDING_108 remains open until RCLD117"]
fn finding_108_projection_operations_use_one_closed_boundary() {
    let source = projection_builder_source();
    assert!(source.contains("ProjectionBuildOperation"));
    assert!(source.contains("perform_projection_build_operation"));
    assert!(!source.contains("while !ready.is_empty()"));
}
