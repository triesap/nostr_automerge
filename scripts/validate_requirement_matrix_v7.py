#!/usr/bin/env python3
"""Fail closed on remediation-v6 exact requirement evidence."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
HEX40 = re.compile(r"^[0-9a-f]{40}$")
HEX64 = re.compile(r"^[0-9a-f]{64}$")
CRITICAL = {
    "NCRDT-CLAIM-001", "NCRDT-CLAIM-002", "NCRDT-CLAIM-003",
    "NCRDT-CONTROLREF-001", "NCRDT-CONTROLREF-002", "NCRDT-FRONTIER-001",
    "NCRDT-CPCHUNK-004", "NCRDT-RESOURCE-005", "NCRDT-RESOURCE-006",
    "NCRDT-RESOURCE-007", "NCRDT-RESOURCE-008", "NCRDT-CONF-007",
    "NCRDT-EVIDENCE-003",
}
GENERIC_IDS = {
    "build_immutable_evidence_corpus_through_public_api",
    "require_local_only_conformance_runner",
    "close_local_implementation_scope_without_release_overclaim",
}


class EvidenceError(ValueError):
    pass


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def signed_artifact_hash(fixture_ids: list[str], fixture_paths: dict[str, Path]) -> str:
    digest = hashlib.sha256()
    for fixture_id in fixture_ids:
        path = fixture_paths[fixture_id]
        relative = path.relative_to(ROOT).as_posix().encode()
        data = path.read_bytes()
        digest.update(len(relative).to_bytes(4, "big") + relative)
        digest.update(len(data).to_bytes(8, "big") + data)
    return digest.hexdigest()


def validate(report: dict) -> None:
    requirements_path = ROOT / "spec/requirements.json"
    applicability_path = ROOT / "spec/requirements_applicability.json"
    distribution_path = ROOT / "fixtures/distribution/manifest_v7.json"
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
    if report.get("schema") != "nostr_automerge.requirement_coverage.v7":
        raise EvidenceError("schema")
    if any(report.get(key) != value for key, value in expected_hashes.items()):
        raise EvidenceError("authority-hash")
    rust = str(report.get("rust_candidate", ""))
    typescript = str(report.get("typescript_candidate", ""))
    if not HEX40.fullmatch(rust) or not HEX40.fullmatch(typescript):
        raise EvidenceError("candidate")
    if subprocess.run(
        ("git", "cat-file", "-e", f"{rust}^{{commit}}"), cwd=ROOT,
        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
    ).returncode:
        raise EvidenceError("rust-candidate")
    rows = report.get("rows")
    if report.get("requirement_count") != 119 or not isinstance(rows, list):
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
            expected = "external-hold" if identifier == "NCRDT-NIP-001" else "not-applicable"
            if row.get("status") != expected or "rust_proof" in row:
                raise EvidenceError(f"hold:{identifier}")
            continue
        proof = row.get("rust_proof", {})
        if row.get("status") != "pass" or proof.get("candidate") != rust or proof.get("result") != "pass":
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
            if any(item not in source for item in ids):
                raise EvidenceError(f"assertion:{identifier}")
            expected_artifact = sha256(test_path)
        else:
            raise EvidenceError(f"proof-kind:{identifier}")
        if proof.get("artifact_sha256") != expected_artifact or not HEX64.fullmatch(expected_artifact):
            raise EvidenceError(f"artifact:{identifier}")
        if identifier in CRITICAL and any(item in GENERIC_IDS for item in ids):
            raise EvidenceError(f"generic-critical:{identifier}")
        if classification == "rust-and-typescript":
            opaque = row.get("typescript_proof", {})
            fixture_ids = opaque.get("fixture_ids")
            if opaque.get("candidate") != typescript or opaque.get("result") != "pass":
                raise EvidenceError(f"typescript-proof:{identifier}")
            if not isinstance(fixture_ids, list) or any(item not in fixture_paths for item in fixture_ids):
                raise EvidenceError(f"typescript-fixture:{identifier}")
            if identifier in CRITICAL and not fixture_ids:
                raise EvidenceError(f"typescript-exact:{identifier}")
            artifact = opaque.get("artifact_sha256")
            if not isinstance(artifact, str) or not HEX64.fullmatch(artifact):
                raise EvidenceError(f"typescript-artifact:{identifier}")
        elif "typescript_proof" in row:
            raise EvidenceError(f"typescript-scope:{identifier}")


def self_test(report: dict) -> dict:
    mutations: list[tuple[str, dict]] = []
    missing = copy.deepcopy(report); missing["rows"].pop(); mutations.append(("missing_row", missing))
    reordered = copy.deepcopy(report); reordered["rows"][0], reordered["rows"][1] = reordered["rows"][1], reordered["rows"][0]; mutations.append(("reordered_rows", reordered))
    authority = copy.deepcopy(report); authority["requirements_sha256"] = "0" * 64; mutations.append(("authority_hash", authority))
    candidate = copy.deepcopy(report); candidate["rust_candidate"] = "0" * 40; mutations.append(("rust_candidate", candidate))
    critical = next(index for index, row in enumerate(report["rows"]) if row["id"] in CRITICAL)
    generic = copy.deepcopy(report); generic["rows"][critical]["rust_proof"]["evidence_ids"] = ["build_immutable_evidence_corpus_through_public_api"]; mutations.append(("generic_proof", generic))
    artifact = copy.deepcopy(report); artifact["rows"][critical]["rust_proof"]["artifact_sha256"] = "0" * 64; mutations.append(("artifact_hash", artifact))
    ts_row = next(index for index, row in enumerate(report["rows"]) if row["id"] in CRITICAL and "typescript_proof" in row)
    typescript = copy.deepcopy(report); typescript["rows"][ts_row]["typescript_proof"]["fixture_ids"] = []; mutations.append(("typescript_fixture", typescript))
    caught = []
    for name, mutation in mutations:
        try:
            validate(mutation)
        except EvidenceError as error:
            caught.append({"mutation": name, "diagnostic": str(error), "result": "caught"})
        else:
            raise AssertionError(f"requirement evidence mutation survived: {name}")
    return {
        "schema": "nostr_automerge.requirement_evidence_mutations.v7",
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
    report = json.loads((ROOT / "reports/requirements_coverage_v7.json").read_text())
    validate(report)
    if args.self_test:
        result = self_test(report)
        print(json.dumps(result, sort_keys=True, separators=(",", ":")))
    print("PASS: all 119 remediation-v6 requirement rows are exact")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
