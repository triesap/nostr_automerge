#!/usr/bin/env python3
"""Fail closed on the exact 129-row remediation-v7 evidence matrix."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import re
import subprocess
from pathlib import Path

from validate_requirement_matrix_v7 import signed_artifact_hash


ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/requirements_coverage_v8.json"
MUTATIONS = ROOT / "reports/requirements_evidence_mutations_v8.json"
HEX40 = re.compile(r"^[0-9a-f]{40}$")
HEX64 = re.compile(r"^[0-9a-f]{64}$")
CRITICAL = {
    "NCRDT-CLAIM-001",
    "NCRDT-CLAIM-002",
    "NCRDT-CLAIM-003",
    "NCRDT-CONTROLREF-001",
    "NCRDT-CONTROLREF-002",
    "NCRDT-FRONTIER-001",
    "NCRDT-CPCHUNK-004",
    "NCRDT-RESOURCE-005",
    "NCRDT-RESOURCE-006",
    "NCRDT-RESOURCE-007",
    "NCRDT-RESOURCE-008",
    "NCRDT-CONF-007",
    "NCRDT-EVIDENCE-003",
    "NCRDT-BRANCH-001",
    "NCRDT-BRANCH-002",
    "NCRDT-SCOPE-004",
    "NCRDT-SCOPE-005",
    "NCRDT-SCOPE-006",
    "NCRDT-RESOURCE-009",
    "NCRDT-RESOURCE-010",
    "NCRDT-CONF-008",
    "NCRDT-EVIDENCE-004",
}
GENERIC_IDS = {
    "build_immutable_evidence_corpus_through_public_api",
    "require_local_only_conformance_runner",
    "close_local_implementation_scope_without_release_overclaim",
}
V8_FIXTURES = {
    "change_references_invalid_noncanonical_child",
    "manifest_references_invalid_noncanonical_child",
    "noncanonical_child_excluded_base_head",
    "noncanonical_child_invalid_base_head",
    "noncanonical_child_pending_base_head",
    "noncanonical_grandchild_invalid_parent_epoch",
    "cross_coordinate_descriptor_reference_isolated",
    "foreign_change_references_target_control",
    "foreign_chunk_excluded_from_target_digest",
    "foreign_chunk_references_target_descriptor",
    "foreign_claim_flood_exact_budget",
    "unrelated_valid_checkpoints_exact_budget",
    "interrupted_finalization_forfeiture",
    "parent_propagation_exact_budget",
}


class EvidenceError(ValueError):
    pass


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def canonical_write(path: Path, value: object) -> None:
    path.write_text(
        json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n",
        encoding="utf-8",
    )


def validate(report: dict) -> None:
    requirements_path = ROOT / "spec/requirements.json"
    applicability_path = ROOT / "spec/requirements_applicability.json"
    distribution_path = ROOT / "fixtures/distribution/manifest_v8.json"
    requirements = json.loads(requirements_path.read_text())["requirements"]
    applicability = json.loads(applicability_path.read_text())["classifications"]
    distribution = json.loads(distribution_path.read_text())
    fixture_paths = {
        item["fixture_id"]: ROOT / item["metadata_path"]
        for item in distribution["fixtures"]
    }
    expected_hashes = {
        "requirements_sha256": sha256(requirements_path),
        "applicability_sha256": sha256(applicability_path),
        "fixture_distribution_sha256": sha256(distribution_path),
    }
    attestation_path = ROOT / "reports/interop_typescript_v8.json"
    attestation = json.loads(attestation_path.read_text())
    if report.get("schema") != "nostr_automerge.requirement_coverage.v8":
        raise EvidenceError("schema")
    if report.get("phase") not in {"rust-complete-typescript-pending", "complete"}:
        raise EvidenceError("phase")
    if any(report.get(key) != value for key, value in expected_hashes.items()):
        raise EvidenceError("authority-hash")
    rust = str(report.get("rust_candidate", ""))
    typescript = str(report.get("typescript_candidate", ""))
    if not HEX40.fullmatch(rust) or not HEX40.fullmatch(typescript):
        raise EvidenceError("candidate")
    if report.get("phase") == "complete":
        if (
            attestation.get("schema")
            != "nostr_automerge.private_typescript_attestation.v8"
            or attestation.get("result") != "pass"
            or attestation.get("candidate") != typescript
            or attestation.get("fixture_count") != 171
            or attestation.get("process_runs_per_fixture") != 2
            or attestation.get("permutations_per_fixture") != 8
            or attestation.get("fixture_distribution_sha256")
            != expected_hashes["fixture_distribution_sha256"]
            or attestation.get("requirements_sha256")
            != expected_hashes["requirements_sha256"]
            or attestation.get("canonical_report_bytes") != "identical"
            or attestation.get("deliberate_mismatch") != "detected"
            or set(attestation.get("profile_output_sha256", {}))
            != {"checkpoint", "core", "malformed", "property"}
            or any(attestation.get("boundaries", {}).values())
        ):
            raise EvidenceError("typescript-attestation")
    if subprocess.run(
        ("git", "cat-file", "-e", f"{rust}^{{commit}}"),
        cwd=ROOT,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    ).returncode:
        raise EvidenceError("rust-candidate")
    rows = report.get("rows")
    if report.get("requirement_count") != 129 or not isinstance(rows, list):
        raise EvidenceError("count")
    if [row.get("id") for row in rows] != [item["id"] for item in requirements]:
        raise EvidenceError("row-order")
    for requirement, row in zip(requirements, rows, strict=True):
        identifier = requirement["id"]
        expected_authority = {
            "source": requirement["source"],
            "section": requirement["section"],
            "text_sha256": hashlib.sha256(requirement["text"].encode()).hexdigest(),
        }
        if row.get("authority") != expected_authority:
            raise EvidenceError(f"authority:{identifier}")
        classification = applicability[identifier]
        if row.get("applicability") != classification:
            raise EvidenceError(f"applicability:{identifier}")
        if classification in {"out-of-core", "explicitly-deferred"}:
            expected = (
                "external-hold"
                if identifier in {"NCRDT-NIP-001", "NCRDT-NIP-002"}
                else "not-applicable"
            )
            if row.get("status") != expected or "rust_proof" in row:
                raise EvidenceError(f"hold:{identifier}")
            continue
        proof = row.get("rust_proof", {})
        if proof.get("candidate") != rust or proof.get("result") != "pass":
            raise EvidenceError(f"rust-proof:{identifier}")
        implementation = ROOT / str(proof.get("implementation_path", ""))
        test_path = ROOT / str(proof.get("test_path", ""))
        if not implementation.is_file() or not test_path.is_file():
            raise EvidenceError(f"rust-path:{identifier}")
        ids = proof.get("evidence_ids")
        if not isinstance(ids, list) or not ids or ids != sorted(set(ids), key=str.encode):
            raise EvidenceError(f"rust-evidence:{identifier}")
        kind = proof.get("evidence_kind")
        if kind == "signed-fixture":
            if any(item not in fixture_paths for item in ids):
                raise EvidenceError(f"fixture:{identifier}")
            expected_artifact = signed_artifact_hash(ids, fixture_paths)
        elif kind == "exact-assertion":
            source = test_path.read_text()
            if any(
                item not in source and item.rsplit("::", 1)[-1] not in source
                for item in ids
            ):
                raise EvidenceError(f"assertion:{identifier}")
            expected_artifact = sha256(test_path)
        else:
            raise EvidenceError(f"proof-kind:{identifier}")
        if proof.get("artifact_sha256") != expected_artifact or not HEX64.fullmatch(expected_artifact):
            raise EvidenceError(f"artifact:{identifier}")
        if identifier in CRITICAL and any(item in GENERIC_IDS for item in ids):
            raise EvidenceError(f"generic-critical:{identifier}")
        if identifier == "NCRDT-CONF-008" and set(ids) != V8_FIXTURES:
            raise EvidenceError("v8-conformance-coverage")
        if classification == "rust-only":
            if row.get("status") != "pass" or "typescript_proof" in row:
                raise EvidenceError(f"rust-only-status:{identifier}")
        elif report.get("phase") == "rust-complete-typescript-pending":
            if row.get("status") != "pending" or "typescript_proof" in row:
                raise EvidenceError(f"typescript-pending:{identifier}")
        else:
            opaque = row.get("typescript_proof", {})
            fixture_ids = opaque.get("fixture_ids")
            if row.get("status") != "pass" or opaque.get("candidate") != typescript:
                raise EvidenceError(f"typescript-proof:{identifier}")
            if opaque.get("result") != "pass" or not HEX40.fullmatch(
                str(opaque.get("evidence_candidate", ""))
            ):
                raise EvidenceError(f"typescript-result:{identifier}")
            if (
                not isinstance(fixture_ids, list)
                or fixture_ids != sorted(set(fixture_ids), key=str.encode)
                or any(item not in fixture_paths for item in fixture_ids)
            ):
                raise EvidenceError(f"typescript-fixture:{identifier}")
            if identifier in CRITICAL and kind == "signed-fixture" and set(ids) != set(fixture_ids):
                raise EvidenceError(f"typescript-coverage:{identifier}")
            artifact = opaque.get("artifact_sha256")
            if (
                artifact != sha256(attestation_path)
                or not isinstance(artifact, str)
                or not HEX64.fullmatch(artifact)
            ):
                raise EvidenceError(f"typescript-artifact:{identifier}")


def self_test(report: dict) -> dict:
    mutations: list[tuple[str, dict]] = []
    missing = copy.deepcopy(report)
    missing["rows"].pop()
    mutations.append(("missing_row", missing))
    reordered = copy.deepcopy(report)
    reordered["rows"][0], reordered["rows"][1] = reordered["rows"][1], reordered["rows"][0]
    mutations.append(("reordered_rows", reordered))
    authority = copy.deepcopy(report)
    authority["requirements_sha256"] = "0" * 64
    mutations.append(("authority_hash", authority))
    candidate = copy.deepcopy(report)
    candidate["rust_candidate"] = "0" * 40
    mutations.append(("stale_candidate", candidate))
    critical_index = next(
        index for index, row in enumerate(report["rows"]) if row["id"] in CRITICAL
    )
    generic = copy.deepcopy(report)
    generic["rows"][critical_index]["rust_proof"]["evidence_ids"] = [
        "build_immutable_evidence_corpus_through_public_api"
    ]
    mutations.append(("generic_critical", generic))
    artifact = copy.deepcopy(report)
    artifact["rows"][critical_index]["rust_proof"]["artifact_sha256"] = "0" * 64
    mutations.append(("artifact_hash", artifact))
    conformance_index = next(
        index for index, row in enumerate(report["rows"])
        if row["id"] == "NCRDT-CONF-008"
    )
    weakened = copy.deepcopy(report)
    weakened["rows"][conformance_index]["rust_proof"]["evidence_ids"].pop()
    mutations.append(("fixture_coverage", weakened))
    pending_index = next(
        index for index, row in enumerate(report["rows"])
        if row["applicability"] == "rust-and-typescript"
    )
    overclaim = copy.deepcopy(report)
    if report.get("phase") == "complete":
        overclaim["rows"][pending_index].pop("typescript_proof", None)
        mutations.append(("typescript_proof", overclaim))
    else:
        overclaim["rows"][pending_index]["status"] = "pass"
        mutations.append(("status_overclaim", overclaim))
    caught = []
    for name, mutation in mutations:
        try:
            validate(mutation)
        except EvidenceError as error:
            caught.append({"mutation": name, "diagnostic": str(error), "result": "caught"})
        else:
            raise AssertionError(f"requirement evidence mutation survived: {name}")
    return {
        "schema": "nostr_automerge.requirement_evidence_mutations.v8",
        "status": "pass",
        "generated": len(mutations),
        "caught": len(caught),
        "survived": 0,
        "mutations": caught,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    report = json.loads(REPORT.read_text())
    validate(report)
    if args.self_test:
        result = self_test(report)
        canonical_write(MUTATIONS, result)
        print(json.dumps(result, sort_keys=True, separators=(",", ":")))
    print(f"PASS: all 129 requirement rows are exact; phase={report['phase']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
