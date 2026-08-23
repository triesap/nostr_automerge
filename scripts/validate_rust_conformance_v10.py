#!/usr/bin/env python3
"""Validate the current Rust signed-v10 distribution evidence."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import subprocess
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
REPORT_PATH = "reports/rust_conformance_v10.json"
SCHEMA_PATH = "tools/validation/rust_conformance_v10.schema.json"
MANIFEST_PATH = "fixtures/distribution/manifest_v10.json"
RUNNER_PATH = "tools/nostr_automerge_conformance/src/runner.rs"
EVIDENCE_CANDIDATE = "6e7084ae32b9d20e55e76b5496c126bd52974f0d"
FIELDS = (
    "schema",
    "status",
    "candidate",
    "manifest_sha256",
    "runner_sha256",
    "cargo_lock_sha256",
    "rust_toolchain_sha256",
    "scenario_count",
    "process_count",
    "permutations_per_fixture",
    "canonical_process_bytes",
    "canonical_output_sha256",
    "distribution_run_sha256",
    "missing_fixture_authority",
    "result_identity_sha256",
)
EXPECTED = {
    "schema": "nostr_automerge.rust_conformance.v10.v1",
    "status": "pass",
    "candidate": "20b786c5c3ff143786aaaca56ad19bd26739b67b",
    "manifest_sha256": "86ec32f34dd99ef0c1e5ea3531360a1f78bf07d62818375096e0bdf0f209b8e5",
    "runner_sha256": "e2b92358273603f48e1c2fc94c96281a5f77e89cc219b34a09b08da99a0a844e",
    "cargo_lock_sha256": "6d1b886ff74637ba6682d349ab81424b0792f2cbc61cf0f213dfcf16af4f6744",
    "rust_toolchain_sha256": "5d959dfcc98b53886ee772ba216c4f9a1b31f093b46b5b263c0d084af54e821d",
    "scenario_count": 192,
    "process_count": 2,
    "permutations_per_fixture": 8,
    "canonical_process_bytes": "identical",
    "canonical_output_sha256": "c9f28deb32dfedce674a6871b0eb949f38b5a5f977a67ca993f7ed639df1e112",
    "distribution_run_sha256": "377b0fe6ae2916b829b3ada84f7adf760d874123ce8be14130999a076c8578c6",
    "missing_fixture_authority": "rejected",
    "result_identity_sha256": "7be69317ca8f007f8b0b74f1bc355558981ba55a75bc3eb8b2b609b3590184c7",
}


class EvidenceError(ValueError):
    """One signed-v10 Rust evidence invariant failed."""


def digest(relative: str) -> str:
    return hashlib.sha256((ROOT / relative).read_bytes()).hexdigest()


def committed_bytes(candidate: str, relative: str) -> bytes:
    result = subprocess.run(
        ("git", "show", f"{candidate}:{relative}"),
        cwd=ROOT,
        check=False,
        capture_output=True,
    )
    if result.returncode != 0:
        raise EvidenceError(f"commit:{relative}")
    return result.stdout


def identity(value: dict[str, Any]) -> str:
    projection = {key: value[key] for key in FIELDS[:-1]}
    payload = json.dumps(
        projection, ensure_ascii=False, sort_keys=True, separators=(",", ":")
    ).encode()
    return hashlib.sha256(payload).hexdigest()


def validate(value: dict[str, Any]) -> None:
    if tuple(value) != FIELDS:
        raise EvidenceError("fields")
    if value != EXPECTED:
        raise EvidenceError("binding")
    if value["result_identity_sha256"] != identity(value):
        raise EvidenceError("identity")
    for relative, field in (
        (MANIFEST_PATH, "manifest_sha256"),
        ("Cargo.lock", "cargo_lock_sha256"),
        ("rust-toolchain.toml", "rust_toolchain_sha256"),
    ):
        if digest(relative) != value[field]:
            raise EvidenceError(f"hash:{field}")
    runner = committed_bytes(EVIDENCE_CANDIDATE, RUNNER_PATH)
    if hashlib.sha256(runner).hexdigest() != value["runner_sha256"]:
        raise EvidenceError("hash:runner_sha256")
    manifest = json.loads((ROOT / MANIFEST_PATH).read_text(encoding="utf-8"))
    if (
        manifest.get("distribution_schema")
        != "nostr_automerge.fixture_distribution.v10"
        or manifest.get("status") != "canonical_signed_neutral_corpus"
        or manifest.get("complete") is not True
        or manifest.get("transition_stage") != "distribution_complete"
        or manifest.get("fixture_count") != 192
        or manifest.get("target_fixture_count") != 192
        or len(manifest.get("fixtures", [])) != 192
    ):
        raise EvidenceError("manifest_authority")
    try:
        source = runner.decode("utf-8")
    except UnicodeDecodeError as error:
        raise EvidenceError("runner_encoding") from error
    for anchor, count in (
        ("fn validate_distribution_authority(", 1),
        ("distribution_authority_rejects_missing_or_incomplete_fixture_inventory", 1),
        ('"nostr_automerge.fixture_distribution.v10"', 2),
        ('Some("distribution_complete")', 1),
    ):
        if source.count(anchor) != count:
            raise EvidenceError(f"source:{anchor}")


def validate_schema() -> None:
    schema = json.loads((ROOT / SCHEMA_PATH).read_text(encoding="utf-8"))
    if (
        tuple(schema)
        != ("title", "type", "required", "properties", "additionalProperties")
        or schema.get("type") != "object"
        or schema.get("required") != list(FIELDS)
        or tuple(schema.get("properties", {})) != FIELDS
        or schema.get("additionalProperties") is not False
    ):
        raise EvidenceError("schema")


def mutation_self_test(value: dict[str, Any]) -> int:
    mutations: list[dict[str, Any]] = []
    for field, replacement in (
        ("candidate", "0" * 40),
        ("manifest_sha256", "0" * 64),
        ("runner_sha256", "0" * 64),
        ("scenario_count", 191),
        ("process_count", 1),
        ("permutations_per_fixture", 7),
        ("canonical_process_bytes", "different"),
        ("canonical_output_sha256", "0" * 64),
        ("distribution_run_sha256", "0" * 64),
        ("missing_fixture_authority", "accepted"),
        ("result_identity_sha256", "0" * 64),
    ):
        mutation = copy.deepcopy(value)
        mutation[field] = replacement
        mutations.append(mutation)
    missing = copy.deepcopy(value)
    missing.pop("status")
    mutations.append(missing)
    extra = copy.deepcopy(value)
    extra["extra"] = False
    mutations.append(extra)
    reordered = {"status": value["status"], **value}
    mutations.append(reordered)
    coordinated = copy.deepcopy(value)
    coordinated["canonical_output_sha256"] = "1" * 64
    coordinated["distribution_run_sha256"] = "2" * 64
    coordinated["result_identity_sha256"] = identity(coordinated)
    mutations.append(coordinated)
    for mutation in mutations:
        try:
            validate(mutation)
        except EvidenceError:
            continue
        raise EvidenceError("mutation_survived")
    return len(mutations)


def run_distribution_twice(value: dict[str, Any]) -> None:
    command = (
        "cargo",
        "run",
        "--quiet",
        "-p",
        "nostr_automerge_conformance",
        "--locked",
        "--",
        "run_distribution",
        MANIFEST_PATH,
    )
    first = subprocess.run(command, cwd=ROOT, check=True, capture_output=True).stdout
    second = subprocess.run(command, cwd=ROOT, check=True, capture_output=True).stdout
    if first != second or hashlib.sha256(first).hexdigest() != value["distribution_run_sha256"]:
        raise EvidenceError("process_identity")
    parsed = json.loads(first)
    if (
        parsed.get("status") != "pass"
        or parsed.get("fixture_count") != 192
        or parsed.get("delivery_permutations") != 8
        or parsed.get("canonical_output_sha256") != value["canonical_output_sha256"]
        or len(parsed.get("reports", [])) != 192
    ):
        raise EvidenceError("run_coverage")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--run", action="store_true")
    args = parser.parse_args()
    value = json.loads((ROOT / REPORT_PATH).read_text(encoding="utf-8"))
    validate_schema()
    validate(value)
    ancestor = subprocess.run(
        ("git", "merge-base", "--is-ancestor", value["candidate"], "HEAD"),
        cwd=ROOT,
        check=False,
    )
    if ancestor.returncode != 0:
        raise EvidenceError("candidate")
    evidence_ancestor = subprocess.run(
        ("git", "merge-base", "--is-ancestor", EVIDENCE_CANDIDATE, "HEAD"),
        cwd=ROOT,
        check=False,
    )
    if evidence_ancestor.returncode != 0:
        raise EvidenceError("evidence_candidate")
    mutations = mutation_self_test(value)
    if args.run:
        run_distribution_twice(value)
    print("PASS: Rust signed-v10 conformance evidence")
    print("- scenarios=192")
    print("- delivery_permutations=8")
    print("- processes=2")
    print(f"- negative_mutations={mutations}")
    print(f"- executed={int(args.run)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
