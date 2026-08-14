#!/usr/bin/env python3
"""Fail closed on the locally completed remediation-v6 evidence set."""

from __future__ import annotations

import hashlib
import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
EXPECTED_NIP = "67019c8ea680714052c65226f620a8e1a60b9b10a8f158603063a835a7bbc7a3"


def load(path: str) -> dict:
    return json.loads((ROOT / path).read_text(encoding="utf-8"))


def sha256(path: str) -> str:
    return hashlib.sha256((ROOT / path).read_bytes()).hexdigest()


def commit_exists(value: str) -> bool:
    return subprocess.run(
        ("git", "cat-file", "-e", f"{value}^{{commit}}"),
        cwd=ROOT,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    ).returncode == 0


def main() -> int:
    closure = load("reports/remediation_v6_evidence.json")
    candidate = load("reports/final_candidate_identity_v6.json")
    attestation = load("reports/interop_typescript_v7.json")
    matrix = load("reports/requirements_coverage_v7.json")
    resource = load("reports/resource_qualification_v6.json")
    supply = load("reports/package_supply_chain_v6.json")
    review = load("reports/remediation_v6_review_packet.json")
    supersession = load("reports/evidence_supersession_v6.json")
    if closure.get("schema") != "nostr_automerge.remediation_v6_evidence.v1":
        raise AssertionError("closure schema")
    if closure.get("status") != "implementation_remediation_required":
        raise AssertionError("truthful final status")
    if set(closure.get("local_gates", {}).values()) != {"pass"}:
        raise AssertionError("local gate")
    holds = closure.get("external_holds", {})
    if set(holds) != {
        "independent_review",
        "nip_reconciliation",
        "publication",
        "rust_source_mutation_campaign",
        "sustained_fuzzing",
        "typescript_source_mutation_campaign",
    } or any(value in {"pass", "complete"} for value in holds.values()):
        raise AssertionError("external hold")
    parity = closure.get("parity", {})
    if parity.get("fixture_count") != 157 or parity.get("runs_per_implementation") != 2:
        raise AssertionError("parity")
    if candidate.get("schema") != "nostr_automerge.final_candidate_identity.v6":
        raise AssertionError("candidate schema")
    rust = candidate.get("rust", {})
    typescript = candidate.get("typescript", {})
    if rust.get("source_candidate") != matrix.get("rust_candidate"):
        raise AssertionError("rust source candidate")
    if typescript.get("implementation_candidate") != attestation.get("candidate"):
        raise AssertionError("typescript candidate")
    if not all(commit_exists(value) for value in (rust.get("source_candidate", ""), rust.get("evidence_candidate", ""))):
        raise AssertionError("rust candidate existence")
    if typescript.get("attestation_sha256") != sha256("reports/interop_typescript_v7.json"):
        raise AssertionError("typescript attestation")
    if candidate.get("authority", {}).get("nip_sha256") != EXPECTED_NIP:
        raise AssertionError("NIP changed")
    manifest = load("fixtures/distribution/manifest_v7.json")
    fixture_ids = sorted(item["fixture_id"] for item in manifest["fixtures"])
    if fixture_ids != attestation.get("executed_fixture_ids") or len(fixture_ids) != 157:
        raise AssertionError("executed TypeScript fixtures")
    if resource.get("status") != "pass" or supply.get("status") != "pass":
        raise AssertionError("resource or supply-chain status")
    if review.get("status") != "prepared_not_submitted" or review.get("external_submission") is not False:
        raise AssertionError("review boundary")
    if supersession.get("status") != "active":
        raise AssertionError("supersession status")
    for item in supersession.get("superseded", []):
        if sha256(item["path"]) != item["sha256"]:
            raise AssertionError(f"superseded hash: {item['path']}")
    if (ROOT / ".act").exists() or (ROOT / ".github").exists():
        raise AssertionError("public workflow boundary")
    print("PASS: remediation v6 local work is complete with explicit external holds")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
