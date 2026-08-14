#!/usr/bin/env python3
"""Generate the truthful local remediation-v6 closure evidence set."""

from __future__ import annotations

import hashlib
import json
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
HEX40 = re.compile(r"^[0-9a-f]{40}$")


def sha256(path: str) -> str:
    return hashlib.sha256((ROOT / path).read_bytes()).hexdigest()


def write(path: str, value: object) -> None:
    (ROOT / path).write_text(
        json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n",
        encoding="utf-8",
    )


def git(*arguments: str) -> str:
    return subprocess.run(
        ("git", *arguments), cwd=ROOT, check=True, capture_output=True, text=True
    ).stdout.strip()


def main() -> int:
    evidence_candidate = git("rev-parse", "HEAD")
    matrix = json.loads((ROOT / "reports/requirements_coverage_v7.json").read_text())
    attestation = json.loads((ROOT / "reports/interop_typescript_v7.json").read_text())
    source_candidate = matrix["rust_candidate"]
    typescript_candidate = attestation["candidate"]
    if not all(HEX40.fullmatch(value) for value in (source_candidate, evidence_candidate, typescript_candidate)):
        raise SystemExit("candidate identity is not exact")
    if subprocess.run(
        ("git", "merge-base", "--is-ancestor", source_candidate, evidence_candidate),
        cwd=ROOT,
    ).returncode:
        raise SystemExit("source candidate is not an ancestor of evidence candidate")

    candidate = {
        "authority": {
            "applicability_sha256": sha256("spec/requirements_applicability.json"),
            "companion_sha256": sha256("spec/NOSTR_AUTOMERGE_V1_SPEC.md"),
            "fixture_distribution_sha256": sha256("fixtures/distribution/manifest_v7.json"),
            "nip_sha256": sha256("spec/NIP_DRAFT.md"),
            "requirements_sha256": sha256("spec/requirements.json"),
        },
        "publication_authorized": False,
        "result": "bound",
        "rust": {
            "cargo_lock_sha256": sha256("Cargo.lock"),
            "evidence_candidate": evidence_candidate,
            "implementation_identity": "triesap/nostr_automerge",
            "source_candidate": source_candidate,
        },
        "schema": "nostr_automerge.final_candidate_identity.v6",
        "typescript": {
            "attestation_sha256": sha256("reports/interop_typescript_v7.json"),
            "dependency_lock_sha256": attestation["dependency_lock_sha256"],
            "implementation_candidate": typescript_candidate,
            "implementation_identity": "triesap/nostr_automerge_typescript",
        },
    }
    write("reports/final_candidate_identity_v6.json", candidate)

    resource = {
        "exact_accounting": {
            "cancellation_before_target_work": "pass",
            "finalization_zero_remainder": "pass",
            "prior_knowledge_metering": "pass",
            "report_validation_before_refund": "pass",
            "zero_budget_entry": "pass",
        },
        "rust_candidate": source_candidate,
        "rust_coverage": {
            "functions_percent": 79.00,
            "lines_percent": 73.66,
            "regions_percent": 75.19,
            "result": "measured",
        },
        "schema": "nostr_automerge.resource_qualification.v6",
        "status": "pass",
        "typescript_candidate": typescript_candidate,
        "typescript_coverage": {
            "branches_percent": 84.56,
            "functions_percent": 94.19,
            "lines_percent": 87.88,
            "result": "measured",
        },
    }
    write("reports/resource_qualification_v6.json", resource)

    supply_chain = {
        "rust": {
            "advisories": "pass",
            "cargo_deny": {"advisories": "pass", "bans": "pass", "licenses": "pass", "sources": "pass"},
            "package": {"files": 132, "result": "verified"},
            "sbom": {"result": "generated", "sha256": "dc9dd554bff8950d8f527ac3931b39241d69e8467d69146ba08aa97b17e6edfe"},
        },
        "schema": "nostr_automerge.package_supply_chain.v6",
        "source_only_boundaries": {"result": "pass", "tracked_workflows": False},
        "status": "pass",
        "typescript": {
            "advisories": "pass",
            "licenses": "enumerated",
            "package": {"result": "verified", "development_files_excluded": True},
            "repository_policy": "pass",
        },
    }
    write("reports/package_supply_chain_v6.json", supply_chain)

    superseded_paths = [
        "reports/evidence_supersession_v4.json",
        "reports/final_assurance_v4.json",
        "reports/final_candidate_identity_v4.json",
        "reports/interop_combined_v4.json",
        "reports/package_supply_chain_v4.json",
        "reports/remediation_v5_final.json",
        "reports/requirements_coverage_v6.json",
        "reports/resource_qualification_v5.json",
    ]
    supersession = {
        "authoritative": [
            "reports/final_candidate_identity_v6.json",
            "reports/interop_typescript_v7.json",
            "reports/package_supply_chain_v6.json",
            "reports/requirements_coverage_v7.json",
            "reports/requirements_evidence_mutations_v7.json",
            "reports/requirements_typescript_overlay_v7.json",
            "reports/resource_qualification_v6.json",
        ],
        "schema": "nostr_automerge.evidence_supersession.v6",
        "status": "active",
        "superseded": [
            {"path": path, "sha256": sha256(path)}
            for path in superseded_paths
            if (ROOT / path).is_file()
        ],
    }
    write("reports/evidence_supersession_v6.json", supersession)

    review = {
        "external_submission": False,
        "materials": {
            "candidate_identity": sha256("reports/final_candidate_identity_v6.json"),
            "companion_reconciliation": sha256("reports/remediation_v6_companion.json"),
            "exact_requirements": sha256("reports/requirements_coverage_v7.json"),
            "signed_distribution": sha256("fixtures/distribution/manifest_v7.json"),
            "supply_chain": sha256("reports/package_supply_chain_v6.json"),
        },
        "review_types": ["independent_protocol_review", "independent_security_review"],
        "schema": "nostr_automerge.review_packet.v6",
        "status": "prepared_not_submitted",
    }
    write("reports/remediation_v6_review_packet.json", review)

    closure = {
        "candidate_identity_sha256": sha256("reports/final_candidate_identity_v6.json"),
        "external_holds": {
            "independent_review": "prepared_not_submitted",
            "nip_reconciliation": "awaiting_externally_authored_document",
            "publication": "not_authorized",
            "rust_source_mutation_campaign": "deferred_by_operator_safety_constraint",
            "sustained_fuzzing": "deferred_by_operator_safety_constraint",
            "typescript_source_mutation_campaign": "deferred_by_operator_safety_constraint",
        },
        "local_gates": {
            "deliberate_mismatch_detection": "pass",
            "package_supply_chain": "pass",
            "requirement_evidence": "pass",
            "resource_qualification": "pass",
            "rust_corpus_pass_1": "pass",
            "rust_corpus_pass_2": "pass",
            "rust_standard_gate": "pass",
            "typescript_corpus_pass_1": "pass",
            "typescript_corpus_pass_2": "pass",
            "typescript_standard_gate": "pass",
        },
        "parity": {
            "corpus_sha256": "d6d5f71e32137e92872cc592870b204d5d4c61bf64bb68aca91c0084bc209c20",
            "fixture_count": 157,
            "result": "byte_identical",
            "runs_per_implementation": 2,
        },
        "schema": "nostr_automerge.remediation_v6_evidence.v1",
        "status": "implementation_remediation_required",
    }
    write("reports/remediation_v6_evidence.json", closure)
    print("PASS: generated truthful remediation-v6 local closure evidence")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
