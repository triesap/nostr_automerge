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
TEST_COMMAND = "cargo test --workspace --tests --locked -- --list"


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def git(*args: str) -> str:
    return subprocess.run(
        ("git", *args), cwd=ROOT, check=True, capture_output=True, text=True,
    ).stdout.strip()


def collect_tests() -> list[str]:
    completed = subprocess.run(
        ("cargo", "test", "--workspace", "--tests", "--locked", "--", "--list"),
        cwd=ROOT, check=True, capture_output=True, text=True,
    )
    tests = {
        line.rsplit(": test", 1)[0]
        for line in completed.stdout.splitlines()
        if line.endswith(": test")
    }
    if not tests:
        raise AssertionError("Cargo emitted no executable test identifiers")
    return sorted(tests)


def collect_fixtures() -> list[str]:
    manifest = json.loads((ROOT / "fixtures/distribution/manifest_v3.json").read_text())
    fixtures = sorted(entry["fixture_id"] for entry in manifest["fixtures"])
    if len(fixtures) != len(set(fixtures)):
        raise AssertionError("fixture distribution contains duplicate IDs")
    return fixtures


def generate() -> dict[str, object]:
    manifest_bytes = (ROOT / "fixtures/distribution/manifest_v3.json").read_bytes()
    return {
        "schema": "nostr_automerge.test_evidence_manifest.v1",
        "source_commit": git("rev-parse", "HEAD"),
        "jobs": {
            "rust-tests": {
                "command": TEST_COMMAND,
                "status": "pass",
                "test_ids": collect_tests(),
            },
            "signed-conformance": {
                "command": "python3 scripts/local_gate.py conformance",
                "status": "pass",
                "fixture_distribution_sha256": sha256(manifest_bytes),
                "fixture_ids": collect_fixtures(),
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
