//! Ignored expected-failure reproductions for the remediation-v7 source review.
//!
//! The repository-owned reproduction harness runs these tests individually and
//! requires each one to fail at the bound baseline. A closing source checkpoint
//! removes `ignore` only after the corresponding assertion becomes green.

#[test]
fn finding_059_noncanonical_control_requires_branch_table() {
    let source = include_str!("../src/engine/reference_evaluator.rs");
    assert!(
        !source.contains("(*disposition == ProtocolDisposition::Excluded).then_some(*event_id)"),
        "FINDING_059 reproduced: preliminary exclusion still implies stateful validity"
    );
}

#[test]
fn finding_060_checkpoint_index_is_coordinate_qualified() {
    let source = include_str!("../src/evidence/indexes.rs");
    assert!(
        source.contains("chunks_by_coordinate_descriptor"),
        "FINDING_060 reproduced: checkpoint chunks lack a coordinate-plus-descriptor index"
    );
}

#[test]
fn finding_061_change_indexes_are_coordinate_qualified() {
    let source = include_str!("../src/evidence/indexes.rs");
    assert!(
        source.contains("hashes_by_coordinate_control")
            && source.contains("carriers_by_coordinate_hash"),
        "FINDING_061 reproduced: change discovery lacks coordinate-qualified indexes"
    );
}

#[test]
#[ignore = "remediation-v7 baseline reproduction for FINDING_062"]
fn finding_062_parent_propagation_is_linear_and_metered() {
    let source = include_str!("../src/reference/evaluate.rs");
    let body = source
        .find("pub(crate) fn propagate_control_parent_dispositions(")
        .map(|start| &source[start..source.len().min(start + 2_400)])
        .unwrap_or("");
    assert!(
        body.contains("budget: &mut WorkBudget")
            && body.contains("cancellation: &impl CancellationCheck")
            && !body.contains("for _ in 0..parents.len()"),
        "FINDING_062 reproduced: propagation remains repeated, unmetered, or uncancellable"
    );
}

#[test]
#[ignore = "remediation-v7 baseline reproduction for FINDING_063"]
fn finding_063_interrupted_settlement_is_explicit() {
    let source = include_str!("../src/engine/reference_evaluator.rs");
    assert!(
        !source.contains("consume_interrupted_omissions") && source.contains("forfeited"),
        "FINDING_063 reproduced: interrupted finalization still erases remainder as consumption"
    );
}
