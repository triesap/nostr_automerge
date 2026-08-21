#!/usr/bin/env python3
"""Validate remediation-v8 candidate identity, evidence supersession, and holds."""

from __future__ import annotations

import hashlib
import json
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
HEX40 = re.compile(r"^[0-9a-f]{40}$")
HEX64 = re.compile(r"^[0-9a-f]{64}$")
HOLD_IDS = {
    "source_mutating_campaigns", "sustained_fuzzing", "independent_external_review",
    "production_readiness_authorization", "nip_submission_and_event_kind_allocation",
    "publication_release_deployment",
}
FORBIDDEN = ("/" + "Users/", "/" + "home/", "domains/" + "labs", "triesap/" + "dev", ".act" + "/", ".github/" + "workflows")


def load(relative: str) -> dict[str, object]:
    value = json.loads((ROOT / relative).read_text())
    if not isinstance(value, dict):
        raise AssertionError(f"object:{relative}")
    return value


def digest(relative: str) -> str:
    return hashlib.sha256((ROOT / relative).read_bytes()).hexdigest()


def ancestor(candidate: object) -> bool:
    return isinstance(candidate, str) and HEX40.fullmatch(candidate) is not None and subprocess.run(
        ("git", "merge-base", "--is-ancestor", candidate, "HEAD"), cwd=ROOT,
        check=False, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
    ).returncode == 0


def main() -> int:
    identity = load("reports/final_candidate_identity_v8.json")
    supersession = load("reports/evidence_supersession_v8.json")
    holds = load("reports/external_holds_v8.json")
    matrix = load("reports/requirements_coverage_v9.json")
    interop = load("reports/interop_combined_v9.json")
    private = load("reports/private_assurance_v9.json")
    if (
        identity.get("schema") != "nostr_automerge.final_candidate_identity.v8"
        or identity.get("result") != "bound"
        or identity.get("status") != "code_complete_publication_held"
        or identity.get("publication_authorized") is not False
    ):
        raise AssertionError("identity_status")
    authority = {
        "nip_sha256": digest("spec/NIP_DRAFT.md"),
        "companion_sha256": digest("spec/NOSTR_AUTOMERGE_V1_SPEC.md"),
        "requirements_sha256": digest("spec/requirements.json"),
        "applicability_sha256": digest("spec/requirements_applicability.json"),
        "fixture_distribution_sha256": digest("fixtures/distribution/manifest_v9.json"),
    }
    if identity.get("authority") != authority:
        raise AssertionError("authority")
    rust = identity.get("rust", {})
    if (
        rust.get("implementation_identity") != "triesap/nostr_automerge"
        or rust.get("source_candidate") != matrix["rust_candidate"]
        or not ancestor(rust.get("source_candidate"))
        or not ancestor(rust.get("evidence_base_candidate"))
        or rust.get("cargo_lock_sha256") != digest("Cargo.lock")
        or rust.get("requirement_matrix_sha256") != digest("reports/requirements_coverage_v9.json")
    ):
        raise AssertionError("rust_identity")
    typescript = identity.get("typescript", {})
    if any(typescript.get(field) != private.get(field) for field in ("implementation_identity", "implementation_candidate", "evidence_candidate", "attestation_candidate", "attestation_sha256")):
        raise AssertionError("typescript_identity")
    if any(not HEX40.fullmatch(str(typescript.get(field, ""))) for field in ("implementation_candidate", "evidence_candidate", "attestation_candidate")):
        raise AssertionError("typescript_candidate")
    conformance = identity.get("conformance", {})
    if conformance != {
        "fixture_count": 180, "permutations_per_fixture": 8,
        "process_runs_per_implementation": 2,
        "canonical_output_sha256": interop["canonical_output_sha256"],
        "canonical_report_bytes": "identical", "deliberate_mismatch": "detected",
    }:
        raise AssertionError("conformance")
    evidence = identity.get("evidence")
    if not isinstance(evidence, dict) or any(digest(path) != value for path, value in evidence.items()):
        raise AssertionError("evidence_hash")
    if supersession.get("schema") != "nostr_automerge.evidence_supersession.v8" or supersession.get("status") != "v9_authoritative":
        raise AssertionError("supersession")
    for group in ("authoritative", "superseded"):
        rows = supersession.get(group)
        if not isinstance(rows, list) or not rows or any(row.get("sha256") != digest(str(row.get("path", ""))) for row in rows):
            raise AssertionError(f"supersession_{group}")
    authoritative = {row["path"] for row in supersession["authoritative"]}
    if authoritative != set(evidence):
        raise AssertionError("authoritative_inventory")
    rows = holds.get("holds")
    if (
        holds.get("schema") != "nostr_automerge.external_holds.v8"
        or holds.get("status") != "code_complete_publication_held"
        or holds.get("remote_actions_performed") is not False
        or not isinstance(rows, list)
        or {row.get("id") for row in rows} != HOLD_IDS
        or any(row.get("executed") is not False or row.get("result_claimed") is not False for row in rows)
    ):
        raise AssertionError("holds")
    public = "\n".join((ROOT / path).read_text() for path in (
        "reports/final_candidate_identity_v8.json", "reports/evidence_supersession_v8.json",
        "reports/external_holds_v8.json", "reports/private_assurance_v9.json",
    ))
    if any(token in public for token in FORBIDDEN):
        raise AssertionError("private_material")
    if any(not HEX64.fullmatch(value) for value in evidence.values()):
        raise AssertionError("evidence_digest")
    print("PASS: final v8 identity supersedes v7 while retaining all holds")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
