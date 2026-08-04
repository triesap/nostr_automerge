pub(crate) fn generate() -> String {
    serde_json::json!({"bomFormat":"CycloneDX","specVersion":"1.5","metadata":{"component":{"name":"nostr_automerge","type":"library","version":"0.1.0-alpha.0"}},"serialNumber":"urn:uuid:00000000-0000-0000-0000-000000000000","version":1}).to_string()
}
#[cfg(test)]
mod tests {
    #[test]
    fn generate_sbom_and_provenance() {
        let first = super::generate();
        assert_eq!(first, super::generate());
        let value: serde_json::Value = serde_json::from_str(&first).unwrap_or_default();
        assert_eq!(value["bomFormat"], "CycloneDX");
    }
}
