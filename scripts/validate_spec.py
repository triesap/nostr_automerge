#!/usr/bin/env python3
"""Run the complete deterministic specification baseline gate."""

import argparse
import hashlib
import json
import os
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
VALIDATORS = [
    "validate_adrs.py", "validate_automerge_qualification.py", "validate_companion_specs.py", "validate_diagnostics.py",
    "validate_dispositions_digest.py", "validate_draft_limits.py", "validate_fixtures.py", "validate_fixture_distribution_v9.py",
    "validate_governance.py", "validate_history_digest.py", "validate_import.py",
    "validate_nip_snapshot.py", "validate_remediation_v8.py", "validate_prior_art.py", "validate_protocol_revision.py",
    "validate_reports.py", "validate_repository_policy.py", "validate_requirements.py",
    "validate_runner_manifest.py", "validate_nip_reconciliation_v8.py",
    "validate_normative_clarifications_v3.py", "validate_rust_conformance_v9.py",
    "validate_interop_attestation_v9.py",
    "validate_requirements_authority_v9.py", "validate_requirement_matrix_v9.py",
    "validate_resource_qualification_v9.py", "validate_assurance_v9.py",
    "validate_private_assurance_v9.py", "validate_final_identity_v8.py",
    "validate_local_gate_summary_v8.py", "validate_remediation_v8_final.py",
    "validate_authority_transition_v10.py",
    "generate_distribution_v10.py",
    "validate_corrected_checkpoint_expectations_v10.py",
    "validate_rust_conformance_v10.py",
    "validate_opaque_conformance_v10.py",
    "validate_signed_conformance_gate_v10.py",
    "validate_semantic_proof_catalog_v10.py",
    "validate_base64_proof_v10.py",
    "validate_rust_requirement_proofs_v10.py",
    "validate_report_finding_proofs_v10.py",
    "validate_opaque_semantic_proofs_v10.py",
    "validate_semantic_proof_mutations_v10.py",
    "validate_semantic_proof_catalog_final_v10.py",
    "validate_semantic_evidence_gate_v10.py",
    "validate_public_assurance_v10.py",
    "validate_opaque_private_assurance_v10.py",
    "validate_final_identity_v10.py",
    "validate_final_finding_closure_v10.py",
    "validate_final_decision_gate_v10.py",
    "validate_remediation_v9.py",
    "validate_carrier_gate_v9.py",
    "validate_report_contract_v9.py",
    "validate_rust_report_gate_v9.py",
    "validate_rust_finalization_gate_v9.py",
    "validate_rust_resource_gate_v9.py",
    "validate_opaque_boundary_gate_v9.py",
    "validate_opaque_resource_gate_v9.py",
    "validate_opaque_finalization_v9.py",
    "validate_report_parity_v9.py",
    "validate_runtime_ledger_v9.py",
    "validate_private_reproduction_boundary_v9.py",
    "validate_resource_followup_authority_v10.py",
    "validate_runtime_ledger_v10.py",
    "validate_resource_operation_inventory_v10.py",
    "generate_distribution_v11.py",
    "validate_appended_conformance_v11.py",
    "validate_resource_ancestry_gate_v10.py",
    "validate_resource_followup_assurance_v10.py",
    "validate_resource_followup_final_decision_v10.py",
    "validate_remediation_v11.py",
    "validate_persistent_state_v11.py",
    "validate_persistent_state_core_gate_v11.py",
    "validate_persistent_state_integration_gate_v11.py",
    "validate_target_work_accounting_v11.py",
    "validate_persistent_ownership_v11.py",
    "validate_unsupported_identity_contradiction_v11.py",
    "generate_distribution_v12.py",
    "validate_distribution_v12.py",
    "validate_rust_conformance_v12.py",
    "validate_opaque_distribution_parity_v12.py",
    "validate_remediation_v11_authority_gate.py",
    "validate_remediation_v11_proof_catalog.py",
    "validate_remediation_v11_adversarial_qualification.py",
    "validate_remediation_v11_local_assurance.py",
    "validate_remediation_v11_finding_closure.py",
    "validate_remediation_v11_final_decision.py",
    "validate_remediation_v12.py",
    "validate_trusted_epoch_projection_gate_v12.py",
    "validate_remediation_v12_actor_gate.py",
    "validate_remediation_v12_ancestry_authorization_gate.py",
    "validate_distribution_v13.py",
    "validate_rust_conformance_v13.py",
    "validate_remediation_v12_distribution_gate.py",
    "validate_distribution_v13_compatibility_contract.py",
    "validate_distribution_v13_parity.py",
    "validate_remediation_v12_operation_inventory.py",
    "validate_remediation_v12_proof_catalog.py",
    "validate_remediation_v12_mutation_qualification.py",
    "validate_remediation_v12_public_assurance.py",
    "validate_remediation_v12_combined_assurance.py",
    "validate_remediation_v12_finding_closure.py",
    "validate_remediation_v12_final_decision.py",
    "validate_remediation_v13.py",
    "validate_causal_projection_operations_v13.py",
    "validate_causal_projection_source_v13.py",
    "run_causal_projection_mutations_v13.py",
    "validate_causal_projection_authority_gate_v13.py",
    "validate_causal_projection_implementation_gate_v13.py",
    "validate_distribution_v14.py",
    "validate_rust_conformance_v14.py",
    "validate_causal_projection_assurance_v13.py",
    "validate_opaque_causal_projection_v14.py",
    "validate_causal_projection_evidence_v14.py",
    "validate_causal_projection_mutation_qualification_v14.py",
    "validate_causal_projection_combined_assurance_v14.py",
    "validate_causal_projection_finding_closure_v14.py",
    "validate_causal_projection_final_verification_v14.py",
]
HISTORICAL_VALIDATORS = {
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
    "validate_runtime_ledger_v9.py",
}
FOLLOWUP_HISTORICAL_VALIDATORS = {
    "validate_resource_followup_authority_v10.py",
    "validate_authority_transition_v10.py",
    "validate_checkpoint_parity_v9.py",
    "validate_carrier_gate_v9.py",
    "validate_report_contract_v9.py",
    "validate_rust_report_gate_v9.py",
    "validate_rust_finalization_gate_v9.py",
    "validate_rust_resource_gate_v9.py",
    "validate_rust_conformance_v10.py",
    "validate_opaque_conformance_v10.py",
    "validate_signed_conformance_gate_v10.py",
    "validate_semantic_proof_catalog_v10.py",
    "validate_base64_proof_v10.py",
    "validate_rust_requirement_proofs_v10.py",
    "validate_report_finding_proofs_v10.py",
    "validate_opaque_semantic_proofs_v10.py",
    "validate_semantic_proof_mutations_v10.py",
    "validate_semantic_proof_catalog_final_v10.py",
    "validate_semantic_evidence_gate_v10.py",
    "validate_public_assurance_v10.py",
    "validate_opaque_private_assurance_v10.py",
    "validate_final_identity_v10.py",
    "validate_final_finding_closure_v10.py",
    "validate_final_decision_gate_v10.py",
    "validate_remediation_v9.py",
    "validate_checkpoint_parity_v9.py",
    "validate_opaque_boundary_gate_v9.py",
    "validate_opaque_resource_gate_v9.py",
    "validate_opaque_finalization_v9.py",
    "validate_report_parity_v9.py",
    "validate_runtime_ledger_v9.py",
    "validate_private_reproduction_boundary_v9.py",
    "validate_runtime_ledger_v10.py",
}
V12_HISTORICAL_VALIDATORS = {
    "generate_distribution_v11.py",
    "validate_appended_conformance_v11.py",
    "validate_resource_ancestry_gate_v10.py",
    "validate_resource_followup_assurance_v10.py",
    "validate_resource_followup_final_decision_v10.py",
    "validate_remediation_v11.py",
    "validate_persistent_state_v11.py",
    "validate_persistent_state_core_gate_v11.py",
    "validate_persistent_state_integration_gate_v11.py",
    "validate_target_work_accounting_v11.py",
    "validate_persistent_ownership_v11.py",
    "validate_unsupported_identity_contradiction_v11.py",
    "generate_distribution_v12.py",
    "validate_distribution_v12.py",
    "validate_rust_conformance_v12.py",
    "validate_opaque_distribution_parity_v12.py",
    "validate_remediation_v11_authority_gate.py",
    "validate_remediation_v11_proof_catalog.py",
    "validate_remediation_v11_adversarial_qualification.py",
    "validate_remediation_v11_local_assurance.py",
    "validate_remediation_v11_finding_closure.py",
    "validate_remediation_v11_final_decision.py",
}
V13_HISTORICAL_VALIDATORS = {
    "validate_resource_operation_inventory_v10.py",
    "validate_remediation_v12.py",
    "validate_trusted_epoch_projection_gate_v12.py",
    "validate_remediation_v12_actor_gate.py",
    "validate_remediation_v12_ancestry_authorization_gate.py",
    "validate_distribution_v13.py",
    "validate_rust_conformance_v13.py",
    "validate_remediation_v12_distribution_gate.py",
    "validate_distribution_v13_compatibility_contract.py",
    "validate_distribution_v13_parity.py",
    "validate_remediation_v12_operation_inventory.py",
    "validate_remediation_v12_proof_catalog.py",
    "validate_remediation_v12_mutation_qualification.py",
    "validate_remediation_v12_public_assurance.py",
    "validate_remediation_v12_combined_assurance.py",
    "validate_remediation_v12_finding_closure.py",
    "validate_remediation_v12_final_decision.py",
}


def transition_stage() -> str:
    transition = json.loads((ROOT / "spec/authority_transition_v10.json").read_text())
    stage = transition.get("current_stage")
    order = transition.get("stage_order")
    if not isinstance(stage, str) or not isinstance(order, list) or stage not in order:
        raise SystemExit("FAIL: invalid v10 authority transition stage")
    return stage


def active_validators(stage: str) -> list[str]:
    v12_active = (ROOT / "spec/remediation_v12_authority.json").is_file()
    v13_active = (ROOT / "spec/remediation_v13_authority.json").is_file()
    if (ROOT / "spec/resource_followup_authority_v10.json").is_file():
        return [
            validator
            for validator in VALIDATORS
            if validator not in HISTORICAL_VALIDATORS
            and validator not in FOLLOWUP_HISTORICAL_VALIDATORS
            and (not v12_active or validator not in V12_HISTORICAL_VALIDATORS)
            and (not v13_active or validator not in V13_HISTORICAL_VALIDATORS)
        ]
    if stage == "transition_installed":
        return VALIDATORS
    return [validator for validator in VALIDATORS if validator not in HISTORICAL_VALIDATORS]


def controlled_files() -> list[pathlib.Path]:
    roots = ("AGENTS.md", "README.md", "CONTRIBUTING.md", "SECURITY.md", "CODEOWNERS", "spec", "fixtures/README.md", "fixtures/examples", "fixtures/schema", "docs/provenance")
    files = []
    for name in roots:
        path = ROOT / name
        files.extend([path] if path.is_file() else (item for item in path.rglob("*") if item.is_file()))
    files.extend(ROOT / "scripts" / name for name in (*VALIDATORS, "validate_spec.py"))
    files.extend(
        ROOT / name
        for name in (
            "docs/execution/remediation_v9/ledger.md",
            "docs/execution/remediation_v10/ledger.md",
            "docs/execution/remediation_v11/ledger.md",
            "docs/execution/remediation_v12/baseline.md",
            "docs/execution/remediation_v12/ledger.md",
            "docs/execution/remediation_v13/baseline.md",
            "docs/execution/remediation_v13/ledger.md",
            "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v11.md",
            "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v12.md",
            "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v13.md",
            "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v10.md",
            "implementation/runtime_ledger_v9.json",
            "implementation/runtime_ledger_v10.json",
            "implementation/runtime_ledger_v11.json",
            "implementation/runtime_ledger_v12.json",
            "implementation/runtime_ledger_v13.json",
            "reports/carrier_gate_v9.json",
            "reports/checkpoint_parity_v9.json",
            "reports/opaque_carrier_v9.json",
            "reports/opaque_checkpoint_v9.json",
            "reports/opaque_reproduction_v9.json",
            "reports/rust_report_gate_v9.json",
            "reports/rust_finalization_gate_v9.json",
            "reports/rust_resource_gate_v9.json",
            "reports/rust_conformance_v10.json",
            "reports/opaque_conformance_v10.json",
            "reports/signed_conformance_gate_v10.json",
            "spec/semantic_proof_catalog_v10.json",
            "reports/semantic_proof_catalog_v10.json",
            "reports/finding_closure_catalog_v10.json",
            "reports/semantic_evidence_gate_v10.json",
            "reports/public_assurance_v10.json",
            "reports/opaque_private_assurance_v10.json",
            "reports/final_identity_v10.json",
            "reports/final_finding_closure_v10.json",
            "reports/final_decision_gate_v10.json",
            "reports/appended_conformance_v11.json",
            "reports/resource_ancestry_gate_v10.json",
            "reports/resource_followup_assurance_v10.json",
            "reports/resource_followup_finding_closure_v10.json",
            "reports/resource_followup_final_decision_v10.json",
            "reports/remediation_v11_authority_gate.json",
            "reports/rust_conformance_v12.json",
            "reports/opaque_distribution_parity_v12.json",
            "reports/remediation_v11_proof_catalog.json",
            "reports/remediation_v11_adversarial_qualification.json",
            "reports/remediation_v11_local_assurance.json",
            "reports/remediation_v11_finding_closure.json",
            "reports/remediation_v11_final_decision.json",
            "reports/remediation_v12_authority_gate.json",
            "reports/trusted_epoch_projection_gate_v12.json",
            "reports/remediation_v12_actor_gate.json",
            "reports/remediation_v12_ancestry_authorization_gate.json",
            "reports/rust_conformance_v13.json",
            "reports/remediation_v12_distribution_gate.json",
            "reports/distribution_v13_parity.json",
            "reports/opaque_compatibility_v13.json",
            "reports/remediation_v12_operation_inventory.json",
            "reports/remediation_v12_proof_catalog.json",
            "reports/remediation_v12_mutation_qualification.json",
            "reports/remediation_v12_public_assurance.json",
            "reports/opaque_private_assurance_v13.json",
            "reports/remediation_v12_combined_assurance.json",
            "reports/remediation_v12_finding_closure.json",
            "reports/remediation_v12_final_decision.json",
            "reports/causal_projection_mutations_v13.json",
            "reports/rust_conformance_v14.json",
            "reports/causal_projection_assurance_v13.json",
            "reports/opaque_causal_projection_v14.json",
            "reports/causal_projection_operation_inventory_v14.json",
            "reports/causal_projection_proof_catalog_v14.json",
            "reports/causal_projection_mutation_qualification_v14.json",
            "reports/causal_projection_combined_assurance_v14.json",
            "reports/causal_projection_finding_closure_v14.json",
            "reports/causal_projection_final_verification_v14.json",
            "fixtures/distribution/manifest_v12.json",
            "fixtures/distribution/manifest_v13.json",
            "fixtures/distribution/manifest_v13.lock.json",
            "fixtures/distribution/manifest_v14.json",
            "fixtures/distribution/manifest_v14.lock.json",
            "spec/distribution_v13_transition.json",
            "spec/distribution_v13_compatibility_contract.json",
            "spec/distribution_v14_transition.json",
            "reports/evidence_transition_v11.json",
            "reports/persistent_state_core_v11.json",
            "tools/validation/persistent_state_core_v11.schema.json",
            "reports/persistent_state_integration_v11.json",
            "tools/validation/persistent_state_integration_v11.schema.json",
            "reports/target_work_accounting_v11.json",
            "tools/validation/target_work_accounting_v11.schema.json",
            "reports/persistent_ownership_v11.json",
            "tools/validation/persistent_ownership_v11.schema.json",
            "tools/validation/runtime_ledger_v12.schema.json",
            "tools/validation/runtime_ledger_v13.schema.json",
            "tools/validation/remediation_v13_evidence_policy.schema.json",
            "tools/validation/causal_projection_operation_contract_v13.schema.json",
            "tools/validation/causal_projection_mutations_v13.schema.json",
            "tools/validation/remediation_v12_evidence_policy.schema.json",
            "tools/validation/remediation_v12_authority_gate.schema.json",
            "tools/validation/trusted_epoch_projection_gate_v12.schema.json",
            "tools/validation/remediation_v12_actor_gate.schema.json",
            "tools/validation/remediation_v12_ancestry_authorization_gate.schema.json",
            "tools/validation/distribution_v13.schema.json",
            "tools/validation/distribution_v14.schema.json",
            "tools/validation/distribution_v14_lock.schema.json",
            "tools/validation/rust_conformance_v14.schema.json",
            "tools/validation/causal_projection_assurance_v13.schema.json",
            "tools/validation/opaque_causal_projection_v14.schema.json",
            "tools/validation/causal_projection_operation_inventory_v14.schema.json",
            "tools/validation/causal_projection_proof_catalog_v14.schema.json",
            "tools/validation/causal_projection_mutation_qualification_v14.schema.json",
            "tools/validation/causal_projection_combined_assurance_v14.schema.json",
            "tools/validation/causal_projection_finding_closure_v14.schema.json",
            "tools/validation/causal_projection_final_verification_v14.schema.json",
            "tools/validation/rust_conformance_v13.schema.json",
            "tools/validation/remediation_v12_distribution_gate.schema.json",
            "tools/validation/distribution_v13_compatibility_contract.schema.json",
            "tools/validation/distribution_v13_parity.schema.json",
            "tools/validation/remediation_v12_operation_inventory.schema.json",
            "tools/validation/remediation_v12_proof_catalog.schema.json",
            "tools/validation/remediation_v12_mutation_qualification.schema.json",
            "tools/validation/remediation_v12_public_assurance.schema.json",
            "tools/validation/remediation_v12_combined_assurance.schema.json",
            "tools/validation/remediation_v12_finding_closure.schema.json",
            "tools/validation/remediation_v12_final_decision.schema.json",
            "scripts/generate_semantic_proof_catalog_final_v10.py",
            "scripts/generate_distribution_v14.py",
            "scripts/reproduce_remediation_v11.py",
            "scripts/reproduce_remediation_v12.py",
            "reports/opaque_boundary_gate_v9.json",
            "reports/opaque_resource_gate_v9.json",
            "reports/opaque_finalization_v9.json",
            "reports/report_parity_v9.json",
            "tools/validation/checkpoint_parity_v9.schema.json",
            "tools/validation/carrier_gate_v9.schema.json",
            "tools/validation/opaque_carrier_v9.schema.json",
            "tools/validation/opaque_checkpoint_v9.schema.json",
            "tools/validation/opaque_reproduction_v9.schema.json",
            "tools/validation/rust_report_gate_v9.schema.json",
            "tools/validation/rust_finalization_gate_v9.schema.json",
            "tools/validation/rust_resource_gate_v9.schema.json",
            "tools/validation/rust_conformance_v10.schema.json",
            "tools/validation/opaque_conformance_v10.schema.json",
            "tools/validation/signed_conformance_gate_v10.schema.json",
            "tools/validation/semantic_proof_catalog_v10.schema.json",
            "tools/validation/finding_closure_catalog_v10.schema.json",
            "tools/validation/semantic_evidence_gate_v10.schema.json",
            "tools/validation/public_assurance_v10.schema.json",
            "tools/validation/opaque_private_assurance_v10.schema.json",
            "tools/validation/final_identity_v10.schema.json",
            "tools/validation/final_finding_closure_v10.schema.json",
            "tools/validation/final_decision_gate_v10.schema.json",
            "tools/validation/opaque_boundary_gate_v9.schema.json",
            "tools/validation/opaque_resource_gate_v9.schema.json",
            "tools/validation/opaque_finalization_v9.schema.json",
            "tools/validation/report_parity_v9.schema.json",
            "tools/validation/runtime_ledger_v9.schema.json",
            "tools/validation/resource_followup_authority_v10.schema.json",
            "tools/validation/runtime_ledger_v10.schema.json",
            "tools/validation/resource_operation_inventory_v10.schema.json",
            "tools/validation/appended_conformance_v11.schema.json",
            "tools/validation/distribution_v11.schema.json",
            "tools/validation/distribution_v12.schema.json",
            "tools/validation/remediation_v11_authority_gate.schema.json",
            "tools/validation/rust_conformance_v12.schema.json",
            "tools/validation/opaque_distribution_parity_v12.schema.json",
            "tools/validation/remediation_v11_proof_catalog.schema.json",
            "tools/validation/remediation_v11_adversarial_qualification.schema.json",
            "tools/validation/remediation_v11_local_assurance.schema.json",
            "tools/validation/remediation_v11_finding_closure.schema.json",
            "tools/validation/remediation_v11_final_decision.schema.json",
            "tools/validation/resource_ancestry_gate_v10.schema.json",
            "tools/validation/resource_ancestry_proof_catalog_v10.schema.json",
            "tools/validation/resource_followup_assurance_v10.schema.json",
            "tools/validation/resource_followup_finding_closure_v10.schema.json",
            "tools/validation/resource_followup_final_decision_v10.schema.json",
            "spec/resource_ancestry_proof_catalog_v10.json",
        )
    )
    files.extend(path for path in (ROOT / "docs/adr").glob("adr_[0-9][0-9][0-9][0-9]_*.md"))
    files.append(ROOT / "docs/adr/README.md")
    files.extend(ROOT / "implementation" / name for name in ("COMMIT_SEQUENCE.md", "TYPESCRIPT_INTEROP_PLAN.md", "commit_sequence.json", "deviations/step_001.md"))
    return sorted(files, key=lambda item: item.relative_to(ROOT).as_posix().encode())


def report(stage: str, validators: list[str]) -> str:
    aggregate = hashlib.sha256()
    for path in controlled_files():
        relative = path.relative_to(ROOT).as_posix().encode()
        data = path.read_bytes()
        aggregate.update(len(relative).to_bytes(4, "big") + relative)
        aggregate.update(len(data).to_bytes(8, "big") + data)
    requirements = json.loads((ROOT / "spec/requirements.json").read_text())["requirements"]
    return "\n".join((
        "nostr_automerge specification baseline",
        "revision: draft_2026_08",
        "status: approved_implementation_baseline",
        f"controlled_files: {len(controlled_files())}",
        f"requirements: {len(requirements)}",
        f"transition_stage: {stage}",
        f"historical_v9_validation: {'direct' if stage == 'transition_installed' else 'immutable_wrapper'}",
        f"validators: {len(validators)}",
        f"aggregate_sha256: {aggregate.hexdigest()}",
        "result: pass",
        "",
    ))


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--print-report", action="store_true")
    args = parser.parse_args()
    stage = transition_stage()
    validators = active_validators(stage)
    validator_env = dict(os.environ)
    validator_env["PYTHONDONTWRITEBYTECODE"] = "1"
    for validator in validators:
        result = subprocess.run(
            [sys.executable, str(ROOT / "scripts" / validator)],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=False,
            env=validator_env,
        )
        if result.returncode:
            sys.stderr.write(result.stdout + result.stderr)
            raise SystemExit(f"FAIL: {validator}")
    current = report(stage, validators)
    if args.print_report:
        print(current, end="")
        return
    expected = (ROOT / "reports/spec_baseline.txt").read_text()
    if current != expected:
        raise SystemExit("FAIL: stale specification baseline report")
    print("PASS: complete specification baseline")
    print(f"- validators={len(validators)}")
    print(f"- historical_v9={'direct' if stage == 'transition_installed' else 'immutable_wrapper'}")
    print(f"- report_sha256={hashlib.sha256(current.encode()).hexdigest()}")


if __name__ == "__main__":
    main()
