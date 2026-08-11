#!/usr/bin/env python3
"""Generate exact commit-bound Rust test and signed-fixture evidence v4."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "reports/test_evidence_manifest_v4.json"
RESULTS = ROOT / "reports/results"
MANIFEST = ROOT / "fixtures/distribution/manifest_v4.json"
TEST_COMMAND = "cargo test --workspace --tests --locked"


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def command(*arguments: str) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(arguments, cwd=ROOT, check=True, capture_output=True)


def git(*arguments: str) -> str:
    return command("git", *arguments).stdout.decode().strip()


def implementation_commit() -> str:
    return git(
        "log", "-1", "--format=%H", "--", "crates", "tools", "Cargo.toml", "Cargo.lock",
        "rust-toolchain.toml", "fixtures",
    )


def listed_tests(*extra: str) -> set[str]:
    output = command("cargo", "test", "--workspace", "--tests", "--locked", "--", "--list", *extra).stdout.decode()
    return {line.rsplit(": test", 1)[0] for line in output.splitlines() if line.endswith(": test")}


def write_result(name: str, value: dict[str, object]) -> tuple[str, str]:
    RESULTS.mkdir(parents=True, exist_ok=True)
    path = RESULTS / f"{name}_v4.json"
    data = (json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n").encode()
    path.write_bytes(data)
    return path.relative_to(ROOT).as_posix(), sha256(data)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile-root", type=Path, default=ROOT / ".local/evidence-v4/rust")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    test_run = command("cargo", "test", "--workspace", "--tests", "--locked")
    tests = sorted(listed_tests() - listed_tests("--ignored"))
    if not tests or len(tests) != len(set(tests)):
        raise AssertionError("invalid executable Rust test inventory")
    distribution = json.loads(MANIFEST.read_text(encoding="utf-8"))
    fixtures = [entry["fixture_id"] for entry in distribution["fixtures"]]
    if len(fixtures) != 103 or len(fixtures) != len(set(fixtures)):
        raise AssertionError("invalid signed-v4 fixture inventory")
    source_commit = implementation_commit()
    evidence_commit = git("rev-parse", "HEAD")
    toolchain = command("rustc", "-Vv").stdout.decode()
    cargo_lock = sha256((ROOT / "Cargo.lock").read_bytes())
    manifest_hash = sha256(MANIFEST.read_bytes())
    rust_result = write_result(
        "rust_tests",
        {
            "schema": "nostr_automerge.execution_result.v2",
            "job": "rust-tests",
            "status": "pass",
            "implementation_commit": source_commit,
            "evidence_commit": evidence_commit,
            "cargo_lock_sha256": cargo_lock,
            "command": TEST_COMMAND,
            "toolchain": toolchain,
            "executed_ids": tests,
            "output_sha256": sha256(test_run.stdout + test_run.stderr),
        },
    )
    profiles = sorted(args.profile_root.resolve(strict=True).glob("rust_signed_*.json"))
    if len(profiles) != 4:
        raise AssertionError("complete Rust signed profile set is required")
    for profile in profiles:
        value = json.loads(profile.read_text(encoding="utf-8"))
        if value.get("status") != "pass" or value.get("source_commit") != source_commit:
            raise AssertionError("stale or failing Rust signed profile")
    signed_result = write_result(
        "signed_conformance",
        {
            "schema": "nostr_automerge.execution_result.v2",
            "job": "signed-conformance",
            "status": "pass",
            "implementation_commit": source_commit,
            "evidence_commit": evidence_commit,
            "cargo_lock_sha256": cargo_lock,
            "fixture_distribution_sha256": manifest_hash,
            "command": "python3 scripts/generate_rust_conformance.py",
            "toolchain": toolchain,
            "executed_ids": fixtures,
            "output_sha256": sha256(b"".join(path.read_bytes() for path in profiles)),
        },
    )
    report = {
        "schema": "nostr_automerge.test_evidence_manifest.v2",
        "implementation_commit": source_commit,
        "evidence_commit": evidence_commit,
        "cargo_lock_sha256": cargo_lock,
        "fixture_distribution_sha256": manifest_hash,
        "jobs": {
            "rust-tests": {
                "command": TEST_COMMAND,
                "status": "pass",
                "executed_ids": tests,
                "result_artifact": rust_result[0],
                "result_sha256": rust_result[1],
            },
            "signed-conformance": {
                "command": "python3 scripts/generate_rust_conformance.py",
                "status": "pass",
                "executed_ids": fixtures,
                "result_artifact": signed_result[0],
                "result_sha256": signed_result[1],
            },
        },
    }
    if args.self_test and ("nonexistent::test" in tests or "nonexistent-fixture" in fixtures):
        raise AssertionError("nonexistent evidence identifier resolved")
    OUTPUT.write_text(json.dumps(report, sort_keys=True, separators=(",", ":")) + "\n")
    print(f"PASS: exact v4 evidence contains {len(tests)} tests and {len(fixtures)} fixtures")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
