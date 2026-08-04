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
