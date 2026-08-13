#!/usr/bin/env python3
"""Fail closed on remediation-v5 106-row requirement evidence."""

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


class EvidenceError(ValueError):
    pass


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(path: str) -> dict:
    value = json.loads((ROOT / path).read_text())
    if not isinstance(value, dict):
        raise EvidenceError("object-shape")
    return value


def validate(report: dict) -> None:
    requirements_path = ROOT / "spec/requirements.json"
    applicability_path = ROOT / "spec/requirements_applicability.json"
    distribution_path = ROOT / "fixtures/distribution/manifest_v6.json"
    requirements = json.loads(requirements_path.read_text())["requirements"]
    applicability = json.loads(applicability_path.read_text())["classifications"]
    distribution = json.loads(distribution_path.read_text())
    fixtures = {item["fixture_id"] for item in distribution["fixtures"]}
    expected_hashes = {
        "requirements_sha256": sha256(requirements_path),
        "applicability_sha256": sha256(applicability_path),
        "fixture_distribution_sha256": sha256(distribution_path),
    }
    if report.get("schema") != "nostr_automerge.requirement_coverage.v6":
        raise EvidenceError("schema")
    if any(report.get(key) != value or not HEX64.fullmatch(value) for key, value in expected_hashes.items()):
        raise EvidenceError("authority-hash")
    if not HEX64.fullmatch(str(report.get("corpus_sha256", ""))):
        raise EvidenceError("corpus-hash")
    rust = report.get("rust_candidate")
    typescript = report.get("typescript_candidate")
    if not HEX40.fullmatch(str(rust)) or not HEX40.fullmatch(str(typescript)):
        raise EvidenceError("candidate")
    if subprocess.run(
        ("git", "cat-file", "-e", f"{rust}^{{commit}}"),
        cwd=ROOT,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    ).returncode:
        raise EvidenceError("rust-candidate")
    rows = report.get("rows")
    if report.get("requirement_count") != 106 or not isinstance(rows, list):
        raise EvidenceError("count")
    if [row.get("id") for row in rows] != [item["id"] for item in requirements]:
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
        if not (ROOT / authority["source"]).is_file():
            raise EvidenceError(f"source:{identifier}")
        if applicability[identifier] in {"out-of-core", "explicitly-deferred"}:
            expected = "external-hold" if identifier == "NCRDT-NIP-001" else "not-applicable"
            if row.get("status") != expected or "rust_proof" in row or "typescript_proof" in row:
                raise EvidenceError(f"hold:{identifier}")
            continue
        proof = row.get("rust_proof", {})
        if row.get("status") != "pass" or proof.get("candidate") != rust or proof.get("result") != "pass":
            raise EvidenceError(f"rust-proof:{identifier}")
        for key in ("implementation_path", "test_path"):
            if not (ROOT / proof.get(key, "")).is_file():
                raise EvidenceError(f"rust-path:{identifier}")
        ids = proof.get("evidence_ids")
        if not isinstance(ids, list) or not ids or any(item not in fixtures for item in ids if proof.get("evidence_kind") == "signed-fixture"):
            raise EvidenceError(f"rust-evidence:{identifier}")
        if applicability[identifier] == "rust-and-typescript":
            opaque = row.get("typescript_proof", {})
            if opaque.get("candidate") != typescript or opaque.get("result") != "pass":
                raise EvidenceError(f"typescript-proof:{identifier}")
            if any(item not in fixtures for item in opaque.get("fixture_ids", [])):
                raise EvidenceError(f"typescript-fixture:{identifier}")
        elif "typescript_proof" in row:
            raise EvidenceError(f"typescript-scope:{identifier}")


def self_test(report: dict) -> None:
    mutations = []
    missing = copy.deepcopy(report); missing["rows"].pop(); mutations.append(missing)
    reordered = copy.deepcopy(report); reordered["rows"][0], reordered["rows"][1] = reordered["rows"][1], reordered["rows"][0]; mutations.append(reordered)
    authority = copy.deepcopy(report); authority["requirements_sha256"] = "0" * 64; mutations.append(authority)
    candidate = copy.deepcopy(report); candidate["rust_candidate"] = "0" * 40; mutations.append(candidate)
    result = copy.deepcopy(report); next(row for row in result["rows"] if row.get("rust_proof"))["status"] = "held"; mutations.append(result)
    hold = copy.deepcopy(report); next(row for row in hold["rows"] if row["id"] == "NCRDT-NIP-001")["status"] = "pass"; mutations.append(hold)
    caught = 0
    for mutation in mutations:
        try:
            validate(mutation)
        except EvidenceError:
            caught += 1
        else:
            raise AssertionError("requirement evidence mutation survived")
    print(f"PASS: caught {caught}/{len(mutations)} requirement evidence mutations")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    report = load("reports/requirements_coverage_v6.json")
    validate(report)
    if args.self_test:
        self_test(report)
    print("PASS: all 106 remediation-v5 requirement rows are exact")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
