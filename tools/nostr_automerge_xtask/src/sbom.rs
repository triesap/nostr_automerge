use serde_json::{Value, json};

fn quoted_value(line: &str, key: &str) -> Option<String> {
    line.strip_prefix(key)
        .and_then(|value| value.strip_prefix('"'))
        .and_then(|value| value.strip_suffix('"'))
        .map(str::to_owned)
}

fn locked_components() -> Vec<Value> {
    let mut components = Vec::new();
    let mut name = None;
    let mut version = None;
    let mut registry_package = false;

    let mut finish = |name: &mut Option<String>, version: &mut Option<String>, registry: bool| {
        if registry {
            if let (Some(name), Some(version)) = (name.take(), version.take()) {
                let reference = format!("pkg:cargo/{name}@{version}");
                components.push(json!({
                    "bom-ref": reference,
                    "name": name,
                    "purl": reference,
                    "type": "library",
                    "version": version,
                }));
            }
        } else {
            *name = None;
            *version = None;
        }
    };

    for line in include_str!("../../../Cargo.lock").lines() {
        if line == "[[package]]" {
            finish(&mut name, &mut version, registry_package);
            registry_package = false;
        } else if let Some(value) = quoted_value(line, "name = ") {
            name = Some(value);
        } else if let Some(value) = quoted_value(line, "version = ") {
            version = Some(value);
        } else if line.starts_with("source = \"registry+") {
            registry_package = true;
        }
    }
    finish(&mut name, &mut version, registry_package);
    components
        .sort_by_key(|component| component["bom-ref"].as_str().unwrap_or_default().to_owned());
    components
}

pub(crate) fn generate() -> String {
    let root = "pkg:cargo/nostr_automerge@0.1.0-alpha.0";
    json!({
        "bomFormat": "CycloneDX",
        "components": locked_components(),
        "metadata": {
            "component": {
                "bom-ref": root,
                "name": "nostr_automerge",
                "purl": root,
                "type": "library",
                "version": "0.1.0-alpha.0"
            }
        },
        "serialNumber": "urn:uuid:00000000-0000-0000-0000-000000000000",
        "specVersion": "1.5",
        "version": 1
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    #[test]
    fn generate_sbom_and_provenance() {
        let first = super::generate();
        assert_eq!(first, super::generate());
        let value: serde_json::Value = serde_json::from_str(&first).unwrap_or_default();
        assert_eq!(value["bomFormat"], "CycloneDX");
        let components = value["components"].as_array().cloned().unwrap_or_default();
        assert!(components.len() > 70);
        assert!(
            components
                .iter()
                .any(|component| { component["purl"] == "pkg:cargo/automerge@0.10.0" })
        );
    }
}
