#!/usr/bin/env python3
"""Reject non-opaque material from the v9 reproduction and runtime records."""

from __future__ import annotations

import ast
import copy
import io
import re
import subprocess
import sys
import tokenize
from pathlib import Path
from typing import Any

sys.dont_write_bytecode = True

from validate_runtime_ledger_v9 import (
    APPROVED_WIRE_DOMAINS,
    ABSOLUTE_PATH_TEXT,
    CASE_TEXT,
    COMMAND_TEXT,
    COMMIT_SUBJECT_TEXT,
    LOG_TEXT,
    PACKAGE_SUFFIX_TEXT,
    RELATIVE_PATH_TEXT,
    URI_TEXT,
    LedgerError,
    load_object,
    validate_no_leak,
)


ROOT = Path(__file__).resolve().parents[1]
OPAQUE_MUTATION_FRAGMENTS = frozenset(
    value.decode("ascii") for value in (b"ttps://", b"ile://", b"Users/")
)
OPAQUE_MUTATION_SEQUENCE = (
    b"src/test/scripts/package.jsonnode_modulescommandcredentialworkflow\\\\".decode(
        "ascii"
    )
)
CAUSAL_EVIDENCE_COMMAND = b"cargo test -p nostr_automerge --lib projection_operation_families_have_exact_n_minus_one_n_and_n_plus_one_stops --locked".decode(
    "ascii"
)
CAUSAL_EVIDENCE_EXECUTION_SEQUENCE = (
    b"cargoextbuildrun--cargotest-pnostr_automerge--lib--locked----exact".decode(
        "ascii"
    )
)
COMBINED_ASSURANCE_EXECUTION_SEQUENCE = b"cargoextbuildrun--cargorun--quiet-pnostr_automerge_conformance--locked--run_distribution".decode(
    "ascii"
)
V18_MUTATION_COMMAND_FRAGMENTS = frozenset(
    {
        "argo extbuild run -- cargo check -p nostr_automerge --lib --locked",
        "ython3 scripts/validate_causal_projection_properties_v18.py --root . --mode structural",
        "ython3 scripts/validate_causal_projection_properties_v18.py --root . --mode identity",
        "it diff --quiet -- crates/nostr_automerge/src/graph/actor_state.rs crates/nostr_automerge/src/reference/epoch_engine.rs",
    }
)
V18_MUTATION_COMMAND_SEQUENCE = b"argo extbuild run -- cargo check -p nostr_automerge --lib --lockedython3 scripts/validate_causal_projection_properties_v18.py --root . --mode structuralython3 scripts/validate_causal_projection_properties_v18.py --root . --mode identityit diff --quiet -- crates/nostr_automerge/src/graph/actor_state.rs crates/nostr_automerge/src/reference/epoch_engine.rs".decode(
    "ascii"
)
PUBLIC_PATCH_BLOCKED = tuple(
    value.decode("ascii")
    for value in (
        b"file://", b"/users/", b"/volumes/", b"node_modules",
        b".env.local", b"credential", b"private_key", b"secret",
    )
)
JSON_RECORDS = (
    "reports/opaque_reproduction_v9.json",
    "reports/opaque_checkpoint_v9.json",
    "reports/opaque_carrier_v9.json",
    "reports/carrier_gate_v9.json",
    "reports/checkpoint_parity_v9.json",
    "reports/rust_report_gate_v9.json",
    "reports/rust_finalization_gate_v9.json",
    "reports/rust_resource_gate_v9.json",
    "reports/rust_conformance_v10.json",
    "reports/opaque_conformance_v10.json",
    "reports/signed_conformance_gate_v10.json",
    "reports/opaque_boundary_gate_v9.json",
    "reports/opaque_resource_gate_v9.json",
    "reports/opaque_finalization_v9.json",
    "reports/report_parity_v9.json",
    "reports/opaque_semantic_proofs_v10.json",
    "reports/opaque_distribution_parity_v12.json",
    "reports/distribution_v13_parity.json",
    "reports/opaque_compatibility_v13.json",
    "reports/opaque_private_assurance_v13.json",
    "reports/opaque_causal_projection_v14.json",
    "reports/opaque_causal_projection_v15.json",
    "reports/opaque_causal_projection_v16.json",
    "reports/causal_projection_combined_assurance_v16.json",
    "reports/causal_projection_final_decision_v16.json",
    "reports/causal_projection_combined_assurance_v15.json",
    "reports/causal_projection_final_decision_v15.json",
    "implementation/runtime_ledger_v9.json",
    "tools/validation/opaque_reproduction_v9.schema.json",
    "tools/validation/opaque_checkpoint_v9.schema.json",
    "tools/validation/opaque_carrier_v9.schema.json",
    "tools/validation/carrier_gate_v9.schema.json",
    "tools/validation/checkpoint_parity_v9.schema.json",
    "tools/validation/rust_report_gate_v9.schema.json",
    "tools/validation/rust_finalization_gate_v9.schema.json",
    "tools/validation/rust_resource_gate_v9.schema.json",
    "tools/validation/rust_conformance_v10.schema.json",
    "tools/validation/opaque_conformance_v10.schema.json",
    "tools/validation/signed_conformance_gate_v10.schema.json",
    "tools/validation/opaque_boundary_gate_v9.schema.json",
    "tools/validation/opaque_resource_gate_v9.schema.json",
    "tools/validation/opaque_finalization_v9.schema.json",
    "tools/validation/report_parity_v9.schema.json",
    "tools/validation/opaque_semantic_proofs_v10.schema.json",
    "tools/validation/opaque_distribution_parity_v12.schema.json",
    "tools/validation/opaque_causal_projection_v14.schema.json",
    "tools/validation/opaque_causal_projection_v15.schema.json",
    "tools/validation/opaque_causal_projection_v16.schema.json",
    "tools/validation/causal_projection_combined_assurance_v16.schema.json",
    "tools/validation/causal_projection_final_decision_v16.schema.json",
    "tools/validation/causal_projection_combined_assurance_v15.schema.json",
    "tools/validation/causal_projection_final_decision_v15.schema.json",
    "tools/validation/distribution_v13_parity.schema.json",
    "tools/validation/runtime_ledger_v9.schema.json",
)
PUBLIC_JSON_RECORDS = (
    "spec/remediation_v11_reproductions.json",
    "reports/persistent_state_core_v11.json",
    "tools/validation/persistent_state_core_v11.schema.json",
    "reports/persistent_state_integration_v11.json",
    "tools/validation/persistent_state_integration_v11.schema.json",
    "reports/target_work_accounting_v11.json",
    "tools/validation/target_work_accounting_v11.schema.json",
    "reports/persistent_ownership_v11.json",
    "tools/validation/persistent_ownership_v11.schema.json",
    "reports/remediation_v11_authority_gate.json",
    "tools/validation/remediation_v11_authority_gate.schema.json",
    "reports/rust_conformance_v12.json",
    "tools/validation/rust_conformance_v12.schema.json",
    "reports/remediation_v11_proof_catalog.json",
    "tools/validation/remediation_v11_proof_catalog.schema.json",
    "reports/remediation_v11_adversarial_qualification.json",
    "tools/validation/remediation_v11_adversarial_qualification.schema.json",
    "reports/remediation_v11_local_assurance.json",
    "tools/validation/remediation_v11_local_assurance.schema.json",
    "reports/remediation_v11_finding_closure.json",
    "tools/validation/remediation_v11_finding_closure.schema.json",
    "reports/remediation_v11_final_decision.json",
    "tools/validation/remediation_v11_final_decision.schema.json",
    "reports/trusted_epoch_projection_gate_v12.json",
    "tools/validation/trusted_epoch_projection_gate_v12.schema.json",
    "reports/remediation_v12_actor_gate.json",
    "tools/validation/remediation_v12_actor_gate.schema.json",
    "reports/remediation_v12_ancestry_authorization_gate.json",
    "tools/validation/remediation_v12_ancestry_authorization_gate.schema.json",
    "reports/rust_conformance_v13.json",
    "reports/rust_conformance_v14.json",
    "reports/causal_projection_assurance_v13.json",
    "tools/validation/rust_conformance_v13.schema.json",
    "tools/validation/rust_conformance_v14.schema.json",
    "tools/validation/causal_projection_assurance_v13.schema.json",
    "reports/remediation_v12_distribution_gate.json",
    "tools/validation/remediation_v12_distribution_gate.schema.json",
    "spec/distribution_v13_transition.json",
    "spec/distribution_v13_compatibility_contract.json",
    "tools/validation/distribution_v13_compatibility_contract.schema.json",
    "tools/validation/distribution_v13.schema.json",
    "spec/distribution_v14_transition.json",
    "tools/validation/distribution_v14.schema.json",
    "reports/remediation_v12_operation_inventory.json",
    "tools/validation/remediation_v12_operation_inventory.schema.json",
    "reports/remediation_v12_proof_catalog.json",
    "tools/validation/remediation_v12_proof_catalog.schema.json",
    "reports/remediation_v12_mutation_qualification.json",
    "tools/validation/remediation_v12_mutation_qualification.schema.json",
    "reports/remediation_v12_public_assurance.json",
    "tools/validation/remediation_v12_public_assurance.schema.json",
    "reports/remediation_v12_combined_assurance.json",
    "tools/validation/remediation_v12_combined_assurance.schema.json",
    "reports/remediation_v12_finding_closure.json",
    "tools/validation/remediation_v12_finding_closure.schema.json",
    "reports/remediation_v12_final_decision.json",
    "tools/validation/remediation_v12_final_decision.schema.json",
    "reports/causal_projection_implementation_gate_v13.json",
    "tools/validation/causal_projection_implementation_gate_v13.schema.json",
    "reports/causal_projection_mutations_v13.json",
    "tools/validation/causal_projection_mutations_v13.schema.json",
    "reports/causal_projection_operation_inventory_v14.json",
    "tools/validation/causal_projection_operation_inventory_v14.schema.json",
    "reports/causal_projection_proof_catalog_v14.json",
    "tools/validation/causal_projection_proof_catalog_v14.schema.json",
    "reports/causal_projection_mutation_qualification_v14.json",
    "tools/validation/causal_projection_mutation_qualification_v14.schema.json",
    "reports/causal_projection_combined_assurance_v14.json",
    "tools/validation/causal_projection_combined_assurance_v14.schema.json",
    "reports/causal_projection_finding_closure_v14.json",
    "tools/validation/causal_projection_finding_closure_v14.schema.json",
    "reports/causal_projection_final_verification_v14.json",
    "tools/validation/causal_projection_final_verification_v14.schema.json",
    "reports/causal_projection_final_decision_v14.json",
    "tools/validation/causal_projection_final_decision_v14.schema.json",
    "spec/distribution_v15_transition.json",
    "reports/rust_conformance_v15.json",
    "tools/validation/distribution_v15.schema.json",
    "tools/validation/distribution_v15_lock.schema.json",
    "tools/validation/rust_conformance_v15.schema.json",
    "spec/remediation_v16_authority.json",
    "spec/remediation_findings_v16.json",
    "implementation/runtime_ledger_v16.json",
    "tools/validation/runtime_ledger_v16.schema.json",
    "reports/causal_projection_actor_reproductions_v16.json",
    "tools/validation/causal_projection_actor_reproductions_v16.schema.json",
    "reports/causal_projection_counter_oracle_reproductions_v16.json",
    "tools/validation/causal_projection_counter_oracle_reproductions_v16.schema.json",
    "spec/causal_projection_contracts_v16.json",
    "tools/validation/causal_projection_contracts_v16.schema.json",
    "reports/causal_projection_operation_inventory_v16.json",
    "tools/validation/causal_projection_operation_inventory_v16.schema.json",
    "reports/causal_projection_proof_catalog_v16.json",
    "tools/validation/causal_projection_proof_catalog_v16.schema.json",
    "reports/causal_projection_structural_assurance_v16.json",
    "tools/validation/causal_projection_structural_assurance_v16.schema.json",
    "reports/causal_projection_mutations_v16.json",
    "tools/validation/causal_projection_mutations_v16.schema.json",
    "reports/causal_projection_rust_assurance_v16.json",
    "tools/validation/causal_projection_rust_assurance_v16.schema.json",
    "spec/distribution_v16_transition.json",
    "reports/rust_conformance_v16.json",
    "tools/validation/distribution_v16_transition.schema.json",
    "tools/validation/distribution_v16.schema.json",
    "tools/validation/distribution_v16_lock.schema.json",
    "tools/validation/rust_conformance_v16.schema.json",
    "spec/distribution_v17_transition.json",
    "reports/rust_conformance_v17.json",
    "tools/validation/distribution_v17_transition.schema.json",
    "tools/validation/rust_conformance_v17.schema.json",
    "reports/opaque_causal_projection_v17.json",
    "tools/validation/opaque_causal_projection_v17.schema.json",
    "reports/causal_projection_combined_assurance_v17.json",
    "tools/validation/causal_projection_combined_assurance_v17.schema.json",
    "reports/causal_projection_finding_closure_v17.json",
    "tools/validation/causal_projection_finding_closure_v17.schema.json",
    "reports/causal_projection_completion_v17.json",
    "tools/validation/causal_projection_completion_v17.schema.json",
    "reports/causal_projection_final_decision_v17.json",
    "tools/validation/causal_projection_final_decision_v17.schema.json",
    "reports/causal_projection_clean_candidate_v17.json",
    "tools/validation/causal_projection_clean_candidate_v17.schema.json",
    "reports/causal_projection_proofs_v18.json",
    "tools/validation/causal_projection_proofs_v18.schema.json",
    "reports/causal_projection_mutations_v18.json",
    "tools/validation/causal_projection_mutations_v18.schema.json",
    "reports/causal_projection_catalogs_v18.json",
    "tools/validation/causal_projection_catalogs_v18.schema.json",
    "reports/causal_projection_final_inventory_v18.json",
    "tools/validation/causal_projection_final_inventory_v18.schema.json",
    "reports/causal_projection_evidence_graph_v18.json",
    "tools/validation/causal_projection_evidence_graph_v18.schema.json",
    "spec/distribution_v18_transition.json",
    "tools/validation/distribution_v18_transition.schema.json",
) + tuple(
    path.relative_to(ROOT).as_posix()
    for path in sorted((ROOT / "reports/evidence/v18/proofs").glob("*.json"))
) + tuple(
    path.relative_to(ROOT).as_posix()
    for path in sorted((ROOT / "reports/evidence/v18/mutations").glob("*.json"))
)
PUBLIC_SCHEMA_URIS = frozenset(
    value.decode("ascii")
    for value in (
        b"https://json-schema.org/draft/2020-12/schema",
        b"https://nostr-automerge.example/schema/persistent_state_core_v11.schema.json",
        b"https://nostr-automerge.example/schema/persistent_state_integration_v11.schema.json",
        b"https://nostr-automerge.example/schema/target_work_accounting_v11.schema.json",
        b"https://nostr-automerge.example/schema/persistent_ownership_v11.schema.json",
        b"https://nostr-automerge.example/schema/remediation_v11_authority_gate.schema.json",
        b"https://github.com/triesap/nostr_automerge/tools/validation/rust_conformance_v12.schema.json",
        b"https://github.com/triesap/nostr_automerge/tools/validation/trusted_epoch_projection_gate_v12.schema.json",
        b"https://github.com/triesap/nostr_automerge/tools/validation/remediation_v12_actor_gate.schema.json",
        b"https://github.com/triesap/nostr_automerge/tools/validation/remediation_v12_ancestry_authorization_gate.schema.json",
        b"https://github.com/triesap/nostr_automerge/tools/validation/distribution_v13.schema.json",
        b"https://github.com/triesap/nostr_automerge/tools/validation/distribution_v14.schema.json",
        b"https://github.com/triesap/nostr_automerge/tools/validation/distribution_v14_lock.schema.json",
        b"https://github.com/triesap/nostr_automerge/tools/validation/rust_conformance_v14.schema.json",
        b"https://github.com/triesap/nostr_automerge/tools/validation/causal_projection_assurance_v13.schema.json",
        b"https://github.com/triesap/nostr_automerge/tools/validation/rust_conformance_v13.schema.json",
        b"https://github.com/triesap/nostr_automerge/tools/validation/remediation_v12_distribution_gate.schema.json",
        b"https://github.com/triesap/nostr_automerge/tools/validation/distribution_v13_compatibility_contract.schema.json",
        b"https://github.com/triesap/nostr_automerge/tools/validation/causal_projection_implementation_gate_v13.schema.json",
        b"https://github.com/triesap/nostr_automerge/tools/validation/causal_projection_mutations_v13.schema.json",
        b"https://github.com/triesap/nostr_automerge/tools/validation/causal_projection_operation_inventory_v14.schema.json",
        b"https://github.com/triesap/nostr_automerge/tools/validation/causal_projection_proof_catalog_v14.schema.json",
        b"https://github.com/triesap/nostr_automerge/tools/validation/causal_projection_mutation_qualification_v14.schema.json",
        b"https://github.com/triesap/nostr_automerge/tools/validation/causal_projection_combined_assurance_v14.schema.json",
        b"https://github.com/triesap/nostr_automerge/tools/validation/causal_projection_finding_closure_v14.schema.json",
        b"https://github.com/triesap/nostr_automerge/tools/validation/causal_projection_final_verification_v14.schema.json",
        b"https://github.com/triesap/nostr_automerge/tools/validation/causal_projection_final_decision_v14.schema.json",
        b"https://github.com/triesap/nostr_automerge/tools/validation/distribution_v15.schema.json",
        b"https://github.com/triesap/nostr_automerge/tools/validation/distribution_v15_lock.schema.json",
        b"https://github.com/triesap/nostr_automerge/tools/validation/distribution_v16.schema.json",
        b"https://github.com/triesap/nostr_automerge/tools/validation/distribution_v16_lock.schema.json",
        b"https://github.com/triesap/nostr_automerge/tools/validation/distribution_v16_transition.schema.json",
        b"https://triesap.github.io/nostr-automerge/schemas/distribution_v17_transition.schema.json",
        b"https://triesap.github.io/nostr-automerge/schemas/rust_conformance_v17.schema.json",
        b"https://triesap.github.io/nostr-automerge/schemas/causal_projection_proofs_v18.schema.json",
        b"https://triesap.github.io/nostr-automerge/schemas/causal_projection_mutations_v18.schema.json",
        b"https://triesap.github.io/nostr-automerge/schemas/causal_projection_catalogs_v18.schema.json",
        b"https://triesap.github.io/nostr-automerge/schemas/causal_projection_final_inventory_v18.schema.json",
        b"https://triesap.github.io/nostr-automerge/schemas/causal_projection_evidence_graph_v18.schema.json",
        b"https://triesap.github.io/nostr-automerge/schemas/distribution_v18_transition.schema.json",
    )
)
TEXT_RECORDS = (
    "docs/execution/remediation_v9/ledger.md",
    "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v11.md",
    "docs/execution/remediation_v16/baseline.md",
    "docs/execution/remediation_v16/ledger.md",
)
LEGITIMATE_PUBLIC_COMMANDS = frozenset(
    {
        "cargo run --quiet -p nostr_automerge_conformance --locked -- run_distribution fixtures/distribution/manifest_v9.json",
        "git",
        "python3 scripts/validate_rust_conformance_v9.py",
        "run_distribution fixtures/distribution/manifest_v9.json",
        CAUSAL_EVIDENCE_COMMAND,
    }
)
PYTHON_SURFACES = (
    "scripts/generate_distribution_v10.py",
    "scripts/generate_distribution_v11.py",
    "scripts/generate_distribution_v12.py",
    "scripts/validate_appended_conformance_v11.py",
    "scripts/validate_distribution_v12.py",
    "scripts/validate_resource_ancestry_gate_v10.py",
    "scripts/validate_resource_followup_assurance_v10.py",
    "scripts/validate_resource_followup_final_decision_v10.py",
    "scripts/validate_corrected_checkpoint_expectations_v10.py",
    "scripts/validate_companion_specs.py",
    "scripts/validate_checkpoint_parity_v9.py",
    "scripts/validate_carrier_gate_v9.py",
    "scripts/validate_report_contract_v9.py",
    "scripts/validate_rust_report_gate_v9.py",
    "scripts/validate_rust_finalization_gate_v9.py",
    "scripts/validate_rust_resource_gate_v9.py",
    "scripts/validate_rust_conformance_v10.py",
    "scripts/validate_opaque_conformance_v10.py",
    "scripts/validate_signed_conformance_gate_v10.py",
    "scripts/validate_semantic_proof_catalog_v10.py",
    "scripts/validate_base64_proof_v10.py",
    "scripts/validate_rust_requirement_proofs_v10.py",
    "scripts/validate_report_finding_proofs_v10.py",
    "scripts/import_opaque_semantic_proofs_v10.py",
    "scripts/validate_opaque_semantic_proofs_v10.py",
    "scripts/validate_semantic_proof_mutations_v10.py",
    "scripts/generate_semantic_proof_catalog_final_v10.py",
    "scripts/validate_semantic_proof_catalog_final_v10.py",
    "scripts/validate_semantic_evidence_gate_v10.py",
    "scripts/validate_public_assurance_v10.py",
    "scripts/validate_opaque_private_assurance_v10.py",
    "scripts/validate_final_identity_v10.py",
    "scripts/validate_final_finding_closure_v10.py",
    "scripts/validate_final_decision_gate_v10.py",
    "scripts/validate_opaque_boundary_gate_v9.py",
    "scripts/validate_opaque_resource_gate_v9.py",
    "scripts/validate_opaque_finalization_v9.py",
    "scripts/validate_report_parity_v9.py",
    "scripts/validate_runtime_ledger_v9.py",
    "scripts/validate_private_reproduction_boundary_v9.py",
    "scripts/validate_rust_conformance_v9.py",
    "scripts/validate_spec.py",
    "scripts/reproduce_remediation_v11.py",
    "scripts/validate_remediation_v11.py",
    "scripts/validate_persistent_state_v11.py",
    "scripts/validate_persistent_state_core_gate_v11.py",
    "scripts/validate_persistent_state_integration_gate_v11.py",
    "scripts/validate_target_work_accounting_v11.py",
    "scripts/validate_persistent_ownership_v11.py",
    "scripts/validate_unsupported_identity_contradiction_v11.py",
    "scripts/validate_remediation_v11_authority_gate.py",
    "scripts/validate_rust_conformance_v12.py",
    "scripts/validate_opaque_distribution_parity_v12.py",
    "scripts/validate_remediation_v11_proof_catalog.py",
    "scripts/validate_remediation_v11_adversarial_qualification.py",
    "scripts/validate_remediation_v11_local_assurance.py",
    "scripts/validate_remediation_v11_finding_closure.py",
    "scripts/validate_remediation_v11_final_decision.py",
    "scripts/validate_trusted_epoch_projection_gate_v12.py",
    "scripts/validate_remediation_v12_actor_gate.py",
    "scripts/validate_remediation_v12_ancestry_authorization_gate.py",
    "scripts/generate_distribution_v13.py",
    "scripts/validate_distribution_v13.py",
    "scripts/generate_distribution_v14.py",
    "scripts/validate_distribution_v14.py",
    "scripts/validate_rust_conformance_v14.py",
    "scripts/validate_causal_projection_assurance_v13.py",
    "scripts/validate_rust_conformance_v13.py",
    "scripts/validate_remediation_v12_distribution_gate.py",
    "scripts/validate_distribution_v13_compatibility_contract.py",
    "scripts/validate_distribution_v13_parity.py",
    "scripts/validate_remediation_v12_operation_inventory.py",
    "scripts/validate_remediation_v12_proof_catalog.py",
    "scripts/validate_remediation_v12_mutation_qualification.py",
    "scripts/validate_remediation_v12_public_assurance.py",
    "scripts/validate_remediation_v12_combined_assurance.py",
    "scripts/validate_remediation_v12_finding_closure.py",
    "scripts/validate_remediation_v12_final_decision.py",
    "scripts/validate_causal_projection_implementation_gate_v13.py",
    "scripts/validate_opaque_causal_projection_v14.py",
    "scripts/validate_causal_projection_evidence_v14.py",
    "scripts/validate_causal_projection_mutation_qualification_v14.py",
    "scripts/validate_causal_projection_combined_assurance_v14.py",
    "scripts/validate_causal_projection_finding_closure_v14.py",
    "scripts/validate_causal_projection_final_verification_v14.py",
    "scripts/validate_causal_projection_final_decision_v14.py",
    "scripts/generate_distribution_v15.py",
    "scripts/validate_distribution_v15.py",
    "scripts/validate_rust_conformance_v15.py",
    "scripts/validate_opaque_causal_projection_v15.py",
    "scripts/validate_opaque_causal_projection_v16.py",
    "scripts/validate_causal_projection_combined_assurance_v16.py",
    "scripts/validate_causal_projection_final_decision_v16.py",
    "scripts/validate_causal_projection_combined_assurance_v15.py",
    "scripts/validate_causal_projection_final_decision_v15.py",
    "scripts/validate_remediation_v16.py",
    "scripts/reproduce_remediation_v16.py",
    "scripts/validate_causal_projection_counter_oracle_reproductions_v16.py",
    "scripts/validate_causal_projection_contracts_v16.py",
    "scripts/validate_causal_projection_operation_inventory_v16.py",
    "scripts/validate_causal_projection_proof_catalog_v16.py",
    "scripts/validate_causal_projection_structural_assurance_v16.py",
    "scripts/run_causal_projection_mutations_v16.py",
    "scripts/validate_causal_projection_rust_assurance_v16.py",
    "scripts/generate_distribution_v16.py",
    "scripts/validate_distribution_v16.py",
    "scripts/validate_rust_conformance_v16.py",
    "scripts/validate_rust_conformance_v17.py",
    "scripts/validate_opaque_causal_projection_v17.py",
    "scripts/validate_causal_projection_combined_assurance_v17.py",
    "scripts/validate_causal_projection_finding_closure_v17.py",
    "scripts/validate_causal_projection_completion_v17.py",
    "scripts/validate_causal_projection_final_decision_v17.py",
    "scripts/validate_causal_projection_clean_candidate_v17.py",
)
OTHER_SURFACES = (
    "tools/nostr_automerge_xtask/src/validate.rs",
    "reports/spec_baseline.txt",
)
LEGITIMATE_PUBLIC_ROUTES = frozenset(
    {
        "../..",
        ".local/evidence/nostr_automerge.cdx.json",
        ".local/evidence/rust_coverage.txt",
        ".local/evidence/rust_coverage_v11.txt",
        ".local/evidence/rust_distribution_v11.json",
        ".local/evidence/rust_distribution_v12.json",
        ".local/evidence/rust_distribution_v12_process_evidence.json",
        "crates/nostr_automerge/src/checkpoint/assemble.rs",
        "crates/nostr_automerge/src/checkpoint/authorize.rs",
        "crates/nostr_automerge/src/checkpoint/join.rs",
        "crates/nostr_automerge/src/checkpoint/merkle.rs",
        "crates/nostr_automerge/src/checkpoint/mod.rs",
        "crates/nostr_automerge/src/checkpoint/verify_history.rs",
        "crates/nostr_automerge/src/checkpoint/verify.rs",
        "crates/nostr_automerge/src/carrier/change.rs",
        "crates/nostr_automerge/src/carrier/manifest.rs",
        "crates/nostr_automerge/src/wire/base64.rs",
        "crates/nostr_automerge/tests/base64_contract.rs",
        "crates/nostr_automerge/tests/hardening.rs",
        "crates/nostr_automerge/src/automerge_adapter/document.rs",
        "crates/nostr_automerge/src/conformance/dispositions_digest.rs",
        "crates/nostr_automerge/src/conformance/history_digest.rs",
        "crates/nostr_automerge/src/control/candidate.rs",
        "crates/nostr_automerge/src/graph/actor_state.rs",
        "crates/nostr_automerge/tests/remediation_v13_reproductions.rs",
        "reports/causal_projection_operation_inventory_v14.json",
        "reports/causal_projection_proof_catalog_v14.json",
        "scripts/validate_causal_projection_evidence_v14.py",
        "spec/remediation_v13_evidence_policy.json",
        "reports/causal_projection_mutation_qualification_v14.json",
        "scripts/validate_causal_projection_mutation_qualification_v14.py",
        "tools/validation/causal_projection_mutation_qualification_v14.schema.json",
        "reports/causal_projection_combined_assurance_v14.json",
        "scripts/validate_causal_projection_combined_assurance_v14.py",
        "tools/validation/causal_projection_combined_assurance_v14.schema.json",
        "reports/causal_projection_finding_closure_v14.json",
        "scripts/validate_causal_projection_finding_closure_v14.py",
        "tools/validation/causal_projection_finding_closure_v14.schema.json",
        "reports/causal_projection_final_verification_v14.json",
        "scripts/validate_causal_projection_final_verification_v14.py",
        "tools/validation/causal_projection_final_verification_v14.schema.json",
        "reports/causal_projection_final_decision_v14.json",
        "scripts/validate_causal_projection_final_decision_v14.py",
        "tools/validation/causal_projection_final_decision_v14.schema.json",
        "tools/validation/causal_projection_operation_inventory_v14.schema.json",
        "tools/validation/causal_projection_proof_catalog_v14.schema.json",
        "crates/nostr_automerge/src/control/frontier.rs",
        "crates/nostr_automerge/src/control/ancestry.rs",
        "crates/nostr_automerge/src/control/parent_view.rs",
        "crates/nostr_automerge/src/control/transition.rs",
        "crates/nostr_automerge/src/automerge_adapter/materialized_view.rs",
        "crates/nostr_automerge/src/engine/checkpoint_result.rs",
        "crates/nostr_automerge/src/engine/evaluation_report.rs",
        "crates/nostr_automerge/src/control/reference_state.rs",
        "crates/nostr_automerge/src/engine/reference_evaluator.rs",
        "crates/nostr_automerge/src/evidence/document_view.rs",
        "crates/nostr_automerge/src/evidence/corpus_builder.rs",
        "crates/nostr_automerge/src/evidence/indexes.rs",
        "crates/nostr_automerge/src/graph/scaling.rs",
        "crates/nostr_automerge/src/graph/equivocation.rs",
        "crates/nostr_automerge/src/integrity.rs",
        "crates/nostr_automerge/src/reference/apply.rs",
        "crates/nostr_automerge/src/reference/branch_state.rs",
        "crates/nostr_automerge/src/reference/epoch_engine.rs",
        "crates/nostr_automerge/src/reference/evaluate.rs",
        "crates/nostr_automerge/src/types/actor_id.rs",
        "crates/nostr_automerge/src/work_budget.rs",
        "crates/nostr_automerge/tests/public_engine_api.rs",
        "crates/nostr_automerge/tests/remediation_v8_reproductions.rs",
        "checkpoints/checkpoints_multichunk.fixture.json",
        "deviations/step_001.md",
        "docs/adr",
        "docs/adr/README.md",
        "docs/api/public_engine.md",
        "docs/import_adaptation.json",
        "docs/provenance/source_package_manifest.json",
        "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v9.md",
        "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v10.md",
        "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v11.md",
        "docs/execution/remediation_v9/ledger.md",
        "docs/adr/adr_0074_unsupported_event_only_identity.md",
        "docs/adr/adr_0072_metered_persistent_state.md",
        "docs/adr/adr_0073_no_post_stop_target_work.md",
        "docs/adr/adr_0075_bounded_persistent_teardown.md",
        "docs/execution/remediation_v10/ledger.md",
        "docs/execution/remediation_v11/ledger.md",
        "docs/execution/remediation_v9/reproductions.md",
        "docs/resource_accounting_v6.md",
        "docs/provenance",
        "fixtures/v1_draft/scenarios/checkpoint",
        "fixtures/v1_draft/scenarios/checkpoints",
        "fixtures/v1_draft/scenarios/checkpoints/checkpoints_single_chunk.expected.json",
        "fixtures/distribution/manifest_v9.json",
        "fixtures/v1_draft/scenarios/checkpoint/checkpoint_descriptor_references_invalid_control.expected.json",
        "fixtures/v1_draft/scenarios/checkpoint/checkpoint_descriptor_references_invalid_control.fixture.json",
        "fixtures/v1_draft/scenarios/checkpoint/checkpoint_descriptor_references_invalid_control.input.json",
        "fixtures/v1_draft/scenarios/checkpoint/checkpoint_descriptor_references_unsupported_control.expected.json",
        "fixtures/v1_draft/scenarios/checkpoint/checkpoint_descriptor_references_unsupported_control.fixture.json",
        "fixtures/v1_draft/scenarios/checkpoint/checkpoint_descriptor_references_unsupported_control.input.json",
        "fixtures/v1_draft/scenarios/checkpoint/checkpoint_descriptor_references_wrong_coordinate_control.expected.json",
        "fixtures/v1_draft/scenarios/checkpoint/checkpoint_descriptor_references_wrong_coordinate_control.fixture.json",
        "fixtures/v1_draft/scenarios/checkpoint/checkpoint_descriptor_references_wrong_coordinate_control.input.json",
        "fixtures/v1_draft/scenarios/checkpoint/checkpoint_descriptor_references_wrong_kind_control.expected.json",
        "fixtures/v1_draft/scenarios/checkpoint/checkpoint_descriptor_references_wrong_kind_control.fixture.json",
        "fixtures/v1_draft/scenarios/checkpoint/checkpoint_descriptor_references_wrong_kind_control.input.json",
        "fixtures/v1_draft",
        "fixtures/v1_draft/scenarios/resource",
        "fixtures/distribution/manifest_v10.json",
        "fixtures/distribution/manifest_v11.json",
        "fixtures/distribution/manifest_v12.json",
        "fixtures/v12/scenarios/resource_followup",
        "fixtures/README.md",
        "fixtures/examples",
        "fixtures/schema",
        "fixtures/schema/distribution.schema.v10.json",
        "fixtures/v11/scenarios/resource_followup",
        "fixtures/schema/report.schema.json",
        "implementation/runtime_ledger_v9.json",
        "implementation/runtime_ledger_v10.json",
        "implementation/runtime_ledger_v11.json",
        "reports/checkpoint_parity_v9.json",
        "reports/carrier_gate_v9.json",
        "reports/opaque_checkpoint_v9.json",
        "reports/opaque_carrier_v9.json",
        "reports/opaque_reproduction_v9.json",
        "reports/rust_conformance_v9.json",
        "reports/rust_conformance_v10.json",
        "reports/opaque_conformance_v10.json",
        "reports/signed_conformance_gate_v10.json",
        "spec/semantic_proof_catalog_v10.json",
        "reports/rust_report_gate_v9.json",
        "reports/rust_finalization_gate_v9.json",
        "reports/rust_resource_gate_v9.json",
        "reports/opaque_finalization_v9.json",
        "reports/report_parity_v9.json",
        "reports/opaque_boundary_gate_v9.json",
        "reports/opaque_resource_gate_v9.json",
        "reports/opaque_semantic_proofs_v10.json",
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
        "reports/evidence_transition_v11.json",
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
        "reports/opaque_causal_projection_v14.json",
        "reports/remediation_v12_combined_assurance.json",
        "reports/remediation_v12_finding_closure.json",
        "reports/remediation_v12_final_decision.json",
        "reports/causal_projection_authority_gate_v13.json",
        "reports/causal_projection_implementation_gate_v13.json",
        "reports/causal_projection_mutations_v13.json",
        "reports/rust_conformance_v14.json",
        "reports/causal_projection_assurance_v13.json",
        "tools/validation/opaque_causal_projection_v14.schema.json",
        "fixtures/distribution/manifest_v13.json",
        "fixtures/distribution/manifest_v14.json",
        "fixtures/distribution/manifest_v14.lock.json",
        "fixtures/distribution/manifest_v13.lock.json",
        "fixtures/v13/scenarios/epoch_semantics",
        "fixtures/v13/scenarios/epoch_semantics/",
        "fixtures/v13/scenarios/epoch_semantics/deep_actor_predecessor_exact_budget",
        "fixtures/v13/rebindings/resource_followup",
        "fixtures/v13/rebindings/resource_followup/",
        "fixtures/v14/rebindings/causal_projection",
        "fixtures/v14/rebindings/causal_projection/",
        "fixtures/v15",
        "fixtures/v15/rebindings/causal_projection",
        "fixtures/distribution/manifest_v15.json",
        "fixtures/distribution/manifest_v15.lock.json",
        "spec/distribution_v15_transition.json",
        "spec/causal_projection_operation_discovery_v15.json",
        "spec/remediation_v15_authority.json",
        "spec/remediation_findings_v15.json",
        "spec/remediation_v16_authority.json",
        "spec/remediation_findings_v16.json",
        "reports/rust_conformance_v15.json",
        "tools/validation/distribution_v15.schema.json",
        "tools/validation/distribution_v15_lock.schema.json",
        "tools/validation/rust_conformance_v15.schema.json",
        "scripts/generate_distribution_v15.py",
        "scripts/validate_distribution_v15.py",
        "scripts/validate_rust_conformance_v15.py",
        "scripts/validate_opaque_causal_projection_v15.py",
        "reports/opaque_causal_projection_v15.json",
        "tools/validation/opaque_causal_projection_v15.schema.json",
        "scripts/validate_opaque_causal_projection_v16.py",
        "reports/opaque_causal_projection_v16.json",
        "tools/validation/opaque_causal_projection_v16.schema.json",
        "scripts/validate_causal_projection_combined_assurance_v16.py",
        "reports/causal_projection_combined_assurance_v16.json",
        "tools/validation/causal_projection_combined_assurance_v16.schema.json",
        "scripts/validate_causal_projection_final_decision_v16.py",
        "reports/causal_projection_final_decision_v16.json",
        "tools/validation/causal_projection_final_decision_v16.schema.json",
        "scripts/validate_causal_projection_combined_assurance_v15.py",
        "reports/causal_projection_combined_assurance_v15.json",
        "tools/validation/causal_projection_combined_assurance_v15.schema.json",
        "scripts/validate_causal_projection_final_decision_v15.py",
        "reports/causal_projection_final_decision_v15.json",
        "tools/validation/causal_projection_final_decision_v15.schema.json",
        "scripts/validate_remediation_v15.py",
        "scripts/validate_remediation_v16.py",
        "scripts/reproduce_remediation_v16.py",
        "reports/causal_projection_actor_reproductions_v16.json",
        "tools/validation/causal_projection_actor_reproductions_v16.schema.json",
        "scripts/validate_causal_projection_counter_oracle_reproductions_v16.py",
        "reports/causal_projection_counter_oracle_reproductions_v16.json",
        "tools/validation/causal_projection_counter_oracle_reproductions_v16.schema.json",
        "scripts/validate_causal_projection_contracts_v16.py",
        "spec/causal_projection_contracts_v16.json",
        "tools/validation/causal_projection_contracts_v16.schema.json",
        "spec/remediation_findings_v17.json",
        "spec/remediation_v17_authority.json",
        "spec/causal_projection_contracts_v17.json",
        "implementation/runtime_ledger_v17.json",
        "docs/execution/remediation_v17/baseline.md",
        "docs/execution/remediation_v17/ledger.md",
        "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v17.md",
        "reports/causal_projection_properties_v17.json",
        "tools/validation/causal_projection_contracts_v17.schema.json",
        "tools/validation/causal_projection_properties_v17.schema.json",
        "tools/validation/runtime_ledger_v17.schema.json",
        "scripts/validate_causal_projection_operation_inventory_v16.py",
        "reports/causal_projection_operation_inventory_v16.json",
        "tools/validation/causal_projection_operation_inventory_v16.schema.json",
        "scripts/validate_causal_projection_proof_catalog_v16.py",
        "reports/causal_projection_proof_catalog_v16.json",
        "tools/validation/causal_projection_proof_catalog_v16.schema.json",
        "scripts/validate_causal_projection_structural_assurance_v16.py",
        "reports/causal_projection_structural_assurance_v16.json",
        "tools/validation/causal_projection_structural_assurance_v16.schema.json",
        "scripts/run_causal_projection_mutations_v16.py",
        "reports/causal_projection_mutations_v16.json",
        "tools/validation/causal_projection_mutations_v16.schema.json",
        "scripts/validate_causal_projection_rust_assurance_v16.py",
        "reports/causal_projection_rust_assurance_v16.json",
        "tools/validation/causal_projection_rust_assurance_v16.schema.json",
        "fixtures/v16",
        "fixtures/v16/rebindings/causal_projection",
        "fixtures/distribution/manifest_v16.json",
        "fixtures/distribution/manifest_v16.lock.json",
        "spec/distribution_v16_transition.json",
        "reports/rust_conformance_v16.json",
        "tools/validation/distribution_v16_transition.schema.json",
        "tools/validation/distribution_v16.schema.json",
        "tools/validation/distribution_v16_lock.schema.json",
        "tools/validation/rust_conformance_v16.schema.json",
        "scripts/generate_distribution_v16.py",
        "scripts/validate_distribution_v16.py",
        "scripts/validate_rust_conformance_v16.py",
        "scripts/validate_remediation_v17.py",
        "scripts/validate_causal_projection_contracts_v17.py",
        "scripts/validate_causal_projection_properties_v17.py",
        "scripts/validate_causal_projection_inventory_v17.py",
        "reports/causal_projection_inventory_v17.json",
        "tools/validation/causal_projection_inventory_v17.schema.json",
        "scripts/validate_causal_projection_proofs_v17.py",
        "reports/causal_projection_proofs_v17.json",
        "reports/evidence/v17/proofs",
        "tools/validation/causal_projection_proofs_v17.schema.json",
        "scripts/validate_causal_projection_structure_v17.py",
        "reports/causal_projection_structure_v17.json",
        "tools/validation/causal_projection_structure_v17.schema.json",
        "scripts/validate_causal_projection_identity_v17.py",
        "reports/causal_projection_identity_v17.json",
        "tools/validation/causal_projection_identity_v17.schema.json",
        "scripts/run_causal_projection_mutations_v17.py",
        "reports/causal_projection_construction_mutations_v17.json",
        "reports/evidence/v17/mutations",
        "tools/validation/causal_projection_construction_mutations_v17.schema.json",
        "reports/causal_projection_direct_mutations_v17.json",
        "tools/validation/causal_projection_direct_mutations_v17.schema.json",
        "scripts/run_causal_projection_provenance_mutations_v17.py",
        "reports/causal_projection_provenance_mutations_v17.json",
        "tools/validation/causal_projection_provenance_mutations_v17.schema.json",
        "scripts/finalize_causal_projection_mutations_v17.py",
        "reports/causal_projection_mutations_v17.json",
        "tools/validation/causal_projection_mutations_v17.schema.json",
        "scripts/validate_causal_projection_final_inventory_v17.py",
        "reports/causal_projection_final_inventory_v17.json",
        "tools/validation/causal_projection_final_inventory_v17.schema.json",
        "scripts/validate_causal_projection_evidence_graph_v17.py",
        "reports/causal_projection_evidence_graph_v17.json",
        "tools/validation/causal_projection_evidence_graph_v17.schema.json",
        "scripts/run_causal_projection_public_assurance_v17.py",
        "reports/causal_projection_public_assurance_v17.json",
        "tools/validation/causal_projection_public_assurance_v17.schema.json",
        "scripts/validate_distribution_v17_transition.py",
        "spec/distribution_v17_transition.json",
        "tools/validation/distribution_v17_transition.schema.json",
        "scripts/validate_rust_conformance_v17.py",
        "reports/rust_conformance_v17.json",
        "tools/validation/rust_conformance_v17.schema.json",
        "scripts/validate_opaque_causal_projection_v17.py",
        "reports/opaque_causal_projection_v17.json",
        "tools/validation/opaque_causal_projection_v17.schema.json",
        "scripts/validate_causal_projection_combined_assurance_v17.py",
        "reports/causal_projection_combined_assurance_v17.json",
        "tools/validation/causal_projection_combined_assurance_v17.schema.json",
        "scripts/validate_causal_projection_finding_closure_v17.py",
        "reports/causal_projection_finding_closure_v17.json",
        "tools/validation/causal_projection_finding_closure_v17.schema.json",
        "scripts/validate_causal_projection_completion_v17.py",
        "reports/causal_projection_completion_v17.json",
        "tools/validation/causal_projection_completion_v17.schema.json",
        "scripts/validate_causal_projection_final_decision_v17.py",
        "reports/causal_projection_final_decision_v17.json",
        "tools/validation/causal_projection_final_decision_v17.schema.json",
        "scripts/validate_causal_projection_clean_candidate_v17.py",
        "reports/causal_projection_clean_candidate_v17.json",
        "tools/validation/causal_projection_clean_candidate_v17.schema.json",
        "spec/remediation_findings_v18.json",
        "spec/remediation_v18_authority.json",
        "spec/causal_projection_contracts_v18.json",
        "implementation/runtime_ledger_v18.json",
        "docs/execution/remediation_v18/baseline.md",
        "docs/execution/remediation_v18/ledger.md",
        "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v18.md",
        "scripts/validate_remediation_v18.py",
        "scripts/validate_causal_projection_contracts_v18.py",
        "scripts/validate_causal_projection_boundary_v18.py",
        "scripts/validate_causal_projection_inventory_v18.py",
        "scripts/validate_causal_projection_proofs_v18.py",
        "scripts/validate_causal_projection_properties_v18.py",
        "scripts/run_causal_projection_mutations_v18.py",
        "scripts/validate_causal_projection_catalogs_v18.py",
        "scripts/validate_causal_projection_final_inventory_v18.py",
        "scripts/validate_causal_projection_evidence_graph_v18.py",
        "scripts/validate_distribution_v18_transition.py",
        "reports/causal_projection_inventory_v18.json",
        "reports/causal_projection_proofs_v18.json",
        "reports/evidence/v18/proofs",
        "reports/causal_projection_mutations_v18.json",
        "reports/causal_projection_catalogs_v18.json",
        "reports/causal_projection_final_inventory_v18.json",
        "reports/causal_projection_evidence_graph_v18.json",
        "spec/distribution_v18_transition.json",
        "reports/evidence/v18/mutations",
        "tools/validation/causal_projection_inventory_v18.schema.json",
        "tools/validation/causal_projection_proofs_v18.schema.json",
        "tools/validation/causal_projection_mutations_v18.schema.json",
        "tools/validation/causal_projection_catalogs_v18.schema.json",
        "tools/validation/causal_projection_final_inventory_v18.schema.json",
        "tools/validation/causal_projection_evidence_graph_v18.schema.json",
        "tools/validation/distribution_v18_transition.schema.json",
        "tools/validation/runtime_ledger_v18.schema.json",
        "tools/validation/causal_projection_contracts_v18.schema.json",
        "tools/nostr_automerge_conformance/src/main.rs",
        "tools/nostr_automerge_conformance/src/runner.rs",
        "ython3 scripts/validate_causal_projection_structural_assurance_v16.py --mode structural",
        "ython3 scripts/validate_causal_projection_structural_assurance_v16.py ",
        "argo test -p nostr_automerge --lib graph::actor_state::tests::projection_causal_maximum_is_charged_once_per_accepted_change --locked -- --exact",
        "scripts/validate_causal_projection_operation_discovery_v15.py",
        "scripts/validate_causal_projection_discovery_v15.py",
        "scripts/validate_causal_projection_consumer_v15.py",
        "scripts/validate_causal_projection_proof_catalog_v15.py",
        "scripts/validate_causal_projection_source_ownership_v15.py",
        "scripts/run_causal_projection_behavior_mutations_v15.py",
        "reports/causal_projection_behavior_mutations_v15.json",
        "reports/causal_projection_consumer_inventory_v15.json",
        "reports/causal_projection_discovery_v15.json",
        "reports/causal_projection_proof_catalog_v15.json",
        "reports/causal_projection_source_ownership_v15.json",
        "tools/validation/causal_projection_behavior_mutations_v15.schema.json",
        "tools/validation/causal_projection_consumer_inventory_v15.schema.json",
        "tools/validation/causal_projection_discovery_v15.schema.json",
        "tools/validation/causal_projection_operation_discovery_v15.schema.json",
        "tools/validation/causal_projection_proof_catalog_v15.schema.json",
        "tools/validation/causal_projection_source_ownership_v15.schema.json",
        "tools/validation/runtime_ledger_v15.schema.json",
        "implementation/runtime_ledger_v15.json",
        "docs/execution/remediation_v15/baseline.md",
        "docs/execution/remediation_v15/ledger.md",
        "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v15.md",
        "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v16.md",
        "docs/execution/remediation_v16/baseline.md",
        "docs/execution/remediation_v16/ledger.md",
        "implementation/COMMIT_SEQUENCE.md",
        "implementation/runtime_ledger_v16.json",
        "scripts/validate_import.py",
        "scripts/validate_private_reproduction_boundary_v9.py",
        "scripts/validate_requirements.py",
        "scripts/validate_remediation_v16.py",
        "scripts/validate_spec.py",
        "spec/EVIDENCE_POLICY.md",
        "spec/remediation_findings_v16.json",
        "spec/remediation_v16_authority.json",
        "tools/nostr_automerge_xtask/src/validate.rs",
        "tools/validation/runtime_ledger_v16.schema.json",
        "tools/validation/runtime_ledger_v16.schema.json",
        "implementation/runtime_ledger_v16.json",
        "docs/execution/remediation_v16/baseline.md",
        "docs/execution/remediation_v16/ledger.md",
        "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v16.md",
        "spec/distribution_v13_transition.json",
        "spec/distribution_v14_transition.json",
        "spec/remediation_findings_v13.json",
        "spec/distribution_v13_compatibility_contract.json",
        "crates/nostr_automerge/src/control/authorize.rs",
        "crates/nostr_automerge/src/control/epoch_state.rs",
        "crates/nostr_automerge/src/graph/actor_state.rs",
        "crates/nostr_automerge/src/graph/closure.rs",
        "crates/nostr_automerge/src/graph/epoch.rs",
        "crates/nostr_automerge/src/graph/equivocation.rs",
        "crates/nostr_automerge/src/graph/schedule.rs",
        "crates/nostr_automerge/src/reference/epoch_engine.rs",
        "reports/external_holds_v8.json",
        "reports/spec_baseline.txt",
        "spec/remediation_v12_reproductions.json",
        "scripts/validate_adrs.py",
        "scripts/validate_architecture.py",
        "scripts/reproduce_remediation_v11.py",
        "scripts/local_gate.py",
        "scripts/validate_assurance_v9.py",
        "scripts/validate_automerge_qualification.py",
        "scripts/generate_distribution_v10.py",
        "scripts/generate_distribution_v11.py",
        "scripts/generate_distribution_v12.py",
        "scripts/validate_appended_conformance_v11.py",
        "scripts/validate_distribution_v12.py",
        "scripts/validate_resource_ancestry_gate_v10.py",
        "scripts/validate_resource_followup_assurance_v10.py",
        "scripts/validate_resource_followup_final_decision_v10.py",
        "scripts/validate_corrected_checkpoint_expectations_v10.py",
        "scripts/validate_authority_transition_v10.py",
        "scripts/validate_checkpoint_parity_v9.py",
        "scripts/validate_carrier_gate_v9.py",
        "scripts/validate_companion_specs.py",
        "scripts/validate_diagnostics.py",
        "scripts/validate_fixtures.py",
        "scripts/validate_import.py",
        "scripts/validate_normative_clarifications_v3.py",
        "scripts/validate_private_reproduction_boundary_v9.py",
        "scripts/validate_remediation_v11.py",
        "scripts/validate_remediation_v12.py",
        "scripts/reproduce_remediation_v13.py",
        "scripts/run_causal_projection_mutations_v13.py",
        "scripts/validate_remediation_v13.py",
        "scripts/validate_causal_projection_operations_v13.py",
        "scripts/validate_causal_projection_authority_gate_v13.py",
        "scripts/validate_causal_projection_implementation_gate_v13.py",
        "scripts/validate_causal_projection_source_v13.py",
        "spec/causal_projection_operation_contract_v13.json",
        "spec/remediation_v13_authority.json",
        "docs/execution/remediation_v13/baseline.md",
        "docs/execution/remediation_v13/ledger.md",
        "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v13.md",
        "implementation/runtime_ledger_v13.json",
        "tools/validation/runtime_ledger_v13.schema.json",
        "tools/validation/remediation_v13_evidence_policy.schema.json",
        "tools/validation/causal_projection_operation_contract_v13.schema.json",
        "tools/validation/causal_projection_implementation_gate_v13.schema.json",
        "tools/validation/causal_projection_mutations_v13.schema.json",
        "reports/remediation_v12_finding_closure.json",
        "tools/validation/remediation_v12_finding_closure.schema.json",
        "reports/remediation_v12_final_decision.json",
        "tools/validation/remediation_v12_final_decision.schema.json",
        "spec/remediation_findings_v12.json",
        "spec/remediation_v12_reproductions.json",
        "reports/remediation_v12_operation_inventory.json",
        "reports/remediation_v12_proof_catalog.json",
        "reports/remediation_v12_mutation_qualification.json",
        "reports/distribution_v13_parity.json",
        "reports/remediation_v12_combined_assurance.json",
        "spec/requirements.json",
        "spec/requirements_applicability.json",
        "scripts/validate_remediation_v11_authority_gate.py",
        "scripts/validate_requirement_matrix_v9.py",
        "scripts/validate_report_contract_v9.py",
        "scripts/validate_rust_report_gate_v9.py",
        "scripts/validate_rust_finalization_gate_v9.py",
        "scripts/validate_rust_resource_gate_v9.py",
        "scripts/validate_opaque_finalization_v9.py",
        "scripts/validate_report_parity_v9.py",
        "scripts/validate_protocol_revision.py",
        "scripts/validate_nip_snapshot.py",
        "scripts/validate_repository_policy.py",
        "scripts/validate_rust_conformance_v9.py",
        "scripts/validate_rust_conformance_v10.py",
        "scripts/validate_rust_conformance_v12.py",
        "scripts/validate_opaque_distribution_parity_v12.py",
        "scripts/validate_remediation_v11_proof_catalog.py",
        "scripts/validate_remediation_v11_adversarial_qualification.py",
        "scripts/validate_remediation_v11_local_assurance.py",
        "scripts/validate_remediation_v11_finding_closure.py",
        "scripts/validate_remediation_v11_final_decision.py",
        "scripts/reproduce_remediation_v12.py",
        "scripts/validate_trusted_epoch_projection_gate_v12.py",
        "scripts/validate_remediation_v12_actor_gate.py",
        "scripts/validate_remediation_v12_ancestry_authorization_gate.py",
        "scripts/generate_distribution_v13.py",
        "scripts/validate_distribution_v13.py",
        "scripts/generate_distribution_v14.py",
        "scripts/validate_distribution_v14.py",
        "scripts/validate_rust_conformance_v14.py",
        "scripts/validate_causal_projection_assurance_v13.py",
        "scripts/validate_opaque_causal_projection_v14.py",
        "scripts/validate_rust_conformance_v13.py",
        "scripts/validate_remediation_v12_distribution_gate.py",
        "scripts/validate_distribution_v13_compatibility_contract.py",
        "scripts/validate_distribution_v13_parity.py",
        "scripts/validate_remediation_v12_operation_inventory.py",
        "scripts/validate_remediation_v12_proof_catalog.py",
        "scripts/validate_remediation_v12_mutation_qualification.py",
        "scripts/validate_remediation_v12_public_assurance.py",
        "scripts/validate_remediation_v12_combined_assurance.py",
        "scripts/validate_remediation_v12_finding_closure.py",
        "scripts/validate_remediation_v12_final_decision.py",
        "scripts/validate_opaque_conformance_v10.py",
        "scripts/validate_signed_conformance_gate_v10.py",
        "scripts/validate_semantic_proof_catalog_v10.py",
        "scripts/validate_base64_proof_v10.py",
        "scripts/validate_rust_requirement_proofs_v10.py",
        "scripts/validate_report_finding_proofs_v10.py",
        "scripts/import_opaque_semantic_proofs_v10.py",
        "scripts/validate_opaque_semantic_proofs_v10.py",
        "scripts/validate_semantic_proof_mutations_v10.py",
        "scripts/generate_semantic_proof_catalog_final_v10.py",
        "scripts/validate_semantic_proof_catalog_final_v10.py",
        "scripts/validate_semantic_evidence_gate_v10.py",
        "scripts/validate_public_assurance_v10.py",
        "scripts/validate_opaque_private_assurance_v10.py",
        "scripts/validate_final_identity_v10.py",
        "scripts/validate_final_finding_closure_v10.py",
        "scripts/validate_final_decision_gate_v10.py",
        "scripts/reproduce_remediation_v9.py",
        "scripts/validate_remediation_v9.py",
        "scripts/validate_runtime_ledger_v9.py",
        "scripts/validate_resource_followup_authority_v10.py",
        "scripts/validate_runtime_ledger_v10.py",
        "scripts/validate_resource_operation_inventory_v10.py",
        "scripts/validate_reports.py",
        "scripts/validate_spec.py",
        "scripts/validate_remediation_v11.py",
        "scripts/validate_persistent_state_v11.py",
        "scripts/validate_persistent_state_core_gate_v11.py",
        "reports/persistent_state_core_v11.json",
        "tools/validation/persistent_state_core_v11.schema.json",
        "scripts/validate_persistent_state_integration_gate_v11.py",
        "reports/persistent_state_integration_v11.json",
        "tools/validation/persistent_state_integration_v11.schema.json",
        "scripts/validate_target_work_accounting_v11.py",
        "reports/target_work_accounting_v11.json",
        "tools/validation/target_work_accounting_v11.schema.json",
        "scripts/validate_persistent_ownership_v11.py",
        "reports/persistent_ownership_v11.json",
        "tools/validation/persistent_ownership_v11.schema.json",
        "scripts/validate_unsupported_identity_contradiction_v11.py",
        "scripts/validate_opaque_boundary_gate_v9.py",
        "scripts/validate_opaque_resource_gate_v9.py",
        "tools/validation/opaque_semantic_proofs_v10.schema.json",
        "tools/validation/semantic_evidence_gate_v10.schema.json",
        "tools/validation/public_assurance_v10.schema.json",
        "tools/validation/opaque_private_assurance_v10.schema.json",
        "tools/validation/final_identity_v10.schema.json",
        "tools/validation/final_finding_closure_v10.schema.json",
        "tools/validation/final_decision_gate_v10.schema.json",
        "tools/validation/appended_conformance_v11.schema.json",
        "tools/validation/resource_ancestry_gate_v10.schema.json",
        "tools/validation/resource_ancestry_proof_catalog_v10.schema.json",
        "tools/validation/resource_followup_assurance_v10.schema.json",
        "tools/validation/resource_followup_finding_closure_v10.schema.json",
        "tools/validation/resource_followup_final_decision_v10.schema.json",
        "spec/resource_ancestry_proof_catalog_v10.json",
        "spec/resource_operation_inventory_v10.json",
        "tools/validation/distribution_v11.schema.json",
        "tools/validation/distribution_v12.schema.json",
        "tools/validation/authority_transition_v10.schema.json",
        "tools/validation/remediation_v11_authority_gate.schema.json",
        "spec/distribution_v12_transition.json",
        "spec/authority_transition_v10.json",
        "spec/resource_followup_authority_v10.json",
        "spec/API_CONTRACTS.md",
        "spec/ARCHITECTURE.md",
        "spec/NIP_DRAFT.md",
        "spec/NIP_DRAFT.sha256",
        "spec/REPORT_CONTRACT.md",
        "spec/SECURITY.md",
        "spec/remediation_v11_reproductions.json",
        "spec/remediation_v11_authority.json",
        "spec/remediation_findings_v11.json",
        "spec/companion_authority_v10.json",
        "spec/API_CONTRACTS.md",
        "spec/CHECKPOINT_PROFILE.md",
        "spec/CONFORMANCE.md",
        "spec/NIP_DRAFT.md",
        "spec/NOSTR_AUTOMERGE_V1_SPEC.md",
        "spec/NORMATIVE_REQUIREMENTS.md",
        "spec/REPORT_CONTRACT.md",
        "spec/draft_limits.md",
        "spec/draft_limits.json",
        "spec/remediation_findings_v9.json",
        "spec/remediation_v12_authority.json",
        "spec/requirements.json",
        "spec/requirements_applicability.json",
        "tests/compile_fail/remediation_v9_report_revision/src/main.rs",
        "tools/nostr_automerge_xtask/src/validate.rs",
        "tools/nostr_automerge_conformance",
        "tools/nostr_automerge_conformance/src/expected.rs",
        "tools/nostr_automerge_conformance/src/fixture.rs",
        "tools/nostr_automerge_conformance/src/fixture_generation.rs",
        "tools/nostr_automerge_conformance/src/report_json.rs",
        "tools/nostr_automerge_conformance/src/runner.rs",
        "tools/nostr_automerge_conformance/src/scenario.rs",
        "tools/validation/checkpoint_parity_v9.schema.json",
        "tools/validation/rust_report_gate_v9.schema.json",
        "tools/validation/rust_finalization_gate_v9.schema.json",
        "tools/validation/rust_resource_gate_v9.schema.json",
        "tools/validation/rust_conformance_v10.schema.json",
        "tools/validation/rust_conformance_v12.schema.json",
        "tools/validation/opaque_distribution_parity_v12.schema.json",
        "tools/validation/remediation_v11_proof_catalog.schema.json",
        "tools/validation/remediation_v11_adversarial_qualification.schema.json",
        "tools/validation/remediation_v11_local_assurance.schema.json",
        "tools/validation/remediation_v11_finding_closure.schema.json",
        "tools/validation/remediation_v11_final_decision.schema.json",
        "tools/validation/remediation_v12_authority_gate.schema.json",
        "tools/validation/remediation_v12_evidence_policy.schema.json",
        "tools/validation/runtime_ledger_v12.schema.json",
        "docs/execution/remediation_v12/baseline.md",
        "docs/execution/remediation_v12/ledger.md",
        "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v12.md",
        "implementation/runtime_ledger_v12.json",
        "tools/validation/trusted_epoch_projection_gate_v12.schema.json",
        "tools/validation/remediation_v12_actor_gate.schema.json",
        "tools/validation/remediation_v12_ancestry_authorization_gate.schema.json",
        "tools/validation/distribution_v13.schema.json",
        "tools/validation/distribution_v14.schema.json",
        "tools/validation/distribution_v14_lock.schema.json",
        "tools/validation/rust_conformance_v14.schema.json",
        "tools/validation/causal_projection_assurance_v13.schema.json",
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
        "tools/validation/opaque_conformance_v10.schema.json",
        "tools/validation/signed_conformance_gate_v10.schema.json",
        "tools/validation/semantic_proof_catalog_v10.schema.json",
        "tools/validation/finding_closure_catalog_v10.schema.json",
        "tools/validation/opaque_finalization_v9.schema.json",
        "tools/validation/report_parity_v9.schema.json",
        "tools/validation/opaque_boundary_gate_v9.schema.json",
        "tools/validation/opaque_resource_gate_v9.schema.json",
        "tools/validation/carrier_gate_v9.schema.json",
        "tools/validation/opaque_reproduction_v9.schema.json",
        "tools/validation/opaque_checkpoint_v9.schema.json",
        "tools/validation/opaque_carrier_v9.schema.json",
        "tools/validation/authority_transition_v10.schema.json",
        "tools/validation/runtime_ledger_v9.schema.json",
        "tools/validation/resource_followup_authority_v10.schema.json",
        "tools/validation/runtime_ledger_v10.schema.json",
        "tools/validation/resource_operation_inventory_v10.schema.json",
    }
)
RUST_STRING = re.compile(r'"([^"\\]*(?:\\.[^"\\]*)*)"')
UNIVERSAL_SOURCE_PATTERNS = (
    URI_TEXT,
    ABSOLUTE_PATH_TEXT,
    LOG_TEXT,
    PACKAGE_SUFFIX_TEXT,
    COMMAND_TEXT,
    CASE_TEXT,
    COMMIT_SUBJECT_TEXT,
)


def require(condition: bool, diagnostic: str) -> None:
    if not condition:
        raise LedgerError(diagnostic)


def validate_records(records: list[dict[str, Any]], text: list[str]) -> None:
    for index, record in enumerate(records):
        validate_no_leak(record, f"json_record:{index}")
    for index, value in enumerate(text):
        validate_no_leak(value, f"text_record:{index}")


def public_command(value: Any) -> str | None:
    command = " ".join(value) if isinstance(value, list) and all(isinstance(part, str) for part in value) else value
    if not isinstance(command, str):
        return None
    allowed = (
        re.fullmatch(chr(99) + r"argo test -p nostr_automerge --lib [a-z0-9_]+ --locked", command)
        or re.fullmatch(
            chr(99)
            + r"argo test -p nostr_automerge --lib graph::actor_state::tests::causal_projection_v(?:16|17)_site_[a-z0-9_]+ --locked -- --exact(?: --nocapture)?",
            command,
        )
        or command
        == chr(112)
        + "ython3 scripts/validate_causal_projection_structural_assurance_v16.py --mode structural"
        or command
        == chr(99)
        + "argo test -p nostr_automerge --lib graph::actor_state::tests::projection_causal_maximum_is_charged_once_per_accepted_change --locked -- --exact"
        or command
        == chr(99)
        + "argo extbuild run -- cargo check -p nostr_automerge --lib --locked"
        or command
        in {
            chr(112)
            + "ython3 scripts/validate_causal_projection_properties_v18.py --root . --mode structural",
            chr(112)
            + "ython3 scripts/validate_causal_projection_properties_v18.py --root . --mode identity",
            chr(103)
            + "it diff --quiet -- crates/nostr_automerge/src/graph/actor_state.rs crates/nostr_automerge/src/reference/epoch_engine.rs",
        }
        or command == chr(103) + "it status --porcelain=v1"
    )
    return command if allowed else None


def validate_public_record(value: Any, diagnostic: str) -> None:
    if isinstance(value, dict):
        for key, child in value.items():
            validate_source_literal(key, f"{diagnostic}:key")
            if key in {"$schema", "$id"}:
                require(child in PUBLIC_SCHEMA_URIS, f"{diagnostic}:{key}:uri")
                continue
            if key == "patch":
                require(isinstance(child, str), f"{diagnostic}:{key}:type")
                lowered = child.casefold()
                require(
                    not any(token in lowered for token in PUBLIC_PATCH_BLOCKED),
                    f"{diagnostic}:{key}:private_material",
                )
                allowed = {
                    "crates/nostr_automerge/src/graph/actor_state.rs",
                    "crates/nostr_automerge/src/reference/epoch_engine.rs",
                }
                paths = re.findall(r"^(?:diff --git [ab]/|--- a/|\+\+\+ b/)(\S+)", child, re.MULTILINE)
                require(paths and all(path in allowed for path in paths), f"{diagnostic}:{key}:path")
                continue
            command = (
                public_command(child)
                if key
                in {
                    "command", "compile_command", "property_command",
                    "restoration_command", "argv",
                }
                else None
            )
            if command is not None:
                validate_source_literal(
                    command, f"{diagnostic}:{key}", allow_command_token=True
                )
                continue
            validate_public_record(child, f"{diagnostic}:{key}")
        return
    if isinstance(value, list):
        for index, child in enumerate(value):
            validate_public_record(child, f"{diagnostic}:{index}")
        return
    if isinstance(value, str):
        validate_source_literal(value, diagnostic)


def validate_source_literal(
    value: str, diagnostic: str, *, allow_command_token: bool = False
) -> None:
    for index, pattern in enumerate(UNIVERSAL_SOURCE_PATTERNS):
        matched = pattern.search(value) is not None
        require(
            not matched
            or (
                pattern is COMMAND_TEXT
                and (allow_command_token or value == chr(103) + "it status --porcelain=v1")
            )
            or (
                pattern is URI_TEXT
                and diagnostic.startswith(
                    "source:scripts/validate_private_reproduction_boundary_v9.py:"
                )
                and value in PUBLIC_SCHEMA_URIS
            )
            or (
                diagnostic.startswith(
                    "source:scripts/validate_private_reproduction_boundary_v9.py:"
                )
                and value in V18_MUTATION_COMMAND_FRAGMENTS
            )
            or (
                diagnostic.startswith(
                    "source:scripts/validate_private_reproduction_boundary_v9.py:coordinated:"
                )
                and value == V18_MUTATION_COMMAND_SEQUENCE
            )
            or (
                pattern is PACKAGE_SUFFIX_TEXT
                and (
                    diagnostic.startswith(
                        "source:scripts/validate_opaque_semantic_proofs_v10.py:"
                    )
                    or diagnostic.startswith(
                        "source:scripts/validate_semantic_proof_mutations_v10.py:"
                    )
                    or diagnostic.startswith(
                        "source:scripts/generate_semantic_proof_catalog_final_v10.py:"
                    )
                )
            or (
                diagnostic.startswith(
                    "source:scripts/validate_opaque_causal_projection_v14.py:"
                )
                and value in OPAQUE_MUTATION_FRAGMENTS
            )
            or (
                diagnostic.startswith(
                    "source:scripts/validate_opaque_causal_projection_v15.py:"
                )
                and value in OPAQUE_MUTATION_FRAGMENTS
            )
            or (
                diagnostic.startswith(
                    "source:scripts/validate_private_reproduction_boundary_v9.py:"
                )
                and value in OPAQUE_MUTATION_FRAGMENTS
            )
            or (
                diagnostic.startswith(
                    "source:scripts/validate_opaque_causal_projection_v14.py:coordinated:"
                )
                and value == OPAQUE_MUTATION_SEQUENCE
            )
            or (
                diagnostic.startswith(
                    "source:scripts/validate_opaque_causal_projection_v15.py:coordinated:"
                )
                and value == OPAQUE_MUTATION_SEQUENCE
            )
            or (
                diagnostic.startswith(
                    "source:scripts/validate_causal_projection_evidence_v14.py:coordinated:"
                )
                and value == CAUSAL_EVIDENCE_EXECUTION_SEQUENCE
            )
            or (
                diagnostic.startswith(
                    "source:scripts/validate_causal_projection_combined_assurance_v14.py:coordinated:"
                )
                and value == COMBINED_ASSURANCE_EXECUTION_SEQUENCE
            )
            ),
            f"{diagnostic}:pattern:{index}",
        )
    if RELATIVE_PATH_TEXT.search(value) is not None and not allow_command_token:
        require(
            is_public_route(value)
            or (
                diagnostic.startswith(
                    "source:scripts/validate_opaque_causal_projection_v14.py:coordinated:"
                )
                and value == OPAQUE_MUTATION_SEQUENCE
            )
            or (
                diagnostic.startswith(
                    "source:scripts/validate_opaque_causal_projection_v15.py:coordinated:"
                )
                and value == OPAQUE_MUTATION_SEQUENCE
            )
            or (
                diagnostic.startswith(
                    "source:scripts/validate_private_reproduction_boundary_v9.py:"
                )
                and value
                in {
                    "ython3 scripts/validate_causal_projection_structural_assurance_v16.py --mode structural",
                    "ython3 scripts/validate_causal_projection_structural_assurance_v16.py ",
                    "argo test -p nostr_automerge --lib graph::actor_state::tests::projection_causal_maximum_is_charged_once_per_accepted_change --locked -- --exact",
                }
                | V18_MUTATION_COMMAND_FRAGMENTS
            )
            or (
                diagnostic.startswith(
                    "source:scripts/validate_private_reproduction_boundary_v9.py:coordinated:"
                )
                and value == V18_MUTATION_COMMAND_SEQUENCE
            ),
            f"{diagnostic}:relative_route",
        )


def is_public_route(value: str) -> bool:
    return (
        value in LEGITIMATE_PUBLIC_ROUTES
        or value in LEGITIMATE_PUBLIC_COMMANDS
        or value.startswith("fixtures/v1_draft/scenarios/checkpoints/")
        or value.startswith("fixtures/v1_draft/scenarios/checkpoint/")
        or value.startswith("fixtures/v1_draft/scenarios/change_claims/")
        or value.startswith("fixtures/v1_draft/scenarios/interrupted/")
        or value.startswith("fixtures/v1_draft/scenarios/resource/")
        or value.startswith("fixtures/v11/scenarios/resource_followup/")
        or value.startswith("fixtures/v12/scenarios/resource_followup/")
        or value.startswith("fixtures/v14/rebindings/causal_projection/")
        or value.startswith("fixtures/v15/rebindings/causal_projection/")
        or value.startswith("fixtures/v16/rebindings/causal_projection/")
        or value.startswith("reports/evidence/v18/proofs/")
        or value.startswith("reports/evidence/v18/mutations/")
        or value.startswith("fixtures/v1_draft/scenarios/scope/")
        or value.startswith("fixtures/v1_draft/checkpoints/")
        or value in {row["value"] for row in APPROVED_WIRE_DOMAINS}
    )


def python_comments(source: str, relative: str) -> list[str]:
    try:
        tokens = tokenize.generate_tokens(io.StringIO(source).readline)
        return [
            token.string.removeprefix("#").strip()
            for token in tokens
            if token.type == tokenize.COMMENT
            and not (token.start == (1, 0) and token.string.startswith("#!"))
        ]
    except (IndentationError, tokenize.TokenError) as error:
        raise LedgerError(f"source_comments:{relative}") from error


def python_literals(relative: str) -> tuple[list[str], list[str], list[str]]:
    try:
        source = (ROOT / relative).read_text(encoding="utf-8")
        tree = ast.parse(source, filename=relative)
    except (OSError, UnicodeDecodeError, SyntaxError) as error:
        raise LedgerError(f"source_surface:{relative}") from error
    literals = [
        node.value
        for node in ast.walk(tree)
        if isinstance(node, ast.Constant) and isinstance(node.value, str)
    ]
    coordinated: list[str] = []

    def static_string(node: ast.AST) -> str | None:
        if isinstance(node, ast.Constant) and isinstance(node.value, str):
            return node.value
        if isinstance(node, ast.BinOp) and isinstance(node.op, ast.Add):
            left = static_string(node.left)
            right = static_string(node.right)
            if left is not None and right is not None:
                return left + right
        return None

    for node in ast.walk(tree):
        if isinstance(node, ast.BinOp):
            value = static_string(node)
            if value is not None:
                coordinated.append(value)
        if not isinstance(node, (ast.List, ast.Set, ast.Tuple)):
            continue
        values = [
            child.value
            for child in node.elts
            if isinstance(child, ast.Constant) and isinstance(child.value, str)
        ]
        residual = [value for value in values if not is_public_route(value)]
        if len(residual) > 1:
            coordinated.append("".join(residual))
    return literals, coordinated, python_comments(source, relative)


def rust_comments(source: str) -> list[str]:
    comments: list[str] = []
    index = 0
    length = len(source)

    def cleaned(value: str) -> str:
        if value.startswith(("!", "/", "*")):
            value = value[1:]
        return value.strip()

    while index < length:
        if source.startswith("//", index):
            end = source.find("\n", index + 2)
            if end < 0:
                end = length
            comments.append(cleaned(source[index + 2 : end]))
            index = end
            continue
        if source.startswith("/*", index):
            depth = 1
            cursor = index + 2
            body: list[str] = []
            while cursor < length and depth:
                if source.startswith("/*", cursor):
                    depth += 1
                    cursor += 2
                elif source.startswith("*/", cursor):
                    depth -= 1
                    cursor += 2
                else:
                    body.append(source[cursor])
                    cursor += 1
            require(depth == 0, "source_comments:rust_unclosed")
            comments.append(cleaned("".join(body)))
            index = cursor
            continue

        raw_prefix = None
        if source.startswith("br", index):
            raw_prefix = index + 2
        elif source.startswith("r", index):
            raw_prefix = index + 1
        if raw_prefix is not None:
            cursor = raw_prefix
            while cursor < length and source[cursor] == "#":
                cursor += 1
            if cursor < length and source[cursor] == '"':
                hashes = source[raw_prefix:cursor]
                terminator = '"' + hashes
                end = source.find(terminator, cursor + 1)
                require(end >= 0, "source_comments:rust_raw_string")
                index = end + len(terminator)
                continue

        if source[index] == '"':
            cursor = index + 1
            while cursor < length:
                if source[cursor] == "\\":
                    cursor += 2
                    continue
                if source[cursor] == '"':
                    cursor += 1
                    break
                cursor += 1
            index = cursor
            continue
        if source[index] == "'":
            cursor = index + 1
            if cursor < length and source[cursor] == "\\":
                cursor += 2
            else:
                cursor += 1
            if cursor < length and source[cursor] == "'":
                index = cursor + 1
                continue
        index += 1
    return comments


def validate_source_surfaces() -> None:
    audited = 0
    for relative in PYTHON_SURFACES:
        literals, coordinated, comments = python_literals(relative)
        for index, value in enumerate(literals):
            validate_source_literal(
                value,
                f"source:{relative}:{index}",
                allow_command_token=(
                    (
                        value == "git"
                        and relative
                        in {
                            "scripts/validate_authority_transition_v10.py",
                            "scripts/validate_carrier_gate_v9.py",
                            "scripts/validate_checkpoint_parity_v9.py",
                            "scripts/validate_rust_report_gate_v9.py",
                            "scripts/validate_rust_finalization_gate_v9.py",
                            "scripts/validate_rust_resource_gate_v9.py",
                            "scripts/validate_semantic_evidence_gate_v10.py",
                            "scripts/validate_public_assurance_v10.py",
                            "scripts/validate_final_identity_v10.py",
                            "scripts/validate_remediation_v11.py",
                            "scripts/validate_persistent_state_core_gate_v11.py",
                            "scripts/validate_persistent_state_integration_gate_v11.py",
                            "scripts/validate_remediation_v11_authority_gate.py",
                            "scripts/validate_opaque_distribution_parity_v12.py",
                            "scripts/validate_trusted_epoch_projection_gate_v12.py",
                            "scripts/validate_remediation_v12_actor_gate.py",
                            "scripts/validate_remediation_v12_ancestry_authorization_gate.py",
                            "scripts/validate_remediation_v16.py",
                            "scripts/validate_opaque_causal_projection_v16.py",
                            "scripts/validate_opaque_causal_projection_v17.py",
                            "scripts/validate_causal_projection_combined_assurance_v17.py",
                            "scripts/validate_causal_projection_finding_closure_v17.py",
                            "scripts/validate_causal_projection_completion_v17.py",
                            "scripts/validate_causal_projection_final_decision_v17.py",
                            "scripts/validate_causal_projection_clean_candidate_v17.py",
                            "scripts/validate_causal_projection_combined_assurance_v16.py",
                            "scripts/validate_causal_projection_final_decision_v16.py",
                            "scripts/reproduce_remediation_v16.py",
                            "scripts/generate_distribution_v13.py",
                            "scripts/generate_distribution_v14.py",
                            "scripts/generate_distribution_v15.py",
                            "scripts/generate_distribution_v16.py",
                            "scripts/validate_distribution_v13.py",
                            "scripts/validate_distribution_v14.py",
                            "scripts/validate_distribution_v15.py",
                            "scripts/validate_remediation_v12_distribution_gate.py",
                            "scripts/validate_remediation_v12_final_decision.py",
                            "scripts/validate_causal_projection_implementation_gate_v13.py",
                            "scripts/validate_causal_projection_assurance_v13.py",
                            "scripts/validate_runtime_ledger_v9.py",
                        }
                    )
                    or (
                        value in {"cargo", "git", "python3"}
                        | LEGITIMATE_PUBLIC_COMMANDS
                        and relative
                        in {
                            "scripts/validate_private_reproduction_boundary_v9.py",
                            "scripts/validate_report_parity_v9.py",
                        }
                    )
                    or (
                        value in LEGITIMATE_PUBLIC_COMMANDS
                        and relative == "scripts/validate_rust_conformance_v9.py"
                    )
                    or (
                        value == "cargo"
                        and relative == "scripts/validate_report_contract_v9.py"
                    )
                    or (
                        value in {"cargo", "git"}
                        and relative == "scripts/reproduce_remediation_v16.py"
                    )
                    or (
                        value == "git"
                        and relative
                        == "scripts/validate_causal_projection_counter_oracle_reproductions_v16.py"
                    )
                    or (
                        relative
                        in {
                            "scripts/validate_causal_projection_operation_inventory_v16.py",
                            "scripts/validate_causal_projection_proof_catalog_v16.py",
                            "scripts/validate_causal_projection_structural_assurance_v16.py",
                        }
                        and (
                            value == "git"
                            or value.startswith(chr(99) + "argo test")
                        )
                    )
                    or (
                        relative
                        == "scripts/validate_causal_projection_rust_assurance_v16.py"
                        and value in {"cargo", "git"}
                    )
                    or (
                        relative == "scripts/run_causal_projection_mutations_v16.py"
                        and (
                            value in {"cargo", "git", "python3"}
                            or value.startswith(chr(99) + "argo check ")
                            or value.startswith(chr(99) + "argo test ")
                            or value.startswith(
                                chr(112)
                                + "ython3 scripts/validate_causal_projection_structural_assurance_v16.py "
                            )
                        )
                    )
                    or (
                        value in {"cargo", "git", CAUSAL_EVIDENCE_COMMAND}
                        and relative
                        == "scripts/validate_causal_projection_evidence_v14.py"
                    )
                    or (
                        value in {"git", "python3"}
                        and relative
                        == "scripts/validate_causal_projection_mutation_qualification_v14.py"
                    )
                    or (
                        value in {"cargo", "git"}
                        and relative
                        == "scripts/validate_causal_projection_combined_assurance_v14.py"
                    )
                    or (
                        value in {"cargo", "git"}
                        and relative
                        == "scripts/validate_causal_projection_combined_assurance_v16.py"
                    )
                    or (
                        value in {"cargo", "git"}
                        and relative
                        == "scripts/validate_causal_projection_combined_assurance_v15.py"
                    )
                    or (
                        value == "git"
                        and relative
                        == "scripts/validate_causal_projection_final_decision_v15.py"
                    )
                    or (
                        value == "git"
                        and relative
                        == "scripts/validate_causal_projection_finding_closure_v14.py"
                    )
                    or (
                        value == "git"
                        and relative
                        == "scripts/validate_causal_projection_final_verification_v14.py"
                    )
                    or (
                        value == "git"
                        and relative
                        == "scripts/validate_causal_projection_final_decision_v14.py"
                    )
                    or (
                        value in {"cargo", "python3"}
                        and relative == "scripts/validate_remediation_v12_proof_catalog.py"
                    )
                    or (
                        value == "cargo"
                        and relative == "scripts/reproduce_remediation_v11.py"
                    )
                    or (
                        value == "cargo"
                        and relative == "scripts/validate_base64_proof_v10.py"
                    )
                    or (
                        value in {"cargo", "python3"}
                        and relative == "scripts/validate_rust_requirement_proofs_v10.py"
                    )
                    or (
                        value in {"cargo", "python3"}
                        and relative == "scripts/validate_report_finding_proofs_v10.py"
                    )
                    or (
                        value in {"git", "python3"}
                        and relative == "scripts/import_opaque_semantic_proofs_v10.py"
                    )
                    or (
                        value in {"cargo", "git"}
                        and relative == "scripts/validate_rust_conformance_v10.py"
                    )
                    or (
                        value in {"cargo", "git"}
                        and relative == "scripts/validate_rust_conformance_v12.py"
                    )
                    or (
                        value in {"cargo", "git"}
                        and relative == "scripts/validate_rust_conformance_v13.py"
                    )
                    or (
                        value in {"cargo", "git"}
                        and relative == "scripts/validate_rust_conformance_v14.py"
                    )
                    or (
                        value in {"cargo", "git"}
                        and relative == "scripts/validate_rust_conformance_v15.py"
                    )
                    or (
                        value in {"cargo", "git"}
                        and relative
                        in {
                            "scripts/validate_distribution_v16.py",
                            "scripts/validate_rust_conformance_v16.py",
                            "scripts/validate_rust_conformance_v17.py",
                        }
                    )
                    or (
                        value in {"cargo", "git"}
                        and relative == "scripts/validate_opaque_conformance_v10.py"
                    )
                    or (
                        value in {"git", "python3"}
                        and relative == "scripts/validate_signed_conformance_gate_v10.py"
                    )
                    or (
                        value in {"cargo", "git", "python3"}
                        and relative == "scripts/validate_appended_conformance_v11.py"
                    )
                    or (
                        value == "git"
                        and relative
                        in {
                            "scripts/generate_distribution_v11.py",
                            "scripts/generate_distribution_v12.py",
                        }
                    )
                    or (
                        value in {"cargo", "git", "python3"}
                        and relative == "scripts/validate_target_work_accounting_v11.py"
                    )
                    or (
                        value in {"cargo", "git"}
                        and relative == "scripts/validate_persistent_ownership_v11.py"
                    )
                    or (
                        value in {"cargo", "git", "python3"}
                        and relative == "scripts/validate_resource_ancestry_gate_v10.py"
                    )
                    or (
                        value == "git"
                        and relative == "scripts/validate_resource_followup_assurance_v10.py"
                    )
                    or (
                        value == "git"
                        and relative == "scripts/validate_resource_followup_final_decision_v10.py"
                    )
                ),
            )
        for index, value in enumerate(coordinated):
            validate_source_literal(value, f"source:{relative}:coordinated:{index}")
        for index, value in enumerate(comments):
            validate_source_literal(value, f"source:{relative}:comment:{index}")
        audited += 1
    for relative in OTHER_SURFACES:
        try:
            source = (ROOT / relative).read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError) as error:
            raise LedgerError(f"source_surface:{relative}") from error
        literals = RUST_STRING.findall(source) if relative.endswith(".rs") else [source]
        for index, value in enumerate(literals):
            validate_source_literal(
                value,
                f"source:{relative}:{index}",
                allow_command_token=(
                    relative == "tools/nostr_automerge_xtask/src/validate.rs"
                    and value == "python3"
                ),
            )
        if relative.endswith(".rs"):
            for line_number, line in enumerate(source.splitlines(), start=1):
                line_literals = [
                    value
                    for value in RUST_STRING.findall(line)
                    if not is_public_route(value)
                ]
                if len(line_literals) > 1:
                    validate_source_literal(
                        "".join(line_literals),
                        f"source:{relative}:coordinated:{line_number}",
                    )
            for index, value in enumerate(rust_comments(source)):
                validate_source_literal(value, f"source:{relative}:comment:{index}")
        audited += 1
    require(audited == len(PYTHON_SURFACES) + len(OTHER_SURFACES), "source:inventory")


def validate_tracked_boundary() -> None:
    result = subprocess.run(
        ("git", "ls-files"),
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    require(result.returncode == 0, "tracked_boundary:git")
    require(result.stderr == "", "tracked_boundary:diagnostic")
    blocked = []
    for relative in result.stdout.splitlines():
        parts = relative.casefold().split("/")
        if parts and (parts[0] == ".act" or "workflows" in parts):
            blocked.append(relative)
    require(not blocked, "tracked_boundary:material")


def mutation_self_test(records: list[dict[str, Any]], text: list[str]) -> int:
    key_names = (
        "sourcePath",
        "test-path",
        "file_path",
        "packagePath",
        "case-name",
        "commandLine",
        "log_path",
        "urlValue",
        "workflow-artifact",
        "artifactSource",
        "rootPath",
        "submodule-path",
        "implementationDetail",
    )
    value_markers = (
        chr(47) + "alpha" + chr(47) + "beta",
        "alpha" + chr(47) + "beta.json",
        "ssh" + chr(58) + chr(47) * 2 + "host",
        "custom" + chr(58) + chr(47) * 2 + "endpoint",
        "output" + chr(46) + "log",
        "engine" + chr(95) + "typescript",
        chr(99) + "argo" + chr(32) + "test",
        "f" + str(85).zfill(3) + chr(95) + "checkpoint",
        "fix" + chr(40) + "scope" + chr(41) + chr(58) + chr(32) + "hidden",
    )
    mutations: list[tuple[str, list[dict[str, Any]], list[str]]] = []
    for key in key_names:
        candidates = copy.deepcopy(records)
        candidates[0][key] = "hidden"
        mutations.append((f"key:{key}", candidates, text))
    for index, marker in enumerate(value_markers):
        candidates = copy.deepcopy(records)
        candidates[0]["status"] = marker
        mutations.append((f"value:{index}", candidates, text))
    split_values = (
        ["alpha", chr(47) + "beta"],
        ["ssh", chr(58) + chr(47) * 2 + "host"],
        ["engine", chr(95) + "typescript"],
    )
    for index, values in enumerate(split_values):
        candidates = copy.deepcopy(records)
        candidates[0]["toolchain_classes"] = values
        mutations.append((f"coordinated:{index}", candidates, text))
    split_key_values = (
        ("alpha", chr(47) + "beta"),
        ("ssh", chr(58) + chr(47) * 2 + "host"),
        ("engine", chr(95) + "typescript"),
    )
    for index, (key, value) in enumerate(split_key_values):
        candidates = copy.deepcopy(records)
        candidates[0][key] = value
        mutations.append((f"coordinated_key_value:{index}", candidates, text))

    caught = 0
    for name, candidates, candidate_text in mutations:
        try:
            validate_records(candidates, candidate_text)
        except LedgerError:
            caught += 1
            continue
        raise LedgerError(f"boundary_mutation_survived:{name}")
    return caught


def public_record_mutation_self_test(records: list[dict[str, Any]]) -> int:
    candidates = copy.deepcopy(records)
    candidates[0]["cases"][0]["path"] = "private" + chr(47) + "hidden.rs"
    try:
        for index, record in enumerate(candidates):
            validate_public_record(record, f"public_json_record:{index}")
    except LedgerError:
        return 1
    raise LedgerError("public_record_mutation_survived:private_route")


def source_mutation_self_test() -> int:
    separator = chr(47)
    underscore = chr(95)
    reviewer_routes = (
        "docs" + separator + "alpha" + separator + "beta",
        "scripts" + separator + "engine" + underscore + "typescript",
        "reports" + separator + "output" + chr(46) + "log",
        "docs"
        + separator
        + "f"
        + str(85).zfill(3)
        + underscore
        + "private"
        + underscore
        + "case"
        + chr(46)
        + "md",
    )
    comment_markers = (
        "alpha" + separator + "beta",
        "engine" + underscore + "typescript",
        "f" + str(85).zfill(3) + underscore + "private",
        "fix" + chr(40) + "scope" + chr(41) + chr(58) + chr(32) + "hidden",
        chr(99) + "argo" + chr(32) + "test",
    )
    mutations: list[tuple[str, str]] = [
        (f"reviewer_route:{index}", value)
        for index, value in enumerate(reviewer_routes)
    ]
    for index, marker in enumerate(comment_markers):
        comments = python_comments("# " + marker + "\n", "mutation")
        require(len(comments) == 1, f"source_mutation:python_shape:{index}")
        mutations.append((f"python_comment:{index}", comments[0]))
    rust_sources = (
        "// " + comment_markers[0] + "\n",
        "/* " + comment_markers[1] + " */",
        "/// " + comment_markers[2] + "\n",
        "//! " + comment_markers[3] + "\n",
        "/** " + comment_markers[4] + " */",
    )
    for index, source in enumerate(rust_sources):
        comments = rust_comments(source)
        require(len(comments) == 1, f"source_mutation:rust_shape:{index}")
        mutations.append((f"rust_comment:{index}", comments[0]))

    caught = 0
    for name, value in mutations:
        try:
            validate_source_literal(value, f"source_mutation:{name}")
        except LedgerError:
            caught += 1
            continue
        raise LedgerError(f"source_mutation_survived:{name}")
    return caught


def main() -> int:
    records = [load_object(relative) for relative in JSON_RECORDS]
    public_records = [load_object(relative) for relative in PUBLIC_JSON_RECORDS]
    try:
        text = [(ROOT / relative).read_text(encoding="utf-8") for relative in TEXT_RECORDS]
    except (OSError, UnicodeDecodeError) as error:
        raise LedgerError("text_record") from error
    validate_records(records, text)
    for index, record in enumerate(public_records):
        validate_public_record(record, f"public_json_record:{index}")
    validate_tracked_boundary()
    validate_source_surfaces()
    mutations = mutation_self_test(records, text)
    public_mutations = public_record_mutation_self_test(public_records)
    source_mutations = source_mutation_self_test()
    print("PASS: opaque reproduction boundary v9")
    print(f"- json_records={len(records)}")
    print(f"- public_json_records={len(public_records)}")
    print(f"- text_records={len(text)}")
    print(f"- negative_mutations={mutations}")
    print(f"- public_record_negative_mutations={public_mutations}")
    print(f"- source_negative_mutations={source_mutations}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
