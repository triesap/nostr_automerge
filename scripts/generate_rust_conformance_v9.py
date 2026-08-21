#!/usr/bin/env python3
"""Run signed distribution v9 twice and bind exact Rust canonical bytes."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "fixtures/distribution/manifest_v9.json"
OUTPUT = ROOT / "reports/rust_conformance_v9.json"
COMMAND = (
    "cargo",
    "run",
    "--quiet",
    "-p",
    "nostr_automerge_conformance",
    "--locked",
    "--",
    "run_distribution",
    "fixtures/distribution/manifest_v9.json",
)


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
            "tools",
            "Cargo.toml",
            "Cargo.lock",
            "rust-toolchain.toml",
            "fixtures",
        ),
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()


def run_process() -> tuple[str, dict[str, object]]:
    completed = subprocess.run(
        COMMAND, cwd=ROOT, check=True, capture_output=True, text=True
    )
    value = json.loads(completed.stdout)
    if not isinstance(value, dict):
        raise AssertionError("distribution run did not return an object")
    return completed.stdout, value


def compare(left: dict[str, object], right: dict[str, object]) -> None:
    if left != right:
        raise AssertionError("canonical distribution bytes differ")


def build_report() -> dict[str, object]:
    first_bytes, first = run_process()
    second_bytes, second = run_process()
    if first_bytes != second_bytes:
        raise AssertionError("serialized distribution output changed between processes")
    compare(first, second)
    if (
        first.get("schema") != "nostr_automerge.distribution_run.v1"
        or first.get("status") != "pass"
        or first.get("fixture_count") != 180
        or first.get("delivery_permutations") != 8
        or len(first.get("reports", [])) != 180
    ):
        raise AssertionError("complete signed-v9 run is invalid")

    mismatch = copy.deepcopy(second)
    mismatch["reports"][0]["report_sha256"] = "00" * 32
    try:
        compare(first, mismatch)
    except AssertionError:
        deliberate_mismatch = "rejected"
    else:
        raise AssertionError("deliberate canonical report mismatch was accepted")

    return {
        "schema": "nostr_automerge.rust_conformance.v9",
        "status": "pass",
        "candidate": rust_source_commit(),
        "manifest_sha256": sha256(MANIFEST),
        "cargo_lock_sha256": sha256(ROOT / "Cargo.lock"),
        "rust_toolchain_sha256": sha256(ROOT / "rust-toolchain.toml"),
        "scenario_count": 180,
        "process_count": 2,
        "permutations_per_fixture": 8,
        "canonical_process_bytes": "identical",
        "canonical_output_sha256": first["canonical_output_sha256"],
        "distribution_run_sha256": hashlib.sha256(first_bytes.encode()).hexdigest(),
        "deliberate_mismatch": deliberate_mismatch,
        "commands": [" ".join(COMMAND), "python3 scripts/validate_rust_conformance_v9.py"],
    }


def canonical(value: object) -> str:
    return json.dumps(value, indent=2) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser()
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true")
    mode.add_argument("--check", action="store_true")
    args = parser.parse_args()
    rendered = canonical(build_report())
    if args.check:
        if not OUTPUT.is_file() or OUTPUT.read_text(encoding="utf-8") != rendered:
            raise AssertionError("Rust conformance v9 evidence is stale")
    else:
        OUTPUT.write_text(rendered, encoding="utf-8")
    print("PASS: Rust signed-v9 conformance is repeatable and byte-exact")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
