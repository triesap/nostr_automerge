#!/usr/bin/env python3
"""Fail closed on altered historical Rust signed-v9 conformance evidence."""

from __future__ import annotations

import copy
import json
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/rust_conformance_v9.json"
HEX40 = re.compile(r"^[0-9a-f]{40}$")
HEX64 = re.compile(r"^[0-9a-f]{64}$")
FIELDS = (
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
)
HISTORICAL = {
    "schema": "nostr_automerge.rust_conformance.v9",
    "status": "pass",
    "candidate": "99314ccdd03b9112fd70aa475b11fc6762457a09",
    "manifest_sha256": "7b4ab5d2146939d142eb92d43060ef2183c95d1fc574132894b3c01c874c7c56",
    "cargo_lock_sha256": "6d1b886ff74637ba6682d349ab81424b0792f2cbc61cf0f213dfcf16af4f6744",
    "rust_toolchain_sha256": "5d959dfcc98b53886ee772ba216c4f9a1b31f093b46b5b263c0d084af54e821d",
    "scenario_count": 180,
    "process_count": 2,
    "permutations_per_fixture": 8,
    "canonical_process_bytes": "identical",
    "canonical_output_sha256": "e193a7b0db3a43e9d33e612afea05bd447a5e968a45e283d098f45278d6ab6fc",
    "distribution_run_sha256": "17140a4c5cc1653bf7de7f4b5eb6ef8e468c063c6d2dca71bc7d52ddac24e896",
    "deliberate_mismatch": "rejected",
    "commands": [
        "cargo run --quiet -p nostr_automerge_conformance --locked -- run_distribution fixtures/distribution/manifest_v9.json",
        "python3 scripts/validate_rust_conformance_v9.py",
    ],
}


class EvidenceError(ValueError):
    """One Rust conformance evidence invariant failed."""


def validate(value: dict[str, object]) -> None:
    if tuple(value) != FIELDS:
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
    if value != HISTORICAL:
        raise EvidenceError("historical_binding")


def main() -> int:
    value = json.loads(REPORT.read_text(encoding="utf-8"))
    validate(value)
    ancestor = subprocess.run(
        ("git", "merge-base", "--is-ancestor", HISTORICAL["candidate"], "HEAD"),
        cwd=ROOT,
        check=False,
    )
    if ancestor.returncode != 0:
        raise EvidenceError("historical_candidate")
    mutations: list[tuple[str, dict[str, object]]] = []
    for field, replacement in (
        ("candidate", "00" * 20),
        ("manifest_sha256", "00" * 32),
        ("cargo_lock_sha256", "00" * 32),
        ("rust_toolchain_sha256", "00" * 32),
        ("canonical_output_sha256", "00" * 32),
        ("distribution_run_sha256", "00" * 32),
        ("process_count", 1),
        ("deliberate_mismatch", "accepted"),
    ):
        mutation = copy.deepcopy(value)
        mutation[field] = replacement
        mutations.append((field, mutation))
    coordinated = copy.deepcopy(value)
    coordinated["candidate"] = "976d6edb0349ae87d5e477e95ae6f3d7dbd89303"
    coordinated["canonical_output_sha256"] = (
        "84f370b201945c844396406acfb022faa2bdadb32d96206511474a00218770cb"
    )
    coordinated["distribution_run_sha256"] = (
        "74b24f58fe9c20da082dd9ae4c1b344e8468c00a70dbd710adf724ab70ed14c4"
    )
    mutations.append(("coordinated_current_drift", coordinated))
    for field, mutation in mutations:
        try:
            validate(mutation)
        except EvidenceError:
            continue
        raise AssertionError(f"Rust conformance mutation passed: {field}")
    print("PASS: historical Rust conformance v9 evidence is immutable and fail-closed")
    print(f"- negative_mutations={len(mutations)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
