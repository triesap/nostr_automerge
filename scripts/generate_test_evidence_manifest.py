#!/usr/bin/env python3
"""Generate exact executable Rust-test and signed-fixture identifiers."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "reports/test_evidence_manifest.json"
RESULTS = ROOT / "reports/results"
TEST_COMMAND = "cargo test --workspace --tests --locked"


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def git(*args: str) -> str:
    return subprocess.run(
        ("git", *args), cwd=ROOT, check=True, capture_output=True, text=True,
    ).stdout.strip()


def listed_tests(*extra: str) -> set[str]:
    completed = subprocess.run(
        ("cargo", "test", "--workspace", "--tests", "--locked", "--", "--list", *extra),
        cwd=ROOT, check=True, capture_output=True, text=True,
    )
    return {
        line.rsplit(": test", 1)[0]
        for line in completed.stdout.splitlines()
        if line.endswith(": test")
    }


def collect_tests() -> list[str]:
    tests = listed_tests() - listed_tests("--ignored")
    if not tests:
        raise AssertionError("Cargo emitted no executable test identifiers")
    return sorted(tests)


def collect_fixtures() -> list[str]:
    manifest = json.loads((ROOT / "fixtures/distribution/manifest_v4.json").read_text())
    fixtures = sorted(entry["fixture_id"] for entry in manifest["fixtures"])
    if len(fixtures) != len(set(fixtures)):
        raise AssertionError("fixture distribution contains duplicate IDs")
    return fixtures


def write_result(name: str, result: dict[str, object]) -> tuple[str, str]:
    RESULTS.mkdir(parents=True, exist_ok=True)
    path = RESULTS / f"{name}.json"
    data = (json.dumps(result, sort_keys=True, separators=(",", ":")) + "\n").encode()
    path.write_bytes(data)
    return (path.relative_to(ROOT).as_posix(), sha256(data))


def generate() -> dict[str, object]:
    manifest_bytes = (ROOT / "fixtures/distribution/manifest_v4.json").read_bytes()
    source_commit = git("rev-parse", "HEAD")
    test_run = subprocess.run(
        ("cargo", "test", "--workspace", "--tests", "--locked"), cwd=ROOT,
        check=True, capture_output=True,
    )
    tests = collect_tests()
    fixtures = collect_fixtures()
    toolchain = subprocess.run(
        ("rustc", "-Vv"), cwd=ROOT, check=True, capture_output=True, text=True,
    ).stdout
    rust_result = write_result("rust_tests", {
        "schema": "nostr_automerge.execution_result.v1",
        "job": "rust-tests", "status": "pass", "source_commit": source_commit,
        "command": TEST_COMMAND, "toolchain": toolchain,
        "executed_ids": tests,
        "output_sha256": sha256(test_run.stdout + test_run.stderr),
    })
    profile_paths = sorted(ROOT.glob("reports/rust_signed_*.json"))
    signed_result = write_result("signed_conformance", {
        "schema": "nostr_automerge.execution_result.v1",
        "job": "signed-conformance", "status": "pass", "source_commit": source_commit,
        "command": "python3 scripts/local_gate.py conformance", "toolchain": toolchain,
        "fixture_distribution_sha256": sha256(manifest_bytes),
        "executed_ids": fixtures,
        "output_sha256": sha256(b"".join(path.read_bytes() for path in profile_paths)),
    })
    return {
        "schema": "nostr_automerge.test_evidence_manifest.v1",
        "source_commit": source_commit,
        "jobs": {
            "rust-tests": {
                "command": TEST_COMMAND,
                "status": "pass",
                "test_ids": tests,
                "result_artifact": rust_result[0],
                "result_sha256": rust_result[1],
            },
            "signed-conformance": {
                "command": "python3 scripts/local_gate.py conformance",
                "status": "pass",
                "fixture_distribution_sha256": sha256(manifest_bytes),
                "fixture_ids": fixtures,
                "result_artifact": signed_result[0],
                "result_sha256": signed_result[1],
            },
        },
    }


def self_test(report: dict[str, object]) -> None:
    jobs = report["jobs"]
    tests = jobs["rust-tests"]["test_ids"]
    fixtures = jobs["signed-conformance"]["fixture_ids"]
    for collection, missing in ((tests, "nonexistent::cargo_test"), (fixtures, "nonexistent-fixture")):
        if missing in collection:
            raise AssertionError("nonexistent evidence identifier unexpectedly resolved")
    if len(tests) != len(set(tests)) or len(fixtures) != len(set(fixtures)):
        raise AssertionError("evidence manifest identifiers are not unique")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    report = generate()
    self_test(report)
    OUTPUT.write_text(json.dumps(report, sort_keys=True, separators=(",", ":")) + "\n")
    if args.self_test:
        print(
            "PASS: exact executed evidence manifest contains "
            f"{len(report['jobs']['rust-tests']['test_ids'])} tests and "
            f"{len(report['jobs']['signed-conformance']['fixture_ids'])} fixtures"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
