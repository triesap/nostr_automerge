#!/usr/bin/env python3
"""Fail closed on exact 139-row signed-v9 requirement evidence."""

from __future__ import annotations

import copy
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

sys.dont_write_bytecode = True

from validate_requirement_matrix_v7 import git_bytes, signed_artifact_hash_at_commit


ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/requirements_coverage_v9.json"
MUTATIONS = ROOT / "reports/requirements_evidence_mutations_v9.json"
HEX40 = re.compile(r"^[0-9a-f]{40}$")
HEX64 = re.compile(r"^[0-9a-f]{64}$")
CRITICAL = {
    "NCRDT-CLAIM-001", "NCRDT-CLAIM-002", "NCRDT-CLAIM-003",
    "NCRDT-CONTROLREF-001", "NCRDT-CONTROLREF-002", "NCRDT-FRONTIER-001",
    "NCRDT-CPCHUNK-004", "NCRDT-RESOURCE-005", "NCRDT-RESOURCE-006",
    "NCRDT-RESOURCE-007", "NCRDT-RESOURCE-008", "NCRDT-CONF-007",
    "NCRDT-EVIDENCE-003", "NCRDT-BRANCH-001", "NCRDT-BRANCH-002",
    "NCRDT-SCOPE-004", "NCRDT-SCOPE-005", "NCRDT-SCOPE-006",
    "NCRDT-RESOURCE-009", "NCRDT-RESOURCE-010", "NCRDT-CONF-008",
    "NCRDT-EVIDENCE-004", "NCRDT-BRANCH-003", "NCRDT-BRANCH-004",
    "NCRDT-SCOPE-007", "NCRDT-RESOURCE-011", "NCRDT-RESOURCE-012",
    "NCRDT-DISPOSITION-004", "NCRDT-DISPOSITION-005", "NCRDT-NIP-003",
    "NCRDT-CONF-009", "NCRDT-EVIDENCE-005",
}
GENERIC_IDS = {
    "build_immutable_evidence_corpus_through_public_api",
    "require_local_only_conformance_runner",
    "close_local_implementation_scope_without_release_overclaim",
}
TS_COMMANDS = [
    "complete pinned package check",
    "signed distribution v9 execution in two independent processes",
    "all eight delivery permutations per fixture",
    "byte-exact comparison and deliberate mismatch rejection",
]


class EvidenceError(ValueError):
    """One requirement evidence invariant failed."""


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def validate(report: dict[str, object]) -> None:
    rust = str(report.get("rust_candidate", ""))
    typescript = str(report.get("typescript_candidate", ""))
    if not HEX40.fullmatch(rust) or not HEX40.fullmatch(typescript):
        raise EvidenceError("candidate")
    if subprocess.run(
        ("git", "merge-base", "--is-ancestor", rust, "HEAD"), cwd=ROOT,
        check=False, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
    ).returncode:
        raise EvidenceError("rust-candidate")
    requirements_bytes = git_bytes(rust, "spec/requirements.json")
    applicability_bytes = git_bytes(rust, "spec/requirements_applicability.json")
    distribution_bytes = git_bytes(rust, "fixtures/distribution/manifest_v9.json")
    requirements = json.loads(requirements_bytes)["requirements"]
    applicability = json.loads(applicability_bytes)["classifications"]
    distribution = json.loads(distribution_bytes)
    fixture_paths = {item["fixture_id"]: item["metadata_path"] for item in distribution["fixtures"]}
    all_fixtures = sorted(fixture_paths, key=str.encode)
    attestation_path = ROOT / "reports/interop_typescript_v9.json"
    attestation = json.loads(attestation_path.read_text())
    if (
        report.get("schema") != "nostr_automerge.requirement_coverage.v9"
        or report.get("phase") != "complete"
        or report.get("requirement_count") != 139
        or report.get("requirements_sha256") != sha256_bytes(requirements_bytes)
        or report.get("applicability_sha256") != sha256_bytes(applicability_bytes)
        or report.get("fixture_distribution_sha256") != sha256_bytes(distribution_bytes)
        or report.get("typescript_candidate") != attestation.get("commit")
    ):
        raise EvidenceError("report-binding")
    rows = report.get("rows")
    if not isinstance(rows, list) or [row.get("id") for row in rows] != [item["id"] for item in requirements]:
        raise EvidenceError("row-order")
    for requirement, row in zip(requirements, rows, strict=True):
        identifier = requirement["id"]
        authority = {
            "source": requirement["source"],
            "section": requirement["section"],
            "text_sha256": hashlib.sha256(requirement["text"].encode()).hexdigest(),
        }
        if row.get("authority") != authority or row.get("applicability") != applicability[identifier]:
            raise EvidenceError(f"authority:{identifier}")
        classification = applicability[identifier]
        if classification in {"out-of-core", "explicitly-deferred"}:
            if row.get("status") != "not-applicable" or "rust_proof" in row or "typescript_proof" in row:
                raise EvidenceError(f"false-hold-claim:{identifier}")
            continue
        proof = row.get("rust_proof")
        if not isinstance(proof, dict) or proof.get("candidate") != rust or proof.get("result") != "pass":
            raise EvidenceError(f"rust-proof:{identifier}")
        implementation_path = str(proof.get("implementation_path", ""))
        test_path = str(proof.get("test_path", ""))
        if not git_bytes(rust, implementation_path) or not git_bytes(rust, test_path):
            raise EvidenceError(f"candidate-path:{identifier}")
        evidence_ids = proof.get("evidence_ids")
        if not isinstance(evidence_ids, list) or not evidence_ids or evidence_ids != sorted(set(evidence_ids), key=str.encode):
            raise EvidenceError(f"ordered-evidence:{identifier}")
        if identifier in CRITICAL and GENERIC_IDS.intersection(evidence_ids):
            raise EvidenceError(f"generic-critical:{identifier}")
        if proof.get("evidence_kind") == "signed-fixture":
            if any(item not in fixture_paths for item in evidence_ids):
                raise EvidenceError(f"fixture:{identifier}")
            artifact = signed_artifact_hash_at_commit(rust, evidence_ids, fixture_paths)
            expected_command = "cargo run -p nostr_automerge_conformance --locked -- run_distribution fixtures/distribution/manifest_v9.json"
        elif proof.get("evidence_kind") == "exact-assertion":
            source = git_bytes(rust, test_path).decode()
            if any(item not in source and item.rsplit("::", 1)[-1] not in source for item in evidence_ids):
                raise EvidenceError(f"assertion:{identifier}")
            artifact = sha256_bytes(git_bytes(rust, test_path))
            expected_command = "cargo test --workspace --all-targets --locked"
        else:
            raise EvidenceError(f"proof-kind:{identifier}")
        if proof.get("artifact_sha256") != artifact or not HEX64.fullmatch(artifact):
            raise EvidenceError(f"artifact:{identifier}")
        if proof.get("command") != expected_command:
            raise EvidenceError(f"weakened-command:{identifier}")
        if identifier == "NCRDT-CONF-009" and evidence_ids != all_fixtures:
            raise EvidenceError("complete-v9-fixtures")
        if classification == "rust-only":
            if row.get("status") != "pass" or "typescript_proof" in row:
                raise EvidenceError(f"rust-only:{identifier}")
            continue
        opaque = row.get("typescript_proof")
        if not isinstance(opaque, dict):
            raise EvidenceError(f"typescript-proof:{identifier}")
        fixture_ids = opaque.get("fixture_ids")
        if (
            row.get("status") != "pass"
            or opaque.get("implementation_identity") != "triesap/nostr_automerge_typescript"
            or opaque.get("candidate") != typescript
            or opaque.get("evidence_candidate") != attestation.get("evidence_commit")
            or opaque.get("dependency_lock_sha256") != attestation.get("dependency_lock_sha256")
            or opaque.get("commands") != TS_COMMANDS
            or opaque.get("result") != "pass"
            or opaque.get("artifact_sha256") != hashlib.sha256(attestation_path.read_bytes()).hexdigest()
            or not isinstance(fixture_ids, list)
            or fixture_ids != sorted(set(fixture_ids), key=str.encode)
            or any(item not in fixture_paths for item in fixture_ids)
        ):
            raise EvidenceError(f"typescript-binding:{identifier}")
        if identifier in CRITICAL and proof.get("evidence_kind") == "signed-fixture" and fixture_ids != evidence_ids:
            raise EvidenceError(f"typescript-coverage:{identifier}")
        if identifier == "NCRDT-CONF-009" and fixture_ids != all_fixtures:
            raise EvidenceError("typescript-complete-v9-fixtures")


def self_test(report: dict[str, object]) -> dict[str, object]:
    mutations: list[tuple[str, dict[str, object]]] = []
    missing = copy.deepcopy(report); missing["rows"].pop(); mutations.append(("missing_row", missing))
    stale_hash = copy.deepcopy(report); stale_hash["requirements_sha256"] = "0" * 64; mutations.append(("stale_hash", stale_hash))
    wrong_candidate = copy.deepcopy(report); wrong_candidate["rust_candidate"] = "0" * 40; mutations.append(("wrong_candidate", wrong_candidate))
    critical_index = next(index for index, row in enumerate(report["rows"]) if row["id"] in CRITICAL and row["rust_proof"]["evidence_kind"] == "signed-fixture")
    generic = copy.deepcopy(report); generic["rows"][critical_index]["rust_proof"]["evidence_ids"] = [next(iter(GENERIC_IDS))]; mutations.append(("generic_proof", generic))
    command = copy.deepcopy(report); command["rows"][critical_index]["rust_proof"]["command"] = "broad test"; mutations.append(("weakened_command", command))
    conformance_index = next(index for index, row in enumerate(report["rows"]) if row["id"] == "NCRDT-CONF-009")
    reordered = copy.deepcopy(report); reordered["rows"][conformance_index]["rust_proof"]["evidence_ids"].reverse(); mutations.append(("reordered_fixtures", reordered))
    held_index = next(index for index, row in enumerate(report["rows"]) if row["status"] == "not-applicable")
    false_hold = copy.deepcopy(report); false_hold["rows"][held_index]["status"] = "pass"; mutations.append(("false_hold_claim", false_hold))
    incomplete = copy.deepcopy(report); incomplete["rows"][conformance_index]["rust_proof"]["evidence_ids"].pop(); mutations.append(("incomplete_distribution", incomplete))
    caught = []
    for name, mutation in mutations:
        try:
            validate(mutation)
        except (EvidenceError, ValueError) as error:
            caught.append({"mutation": name, "diagnostic": str(error), "result": "caught"})
        else:
            raise AssertionError(f"requirement mutation survived: {name}")
    return {
        "schema": "nostr_automerge.requirement_evidence_mutations.v9",
        "status": "pass", "generated": len(caught), "caught": len(caught), "survived": 0,
        "mutations": caught,
    }


def main() -> int:
    report = json.loads(REPORT.read_text())
    validate(report)
    mutations = self_test(report)
    MUTATIONS.write_text(json.dumps(mutations, sort_keys=True, separators=(",", ":")) + "\n", encoding="utf-8")
    print("PASS: all 139 requirement rows and 8 fail-closed mutations are exact")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
