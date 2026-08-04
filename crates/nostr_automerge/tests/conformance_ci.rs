//! Structural validation for the deterministic conformance workflow.

use std::fs;
use std::path::Path;

#[test]
fn add_deterministic_conformance_ci() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let workflow = fs::read_to_string(root.join(".github/workflows/conformance.yml"));
    assert!(workflow.is_ok());
    let Ok(workflow) = workflow else { return };
    assert!(workflow.contains("cargo run -p nostr_automerge_xtask --locked -- validate"));
    assert_eq!(workflow.matches("run_corpus fixtures").count(), 2);
    assert!(workflow.contains("cmp core-profile-1.json core-profile-2.json"));
    assert!(workflow.contains("actions/upload-artifact@v4"));
    assert!(workflow.contains("if-no-files-found: error"));
    let contributing = fs::read_to_string(root.join("CONTRIBUTING.md"));
    assert!(contributing.is_ok());
    assert!(contributing.is_ok_and(|text| text.contains("local conformance-CI equivalent")));
}
