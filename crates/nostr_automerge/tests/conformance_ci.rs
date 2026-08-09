//! Structural validation for the local-only runner policy.

use std::fs;
use std::path::Path;

#[test]
fn require_local_only_conformance_runner() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let hosted_workflows = root.join(".github/workflows");
    assert!(
        !hosted_workflows.exists()
            || fs::read_dir(hosted_workflows).is_ok_and(|mut entries| entries.next().is_none())
    );
    assert!(!root.join(".act").exists());
    let ignore = fs::read_to_string(root.join(".gitignore"));
    assert!(ignore.is_ok_and(|text| !text.contains("/.act/")));
    let contributing = fs::read_to_string(root.join("CONTRIBUTING.md"));
    assert!(contributing.is_ok());
    assert!(contributing.is_ok_and(|text| text.contains("local conformance-CI equivalent")));
}
