use std::path::Path;
use std::process::Command;

use crate::requirements::{discover_fixture_metadata, generate_coverage_report};

const PYTHON_VALIDATORS: &[(&str, &str)] = &[
    ("remediation_authority", "scripts/validate_remediation.py"),
    (
        "remediation_v5_evidence",
        "scripts/validate_remediation_v5.py",
    ),
    (
        "remediation_v6_authority",
        "scripts/validate_remediation_v6.py",
    ),
    ("complete_specification", "scripts/validate_spec.py"),
    (
        "fixture_schema_checksum_snake_case",
        "scripts/validate_fixtures.py",
    ),
    (
        "fixture_distribution",
        "scripts/validate_fixture_distribution_v6.py",
    ),
    ("sealed_constants", "scripts/validate_protocol_revision.py"),
    ("automerge_boundary", "scripts/validate_architecture.py"),
    ("diagnostic_registry", "scripts/validate_diagnostics.py"),
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ValidationReport {
    pub(crate) validators: Vec<&'static str>,
    pub(crate) covered_requirements: usize,
    pub(crate) deferred_checkpoint_requirements: usize,
}

pub(crate) fn validate_repository(root: &Path) -> Result<ValidationReport, String> {
    let mut validators = Vec::new();
    for (name, script) in PYTHON_VALIDATORS {
        let output = Command::new("python3")
            .current_dir(root)
            .arg(script)
            .output()
            .map_err(|error| format!("{name}: {error}"))?;
        if !output.status.success() {
            return Err(format!(
                "{name}: {}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        validators.push(*name);
    }
    let fixture_paths = discover_fixture_metadata(&root.join("fixtures"))
        .map_err(|error| format!("requirement_coverage: {error:?}"))?;
    let coverage = generate_coverage_report(&root.join("spec/requirements.json"), fixture_paths)
        .map_err(|error| format!("requirement_coverage: {error:?}"))?;
    if !coverage.unknown.is_empty() {
        return Err(format!(
            "requirement_coverage: unknown IDs {:?}",
            coverage.unknown
        ));
    }
    validators.push("requirement_coverage");
    Ok(ValidationReport {
        validators,
        covered_requirements: coverage.covered.len(),
        deferred_checkpoint_requirements: coverage.deferred_checkpoint.len(),
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{PYTHON_VALIDATORS, validate_repository};

    #[test]
    fn add_repository_xtask_validation() {
        let names = PYTHON_VALIDATORS
            .iter()
            .map(|(name, _)| *name)
            .collect::<Vec<_>>();
        assert!(names.contains(&"fixture_schema_checksum_snake_case"));
        assert!(names.contains(&"fixture_distribution"));
        assert!(names.contains(&"sealed_constants"));
        assert!(names.contains(&"automerge_boundary"));
        assert!(names.contains(&"diagnostic_registry"));
        assert!(names.contains(&"remediation_authority"));
        assert!(names.contains(&"remediation_v5_evidence"));
        assert!(names.contains(&"remediation_v6_authority"));
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let report = validate_repository(&root);
        assert!(report.is_ok(), "{report:?}");
        let Ok(report) = report else { return };
        assert_eq!(report.validators.last(), Some(&"requirement_coverage"));
        assert!(report.covered_requirements > 0);
        assert!(report.deferred_checkpoint_requirements > 0);
    }
}
