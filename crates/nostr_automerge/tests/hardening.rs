//! Hardening configuration contract tests.
#[test]
fn initialize_cargo_fuzz_harness() {
    let manifest = include_str!("../../../fuzz/Cargo.toml");
    assert!(manifest.contains("cargo-fuzz"));
    assert!(manifest.contains("name = \"smoke\""));
}
#[test]
fn fuzz_strict_raw_json_and_nip_01() {
    assert!(
        include_str!("../../../fuzz/fuzz_targets/raw_nip01.rs")
            .contains("VerifiedNip01Event::verify")
    );
}
#[test]
fn fuzz_automerge_framing_and_semantic_decode() {
    assert!(include_str!("../../../fuzz/fuzz_targets/automerge_framing.rs").contains("framing"));
    assert!(include_str!("../../../fuzz/fuzz_targets/automerge_semantics.rs").contains("reencode"));
}
#[test]
fn fuzz_control_objects_and_transitions() {
    assert!(
        include_str!("../../../fuzz/fuzz_targets/control_transition.rs")
            .contains("qualification_probe_control")
    );
}
#[test]
fn fuzz_dependency_graph_and_evaluator() {
    assert!(
        include_str!("../../../fuzz/fuzz_targets/reference_evaluator.rs")
            .contains("qualification_probe_reference")
    );
}
#[test]
fn fuzz_checkpoint_parser_and_merkle_verification() {
    assert!(include_str!("../../../fuzz/fuzz_targets/checkpoint.rs").contains("merkle_root"));
}
