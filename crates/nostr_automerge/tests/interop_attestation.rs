//! Validation for commit- and fixture-bound local interoperability inputs.

#[test]
fn generate_rust_profile_attestation_inputs() {
    for profile in ["core", "checkpoint"] {
        let path = format!("../../reports/interop_rust_{profile}.attestation.json");
        let bytes = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path));
        assert!(bytes.is_ok());
        let Ok(bytes) = bytes else { return };
        let report: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or_default();
        assert_eq!(report["schema"], "nostr_automerge.interop_attestation.v1");
        assert_eq!(
            report["implementation"]["repository"],
            "triesap/nostr_automerge"
        );
        assert_eq!(report["profile"], profile);
        assert_eq!(report["result"], "pass");
        assert_eq!(report["commit"].as_str().map(str::len), Some(40));
    }
}
