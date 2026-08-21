#!/usr/bin/env python3
"""Fail closed on stale or incomplete Rust signed-v9 conformance evidence."""

from __future__ import annotations

import copy
import hashlib
import json
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/rust_conformance_v9.json"
HEX40 = re.compile(r"^[0-9a-f]{40}$")
HEX64 = re.compile(r"^[0-9a-f]{64}$")
FIELDS = {
    "schema",
    "status",
    "candidate",
    "manifest_sha256",
    "cargo_lock_sha256",
    "rust_toolchain_sha256",
    "scenario_count",
    "process_count",
    "permutations_per_fixture",
    "canonical_process_bytes",
    "canonical_output_sha256",
    "distribution_run_sha256",
    "deliberate_mismatch",
    "commands",
}


class EvidenceError(ValueError):
    """One Rust conformance evidence invariant failed."""


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def rust_source_commit() -> str:
    return subprocess.run(
        (
            "git",
            "log",
            "-1",
            "--format=%H",
            "--",
            "crates",
            "tools/nostr_automerge_conformance",
            "Cargo.toml",
            "Cargo.lock",
            "rust-toolchain.toml",
            "fixtures/distribution/manifest_v9.json",
            "fixtures/v1_draft",
        ),
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()


def expected_distribution_hashes() -> tuple[str, str]:
    manifest = json.loads((ROOT / "fixtures/distribution/manifest_v9.json").read_text())
    aggregate = hashlib.sha256()
    reports = []
    for fixture in sorted(manifest["fixtures"], key=lambda item: item["fixture_id"].encode()):
        fixture_id = fixture["fixture_id"].encode()
        report = (ROOT / fixture["expected_path"]).read_bytes()
        aggregate.update(len(fixture_id).to_bytes(8, "big"))
        aggregate.update(fixture_id)
        aggregate.update(len(report).to_bytes(8, "big"))
        aggregate.update(report)
        reports.append(
            {
                "fixture_id": fixture["fixture_id"],
                "report_sha256": hashlib.sha256(report).hexdigest(),
            }
        )
    canonical_output = aggregate.hexdigest()
    distribution = {
        "canonical_output_sha256": canonical_output,
        "delivery_permutations": 8,
        "fixture_count": len(reports),
        "reports": reports,
        "schema": "nostr_automerge.distribution_run.v1",
        "status": "pass",
    }
    serialized = (json.dumps(distribution, separators=(",", ":")) + "\n").encode()
    return canonical_output, hashlib.sha256(serialized).hexdigest()


def validate(value: dict[str, object], current: bool) -> None:
    if set(value) != FIELDS:
        raise EvidenceError("fields")
    if value["schema"] != "nostr_automerge.rust_conformance.v9" or value["status"] != "pass":
        raise EvidenceError("identity")
    if not HEX40.fullmatch(str(value["candidate"])):
        raise EvidenceError("candidate_shape")
    for field in (
        "manifest_sha256",
        "cargo_lock_sha256",
        "rust_toolchain_sha256",
        "canonical_output_sha256",
        "distribution_run_sha256",
    ):
        if not HEX64.fullmatch(str(value[field])):
            raise EvidenceError(field)
    if (
        value["scenario_count"] != 180
        or value["process_count"] != 2
        or value["permutations_per_fixture"] != 8
        or value["canonical_process_bytes"] != "identical"
        or value["deliberate_mismatch"] != "rejected"
    ):
        raise EvidenceError("coverage")
    commands = value["commands"]
    if (
        not isinstance(commands, list)
        or len(commands) != 2
        or "run_distribution fixtures/distribution/manifest_v9.json" not in commands[0]
        or commands[1] != "python3 scripts/validate_rust_conformance_v9.py"
    ):
        raise EvidenceError("commands")
    if current:
        canonical_output, distribution_run = expected_distribution_hashes()
        if (
            value["candidate"] != rust_source_commit()
            or value["manifest_sha256"] != sha256(ROOT / "fixtures/distribution/manifest_v9.json")
            or value["cargo_lock_sha256"] != sha256(ROOT / "Cargo.lock")
            or value["rust_toolchain_sha256"] != sha256(ROOT / "rust-toolchain.toml")
            or value["canonical_output_sha256"] != canonical_output
            or value["distribution_run_sha256"] != distribution_run
        ):
            raise EvidenceError("stale_binding")


def main() -> int:
    value = json.loads(REPORT.read_text(encoding="utf-8"))
    validate(value, True)
    for field, replacement in (
        ("candidate", "00" * 20),
        ("canonical_output_sha256", "00" * 32),
        ("process_count", 1),
        ("deliberate_mismatch", "accepted"),
    ):
        mutation = copy.deepcopy(value)
        mutation[field] = replacement
        try:
            validate(mutation, True)
        except EvidenceError:
            continue
        raise AssertionError(f"Rust conformance mutation passed: {field}")
    print("PASS: Rust conformance v9 evidence is current and fail-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
