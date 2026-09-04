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
    (
        "private_reproduction_boundary_v9",
        "scripts/validate_private_reproduction_boundary_v9.py",
    ),
    (
        "resource_followup_authority_v10",
        "scripts/validate_resource_followup_authority_v10.py",
    ),
    (
        "runtime_ledger_v10",
        "scripts/validate_runtime_ledger_v10.py",
    ),
    (
        "resource_operation_inventory_v10",
        "scripts/validate_resource_operation_inventory_v10.py",
    ),
    (
        "appended_conformance_v11",
        "scripts/validate_appended_conformance_v11.py",
    ),
    (
        "resource_ancestry_gate_v10",
        "scripts/validate_resource_ancestry_gate_v10.py",
    ),
    (
        "resource_followup_assurance_v10",
        "scripts/validate_resource_followup_assurance_v10.py",
    ),
    (
        "resource_followup_final_decision_v10",
        "scripts/validate_resource_followup_final_decision_v10.py",
    ),
    ("distribution_v12", "scripts/validate_distribution_v12.py"),
    (
        "rust_conformance_v12",
        "scripts/validate_rust_conformance_v12.py",
    ),
    (
        "opaque_distribution_parity_v12",
        "scripts/validate_opaque_distribution_parity_v12.py",
    ),
    (
        "remediation_v11_authority_gate",
        "scripts/validate_remediation_v11_authority_gate.py",
    ),
    (
        "remediation_v11_proof_catalog",
        "scripts/validate_remediation_v11_proof_catalog.py",
    ),
    (
        "remediation_v11_adversarial_qualification",
        "scripts/validate_remediation_v11_adversarial_qualification.py",
    ),
    (
        "remediation_v11_local_assurance",
        "scripts/validate_remediation_v11_local_assurance.py",
    ),
    (
        "remediation_v11_finding_closure",
        "scripts/validate_remediation_v11_finding_closure.py",
    ),
    (
        "remediation_v11_final_decision",
        "scripts/validate_remediation_v11_final_decision.py",
    ),
    ("remediation_v11", "scripts/validate_remediation_v11.py"),
    (
        "persistent_state_v11",
        "scripts/validate_persistent_state_v11.py",
    ),
    (
        "persistent_state_core_gate_v11",
        "scripts/validate_persistent_state_core_gate_v11.py",
    ),
    (
        "persistent_state_integration_gate_v11",
        "scripts/validate_persistent_state_integration_gate_v11.py",
    ),
    (
        "target_work_accounting_v11",
        "scripts/validate_target_work_accounting_v11.py",
    ),
    (
        "persistent_ownership_v11",
        "scripts/validate_persistent_ownership_v11.py",
    ),
    (
        "unsupported_identity_contradiction_v11",
        "scripts/validate_unsupported_identity_contradiction_v11.py",
    ),
    ("remediation_v12", "scripts/validate_remediation_v12.py"),
    (
        "trusted_epoch_projection_gate_v12",
        "scripts/validate_trusted_epoch_projection_gate_v12.py",
    ),
    (
        "remediation_v12_actor_gate",
        "scripts/validate_remediation_v12_actor_gate.py",
    ),
    (
        "remediation_v12_ancestry_authorization_gate",
        "scripts/validate_remediation_v12_ancestry_authorization_gate.py",
    ),
    ("distribution_v13", "scripts/validate_distribution_v13.py"),
    (
        "rust_conformance_v13",
        "scripts/validate_rust_conformance_v13.py",
    ),
    (
        "remediation_v12_distribution_gate",
        "scripts/validate_remediation_v12_distribution_gate.py",
    ),
    (
        "distribution_v13_compatibility_contract",
        "scripts/validate_distribution_v13_compatibility_contract.py",
    ),
    (
        "distribution_v13_parity",
        "scripts/validate_distribution_v13_parity.py",
    ),
    (
        "remediation_v12_operation_inventory",
        "scripts/validate_remediation_v12_operation_inventory.py",
    ),
    (
        "remediation_v12_proof_catalog",
        "scripts/validate_remediation_v12_proof_catalog.py",
    ),
    (
        "remediation_v12_mutation_qualification",
        "scripts/validate_remediation_v12_mutation_qualification.py",
    ),
    (
        "remediation_v12_public_assurance",
        "scripts/validate_remediation_v12_public_assurance.py",
    ),
    (
        "remediation_v12_combined_assurance",
        "scripts/validate_remediation_v12_combined_assurance.py",
    ),
    (
        "remediation_v12_finding_closure",
        "scripts/validate_remediation_v12_finding_closure.py",
    ),
    (
        "remediation_v12_final_decision",
        "scripts/validate_remediation_v12_final_decision.py",
    ),
    ("remediation_v13", "scripts/validate_remediation_v13.py"),
    (
        "causal_projection_operations_v13",
        "scripts/validate_causal_projection_operations_v13.py",
    ),
    (
        "causal_projection_source_v13",
        "scripts/validate_causal_projection_source_v13.py",
    ),
    (
        "causal_projection_mutations_v13",
        "scripts/run_causal_projection_mutations_v13.py",
    ),
    (
        "causal_projection_authority_gate_v13",
        "scripts/validate_causal_projection_authority_gate_v13.py",
    ),
    (
        "causal_projection_implementation_gate_v13",
        "scripts/validate_causal_projection_implementation_gate_v13.py",
    ),
    ("distribution_v14", "scripts/validate_distribution_v14.py"),
    (
        "rust_conformance_v14",
        "scripts/validate_rust_conformance_v14.py",
    ),
    ("remediation_v15", "scripts/validate_remediation_v15.py"),
    ("remediation_v16", "scripts/validate_remediation_v16.py"),
    (
        "causal_projection_actor_reproductions_v16",
        "scripts/reproduce_remediation_v16.py",
    ),
    (
        "causal_projection_counter_oracle_reproductions_v16",
        "scripts/validate_causal_projection_counter_oracle_reproductions_v16.py",
    ),
    (
        "causal_projection_contracts_v16",
        "scripts/validate_causal_projection_contracts_v16.py",
    ),
    (
        "causal_projection_mutations_v16",
        "scripts/run_causal_projection_mutations_v16.py",
    ),
    (
        "opaque_causal_projection_v16",
        "scripts/validate_opaque_causal_projection_v16.py",
    ),
    (
        "causal_projection_combined_assurance_v16",
        "scripts/validate_causal_projection_combined_assurance_v16.py",
    ),
    (
        "causal_projection_final_decision_v16",
        "scripts/validate_causal_projection_final_decision_v16.py",
    ),
    ("remediation_v17", "scripts/validate_remediation_v17.py"),
    ("remediation_v18", "scripts/validate_remediation_v18.py"),
    (
        "causal_projection_contracts_v18",
        "scripts/validate_causal_projection_contracts_v18.py",
    ),
    (
        "causal_projection_contracts_v17",
        "scripts/validate_causal_projection_contracts_v17.py",
    ),
    (
        "causal_projection_properties_v17",
        "scripts/validate_causal_projection_properties_v17.py",
    ),
    (
        "causal_projection_inventory_v17",
        "scripts/validate_causal_projection_inventory_v17.py",
    ),
    (
        "causal_projection_proofs_v17",
        "scripts/validate_causal_projection_proofs_v17.py",
    ),
    (
        "causal_projection_structure_v17",
        "scripts/validate_causal_projection_structure_v17.py",
    ),
    (
        "causal_projection_identity_v17",
        "scripts/validate_causal_projection_identity_v17.py",
    ),
    (
        "causal_projection_mutations_v17",
        "scripts/run_causal_projection_mutations_v17.py",
    ),
    (
        "causal_projection_provenance_mutations_v17",
        "scripts/run_causal_projection_provenance_mutations_v17.py",
    ),
    (
        "causal_projection_mutation_coverage_v17",
        "scripts/finalize_causal_projection_mutations_v17.py",
    ),
    (
        "causal_projection_final_inventory_v17",
        "scripts/validate_causal_projection_final_inventory_v17.py",
    ),
    (
        "causal_projection_evidence_graph_v17",
        "scripts/validate_causal_projection_evidence_graph_v17.py",
    ),
    (
        "causal_projection_public_assurance_v17",
        "scripts/run_causal_projection_public_assurance_v17.py",
    ),
    (
        "distribution_v17_transition",
        "scripts/validate_distribution_v17_transition.py",
    ),
    (
        "rust_conformance_v17",
        "scripts/validate_rust_conformance_v17.py",
    ),
    (
        "opaque_causal_projection_v17",
        "scripts/validate_opaque_causal_projection_v17.py",
    ),
    (
        "causal_projection_combined_assurance_v17",
        "scripts/validate_causal_projection_combined_assurance_v17.py",
    ),
    (
        "causal_projection_finding_closure_v17",
        "scripts/validate_causal_projection_finding_closure_v17.py",
    ),
    (
        "causal_projection_completion_v17",
        "scripts/validate_causal_projection_completion_v17.py",
    ),
    (
        "causal_projection_final_decision_v17",
        "scripts/validate_causal_projection_final_decision_v17.py",
    ),
    (
        "causal_projection_clean_candidate_v17",
        "scripts/validate_causal_projection_clean_candidate_v17.py",
    ),
    (
        "causal_projection_operation_discovery_v15",
        "scripts/validate_causal_projection_operation_discovery_v15.py",
    ),
    (
        "causal_projection_discovery_v15",
        "scripts/validate_causal_projection_discovery_v15.py",
    ),
    (
        "causal_projection_proof_catalog_v15",
        "scripts/validate_causal_projection_proof_catalog_v15.py",
    ),
    (
        "causal_projection_source_ownership_v15",
        "scripts/validate_causal_projection_source_ownership_v15.py",
    ),
    (
        "causal_projection_behavior_mutations_v15",
        "scripts/run_causal_projection_behavior_mutations_v15.py",
    ),
    ("distribution_v15", "scripts/validate_distribution_v15.py"),
    (
        "rust_conformance_v15",
        "scripts/validate_rust_conformance_v15.py",
    ),
    ("distribution_v16", "scripts/validate_distribution_v16.py"),
    (
        "rust_conformance_v16",
        "scripts/validate_rust_conformance_v16.py",
    ),
    (
        "opaque_causal_projection_v15",
        "scripts/validate_opaque_causal_projection_v15.py",
    ),
    (
        "causal_projection_combined_assurance_v15",
        "scripts/validate_causal_projection_combined_assurance_v15.py",
    ),
    (
        "causal_projection_final_decision_v15",
        "scripts/validate_causal_projection_final_decision_v15.py",
    ),
    (
        "causal_projection_assurance_v13",
        "scripts/validate_causal_projection_assurance_v13.py",
    ),
    (
        "opaque_causal_projection_v14",
        "scripts/validate_opaque_causal_projection_v14.py",
    ),
    (
        "causal_projection_evidence_v14",
        "scripts/validate_causal_projection_evidence_v14.py",
    ),
    (
        "causal_projection_mutation_qualification_v14",
        "scripts/validate_causal_projection_mutation_qualification_v14.py",
    ),
    (
        "causal_projection_combined_assurance_v14",
        "scripts/validate_causal_projection_combined_assurance_v14.py",
    ),
    (
        "causal_projection_finding_closure_v14",
        "scripts/validate_causal_projection_finding_closure_v14.py",
    ),
    (
        "causal_projection_final_verification_v14",
        "scripts/validate_causal_projection_final_verification_v14.py",
    ),
    (
        "causal_projection_final_decision_v14",
        "scripts/validate_causal_projection_final_decision_v14.py",
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
    let followup_active = root
        .join("spec/resource_followup_authority_v10.json")
        .is_file();
    let v12_active = root.join("spec/remediation_v12_authority.json").is_file();
    let v13_active = root.join("spec/remediation_v13_authority.json").is_file();
    let v18_active = root.join("spec/remediation_v18_authority.json").is_file();
    for (name, script) in PYTHON_VALIDATORS {
        if followup_active && followup_historical_validator(name) {
            continue;
        }
        if v12_active && v12_historical_validator(name) {
            continue;
        }
        if v13_active && v13_historical_validator(name) {
            continue;
        }
        if v18_active && v18_historical_validator(name) {
            continue;
        }
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

fn v12_historical_validator(name: &str) -> bool {
    matches!(
        name,
        "resource_followup_authority_v10"
            | "runtime_ledger_v10"
            | "resource_operation_inventory_v10"
            | "appended_conformance_v11"
            | "resource_ancestry_gate_v10"
            | "resource_followup_assurance_v10"
            | "resource_followup_final_decision_v10"
            | "distribution_v12"
            | "rust_conformance_v12"
            | "opaque_distribution_parity_v12"
            | "remediation_v11_authority_gate"
            | "remediation_v11_proof_catalog"
            | "remediation_v11_adversarial_qualification"
            | "remediation_v11_local_assurance"
            | "remediation_v11_finding_closure"
            | "remediation_v11_final_decision"
            | "remediation_v11"
            | "persistent_state_v11"
            | "persistent_state_core_gate_v11"
            | "persistent_state_integration_gate_v11"
            | "target_work_accounting_v11"
            | "persistent_ownership_v11"
            | "unsupported_identity_contradiction_v11"
    )
}

fn followup_historical_validator(name: &str) -> bool {
    matches!(
        name,
        "authority_transition_v10"
            | "checkpoint_parity_v9"
            | "carrier_gate_v9"
            | "report_contract_v9"
            | "rust_report_gate_v9"
            | "rust_finalization_gate_v9"
            | "rust_resource_gate_v9"
            | "rust_conformance_v10"
            | "opaque_conformance_v10"
            | "signed_conformance_gate_v10"
            | "semantic_proof_catalog_v10"
            | "base64_proof_v10"
            | "rust_requirement_proofs_v10"
            | "report_finding_proofs_v10"
            | "opaque_semantic_proofs_v10"
            | "semantic_proof_mutations_v10"
            | "semantic_proof_catalog_final_v10"
            | "semantic_evidence_gate_v10"
            | "public_assurance_v10"
            | "opaque_private_assurance_v10"
            | "final_identity_v10"
            | "final_finding_closure_v10"
            | "final_decision_gate_v10"
            | "opaque_boundary_gate_v9"
            | "opaque_resource_gate_v9"
            | "opaque_finalization_v9"
            | "report_parity_v9"
            | "private_reproduction_boundary_v9"
            | "runtime_ledger_v10"
    )
}

fn v13_historical_validator(name: &str) -> bool {
    matches!(
        name,
        "resource_operation_inventory_v10"
            | "remediation_v12"
            | "trusted_epoch_projection_gate_v12"
            | "remediation_v12_actor_gate"
            | "remediation_v12_ancestry_authorization_gate"
            | "distribution_v13"
            | "rust_conformance_v13"
            | "remediation_v12_distribution_gate"
            | "distribution_v13_compatibility_contract"
            | "distribution_v13_parity"
            | "remediation_v12_operation_inventory"
            | "remediation_v12_proof_catalog"
            | "remediation_v12_mutation_qualification"
            | "remediation_v12_public_assurance"
            | "remediation_v12_combined_assurance"
            | "remediation_v12_finding_closure"
            | "remediation_v12_final_decision"
    )
}

fn v18_historical_validator(name: &str) -> bool {
    matches!(
        name,
        "causal_projection_contracts_v17"
            | "causal_projection_properties_v17"
            | "causal_projection_inventory_v17"
            | "causal_projection_proofs_v17"
            | "causal_projection_structure_v17"
            | "causal_projection_identity_v17"
            | "causal_projection_mutations_v17"
            | "causal_projection_provenance_mutations_v17"
            | "causal_projection_mutation_coverage_v17"
            | "causal_projection_final_inventory_v17"
            | "causal_projection_evidence_graph_v17"
            | "causal_projection_public_assurance_v17"
            | "distribution_v17_transition"
            | "rust_conformance_v17"
            | "opaque_causal_projection_v17"
            | "causal_projection_combined_assurance_v17"
            | "causal_projection_finding_closure_v17"
            | "causal_projection_completion_v17"
            | "causal_projection_final_decision_v17"
            | "causal_projection_clean_candidate_v17"
    )
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
        assert!(names.contains(&"private_reproduction_boundary_v9"));
        assert!(names.contains(&"resource_followup_authority_v10"));
        assert!(names.contains(&"runtime_ledger_v10"));
        assert!(names.contains(&"resource_operation_inventory_v10"));
        assert!(names.contains(&"appended_conformance_v11"));
        assert!(names.contains(&"rust_conformance_v12"));
        assert!(names.contains(&"resource_ancestry_gate_v10"));
        assert!(names.contains(&"resource_followup_assurance_v10"));
        assert!(names.contains(&"resource_followup_final_decision_v10"));
        assert!(names.contains(&"remediation_v11"));
        assert!(names.contains(&"remediation_v12"));
        assert!(names.contains(&"remediation_v13"));
        assert!(names.contains(&"remediation_v17"));
        assert!(names.contains(&"remediation_v18"));
        assert!(names.contains(&"causal_projection_contracts_v18"));
        assert!(names.contains(&"causal_projection_contracts_v17"));
        assert!(names.contains(&"causal_projection_properties_v17"));
        assert!(!names.contains(&"causal_projection_operation_inventory_v16"));
        assert!(!names.contains(&"causal_projection_proof_catalog_v16"));
        assert!(!names.contains(&"causal_projection_structural_assurance_v16"));
        assert!(!names.contains(&"causal_projection_rust_assurance_v16"));
        assert!(!names.contains(&"causal_projection_consumer_v15"));
        assert!(names.contains(&"causal_projection_operations_v13"));
        assert!(names.contains(&"trusted_epoch_projection_gate_v12"));
        assert!(names.contains(&"remediation_v12_actor_gate"));
        assert!(names.contains(&"remediation_v12_ancestry_authorization_gate"));
        assert!(names.contains(&"distribution_v13"));
        assert!(names.contains(&"rust_conformance_v13"));
        assert!(names.contains(&"remediation_v12_distribution_gate"));
        assert!(names.contains(&"distribution_v13_compatibility_contract"));
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
