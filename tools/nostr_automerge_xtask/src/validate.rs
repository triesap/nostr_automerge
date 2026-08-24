use std::path::Path;
use std::process::Command;

use crate::requirements::{discover_fixture_metadata, generate_coverage_report};

const PYTHON_VALIDATORS: &[(&str, &str)] = &[
    (
        "authority_transition_v10",
        "scripts/validate_authority_transition_v10.py",
    ),
    (
        "checkpoint_parity_v9",
        "scripts/validate_checkpoint_parity_v9.py",
    ),
    ("carrier_gate_v9", "scripts/validate_carrier_gate_v9.py"),
    (
        "report_contract_v9",
        "scripts/validate_report_contract_v9.py",
    ),
    (
        "rust_report_gate_v9",
        "scripts/validate_rust_report_gate_v9.py",
    ),
    (
        "rust_finalization_gate_v9",
        "scripts/validate_rust_finalization_gate_v9.py",
    ),
    (
        "rust_resource_gate_v9",
        "scripts/validate_rust_resource_gate_v9.py",
    ),
    (
        "rust_conformance_v10",
        "scripts/validate_rust_conformance_v10.py",
    ),
    (
        "opaque_conformance_v10",
        "scripts/validate_opaque_conformance_v10.py",
    ),
    (
        "signed_conformance_gate_v10",
        "scripts/validate_signed_conformance_gate_v10.py",
    ),
    (
        "semantic_proof_catalog_v10",
        "scripts/validate_semantic_proof_catalog_v10.py",
    ),
    ("base64_proof_v10", "scripts/validate_base64_proof_v10.py"),
    (
        "rust_requirement_proofs_v10",
        "scripts/validate_rust_requirement_proofs_v10.py",
    ),
    (
        "report_finding_proofs_v10",
        "scripts/validate_report_finding_proofs_v10.py",
    ),
    (
        "opaque_semantic_proofs_v10",
        "scripts/validate_opaque_semantic_proofs_v10.py",
    ),
    (
        "semantic_proof_mutations_v10",
        "scripts/validate_semantic_proof_mutations_v10.py",
    ),
    (
        "semantic_proof_catalog_final_v10",
        "scripts/validate_semantic_proof_catalog_final_v10.py",
    ),
    (
        "semantic_evidence_gate_v10",
        "scripts/validate_semantic_evidence_gate_v10.py",
    ),
    (
        "public_assurance_v10",
        "scripts/validate_public_assurance_v10.py",
    ),
    (
        "opaque_private_assurance_v10",
        "scripts/validate_opaque_private_assurance_v10.py",
    ),
    (
        "final_identity_v10",
        "scripts/validate_final_identity_v10.py",
    ),
    (
        "final_finding_closure_v10",
        "scripts/validate_final_finding_closure_v10.py",
    ),
    (
        "final_decision_gate_v10",
        "scripts/validate_final_decision_gate_v10.py",
    ),
    (
        "opaque_boundary_gate_v9",
        "scripts/validate_opaque_boundary_gate_v9.py",
    ),
    (
        "opaque_resource_gate_v9",
        "scripts/validate_opaque_resource_gate_v9.py",
    ),
    (
        "opaque_finalization_v9",
        "scripts/validate_opaque_finalization_v9.py",
    ),
    ("report_parity_v9", "scripts/validate_report_parity_v9.py"),
    ("runtime_ledger_v9", "scripts/validate_runtime_ledger_v9.py"),
    (
        "private_reproduction_boundary_v9",
        "scripts/validate_private_reproduction_boundary_v9.py",
    ),
    ("complete_specification", "scripts/validate_spec.py"),
    (
        "fixture_schema_checksum_snake_case",
        "scripts/validate_fixtures.py",
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
            .env("PYTHONDONTWRITEBYTECODE", "1")
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
        assert!(names.contains(&"authority_transition_v10"));
        assert!(names.contains(&"checkpoint_parity_v9"));
        assert!(names.contains(&"carrier_gate_v9"));
        assert!(names.contains(&"report_contract_v9"));
        assert!(names.contains(&"rust_report_gate_v9"));
        assert!(names.contains(&"rust_finalization_gate_v9"));
        assert!(names.contains(&"rust_resource_gate_v9"));
        assert!(names.contains(&"rust_conformance_v10"));
        assert!(names.contains(&"opaque_conformance_v10"));
        assert!(names.contains(&"signed_conformance_gate_v10"));
        assert!(names.contains(&"semantic_proof_catalog_v10"));
        assert!(names.contains(&"opaque_boundary_gate_v9"));
        assert!(names.contains(&"opaque_resource_gate_v9"));
        assert!(names.contains(&"opaque_finalization_v9"));
        assert!(names.contains(&"report_parity_v9"));
        assert!(names.contains(&"runtime_ledger_v9"));
        assert!(names.contains(&"private_reproduction_boundary_v9"));
        assert!(names.contains(&"complete_specification"));
        assert!(names.contains(&"sealed_constants"));
        assert!(names.contains(&"automerge_boundary"));
        assert!(names.contains(&"diagnostic_registry"));
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let complete_spec = std::fs::read_to_string(root.join("scripts/validate_spec.py"));
        assert!(
            complete_spec.is_ok(),
            "complete specification validator is readable: {complete_spec:?}"
        );
        let Ok(complete_spec) = complete_spec else {
            return;
        };
        for historical in [
            "validate_fixture_distribution_v9.py",
            "validate_remediation_v8.py",
            "validate_nip_reconciliation_v8.py",
            "validate_rust_conformance_v9.py",
            "validate_interop_attestation_v9.py",
            "validate_requirements_authority_v9.py",
            "validate_requirement_matrix_v9.py",
            "validate_resource_qualification_v9.py",
            "validate_assurance_v9.py",
            "validate_private_assurance_v9.py",
            "validate_final_identity_v8.py",
            "validate_local_gate_summary_v8.py",
            "validate_remediation_v8_final.py",
        ] {
            assert!(complete_spec.contains(historical));
        }
        let report = validate_repository(&root);
        assert!(report.is_ok(), "{report:?}");
        let Ok(report) = report else { return };
        assert_eq!(report.validators.last(), Some(&"requirement_coverage"));
        assert!(report.covered_requirements > 0);
        assert!(report.deferred_checkpoint_requirements > 0);
    }
}
