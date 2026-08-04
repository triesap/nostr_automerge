//! Hardening configuration contract tests.
#[test]
fn initialize_cargo_fuzz_harness() {
    let manifest = include_str!("../../../fuzz/Cargo.toml");
    assert!(manifest.contains("cargo-fuzz"));
    assert!(manifest.contains("name = \"smoke\""));
}
