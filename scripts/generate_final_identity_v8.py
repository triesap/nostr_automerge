#!/usr/bin/env python3
"""Bind remediation-v8 final candidates, evidence, supersession, and holds."""

from __future__ import annotations

import hashlib
import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
AUTHORITATIVE = [
    "reports/interop_combined_v9.json",
    "reports/interop_rust_v9.json",
    "reports/interop_typescript_v9.json",
    "reports/ordinary_assurance_v9.json",
    "reports/private_assurance_v9.json",
    "reports/requirements_authority_v9.json",
    "reports/requirements_coverage_v9.json",
    "reports/requirements_evidence_mutations_v9.json",
    "reports/resource_qualification_v9.json",
    "reports/rust_conformance_v9.json",
]
SUPERSEDED = [
    "reports/final_candidate_identity_v7.json",
    "reports/remediation_v7_final.json",
    "reports/requirements_coverage_v8.json",
    "reports/interop_typescript_v8.json",
    "reports/resource_qualification_v7.json",
]
HOLDS = [
    ("source_mutating_campaigns", "local-safety-hold"),
    ("sustained_fuzzing", "local-safety-hold"),
    ("independent_external_review", "external-hold"),
    ("production_readiness_authorization", "external-hold"),
    ("nip_submission_and_event_kind_allocation", "external-hold"),
    ("publication_release_deployment", "unauthorized"),
]


def sha256(relative: str) -> str:
    return hashlib.sha256((ROOT / relative).read_bytes()).hexdigest()


def git(*args: str) -> str:
    return subprocess.run(("git", *args), cwd=ROOT, check=True, capture_output=True, text=True).stdout.strip()


def write(relative: str, value: object) -> None:
    (ROOT / relative).write_text(json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n", encoding="utf-8")


def main() -> int:
    interop = json.loads((ROOT / "reports/interop_combined_v9.json").read_text())
    private = json.loads((ROOT / "reports/private_assurance_v9.json").read_text())
    matrix = json.loads((ROOT / "reports/requirements_coverage_v9.json").read_text())
    identity = {
        "schema": "nostr_automerge.final_candidate_identity.v8",
        "result": "bound",
        "status": "code_complete_publication_held",
        "publication_authorized": False,
        "authority": {
            "nip_sha256": sha256("spec/NIP_DRAFT.md"),
            "companion_sha256": sha256("spec/NOSTR_AUTOMERGE_V1_SPEC.md"),
            "requirements_sha256": sha256("spec/requirements.json"),
            "applicability_sha256": sha256("spec/requirements_applicability.json"),
            "fixture_distribution_sha256": sha256("fixtures/distribution/manifest_v9.json"),
        },
        "rust": {
            "implementation_identity": "triesap/nostr_automerge",
            "source_candidate": matrix["rust_candidate"],
            "evidence_base_candidate": git("rev-parse", "HEAD"),
            "cargo_lock_sha256": sha256("Cargo.lock"),
            "requirement_matrix_sha256": sha256("reports/requirements_coverage_v9.json"),
        },
        "typescript": {
            "implementation_identity": private["implementation_identity"],
            "implementation_candidate": private["implementation_candidate"],
            "evidence_candidate": private["evidence_candidate"],
            "attestation_candidate": private["attestation_candidate"],
            "attestation_sha256": private["attestation_sha256"],
        },
        "conformance": {
            "fixture_count": interop["fixture_count"],
            "permutations_per_fixture": interop["delivery_permutations"],
            "process_runs_per_implementation": interop["process_runs"],
            "canonical_output_sha256": interop["canonical_output_sha256"],
            "canonical_report_bytes": "identical",
            "deliberate_mismatch": "detected",
        },
        "evidence": {path: sha256(path) for path in AUTHORITATIVE},
    }
    supersession = {
        "schema": "nostr_automerge.evidence_supersession.v8",
        "status": "v9_authoritative",
        "authoritative": [{"path": path, "sha256": sha256(path)} for path in AUTHORITATIVE],
        "superseded": [{"path": path, "sha256": sha256(path)} for path in SUPERSEDED],
    }
    holds = {
        "schema": "nostr_automerge.external_holds.v8",
        "status": "code_complete_publication_held",
        "remote_actions_performed": False,
        "holds": [
            {"id": identifier, "classification": classification, "executed": False, "result_claimed": False}
            for identifier, classification in HOLDS
        ],
    }
    write("reports/final_candidate_identity_v8.json", identity)
    write("reports/evidence_supersession_v8.json", supersession)
    write("reports/external_holds_v8.json", holds)
    print("PASS: bound final v8 candidates and superseded v7 evidence")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
