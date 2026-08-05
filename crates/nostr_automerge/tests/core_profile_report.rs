//! Verification for the checked-in Rust core-profile evidence.

use std::fs;
use std::path::Path;

use serde_json::Value;
use sha2::{Digest, Sha256};

fn sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(64);
    for byte in digest {
        use core::fmt::Write;
        let result = write!(&mut encoded, "{byte:02x}");
        assert!(result.is_ok());
    }
    encoded
}

#[test]
fn publish_the_rust_core_profile_conformance_report() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let report_bytes = fs::read(root.join("reports/core_profile_conformance.json"));
    let narrative = fs::read_to_string(root.join("reports/core_profile_conformance.md"));
    assert!(report_bytes.is_ok() && narrative.is_ok());
    let (Ok(report_bytes), Ok(narrative)) = (report_bytes, narrative) else {
        return;
    };
    let report: Result<Value, _> = serde_json::from_slice(&report_bytes);
    assert!(report.is_ok());
    let Ok(report) = report else { return };
    assert_eq!(
        report["schema"],
        "nostr_automerge.core_profile_conformance.v1"
    );
    assert_eq!(report["result"], "passed");
    assert_eq!(report["evaluated_commit"].as_str().map(str::len), Some(40));
    assert_eq!(
        report["unimplemented_checkpoint_requirements"]
            .as_array()
            .map(Vec::len),
        Some(12)
    );
    assert!(
        report["gates"]
            .as_object()
            .is_some_and(|gates| { gates.values().all(|result| result == "passed") })
    );
    let report_sha = sha256(&report_bytes);
    assert_eq!(
        report_sha,
        "96ca5184433e85ee59741740e8e6fd14e6c2dfed85828f0effdaa76a554e3158"
    );
    assert!(narrative.contains(&report_sha));

    let fixture = &report["fixture_manifest"][0];
    let fixture_root = root.join("fixtures/examples");
    let files = [
        ("actor_derivation_001.fixture.json", "metadata_sha256"),
        ("actor_derivation_001.input.json", "input_sha256"),
        ("actor_derivation_001.expected.json", "expected_sha256"),
    ];
    for (name, field) in files {
        let bytes = fs::read(fixture_root.join(name));
        assert!(bytes.is_ok());
        let Ok(bytes) = bytes else { return };
        assert_eq!(Some(sha256(&bytes).as_str()), fixture[field].as_str());
    }
    let lock = fs::read(root.join("Cargo.lock"));
    assert!(lock.is_ok());
    let Ok(lock) = lock else { return };
    assert_eq!(
        Some(sha256(&lock).as_str()),
        report["dependencies"]["cargo_lock_sha256"].as_str()
    );
}
