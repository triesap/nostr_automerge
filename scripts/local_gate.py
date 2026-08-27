#!/usr/bin/env python3
"""Execute one tracked Rust local-runner gate."""

from __future__ import annotations

import hashlib
import json
import os
import resource as resource_usage
import subprocess
import sys
import time
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = Path(os.environ.get("NOSTR_AUTOMERGE_OUTPUT_ROOT", ROOT / ".local/evidence"))


def run(*command: str, capture: bool = False) -> subprocess.CompletedProcess[str]:
    """Run one command from the repository root and fail closed."""

    try:
        return subprocess.run(
            command, cwd=ROOT, check=True, text=True, capture_output=capture
        )
    except subprocess.CalledProcessError as error:
        if capture:
            print(error.stdout or "", end="")
            print(error.stderr or "", end="", file=sys.stderr)
        raise


def policy() -> None:
    run("python3", "scripts/validate_repository_policy.py")
    run("python3", "scripts/validate_runner_manifest.py")


def remediation() -> None:
    run("python3", "scripts/validate_remediation.py")
    run("python3", "scripts/validate_resource_operation_inventory_v10.py")
    run("python3", "scripts/reproduce_resource_followup_v10.py", "--verify-open")
    run("python3", "scripts/reproduce_remediation_v11.py", "--verify-state")


def standard() -> None:
    run("cargo", "fmt", "--all", "--check")
    run("cargo", "check", "--workspace", "--all-targets", "--locked")
    run("cargo", "test", "--workspace", "--all-targets", "--locked")
    run("cargo", "clippy", "--workspace", "--all-targets", "--all-features", "--locked", "--", "-D", "warnings")
    run("cargo", "doc", "--workspace", "--no-deps", "--locked")
    run("cargo", "run", "-p", "nostr_automerge_xtask", "--locked", "--", "validate")


def conformance() -> None:
    command = (
        "cargo", "run", "--quiet", "-p", "nostr_automerge_conformance",
        "--locked", "--", "run_distribution", "fixtures/distribution/manifest_v11.json",
    )
    first = run(*command, capture=True).stdout
    second = run(*command, capture=True).stdout
    if first != second:
        raise AssertionError("Rust corpus output changed between local runs")
    summary = json.loads(first)
    if summary.get("status") != "pass" or summary.get("fixture_count") != 193:
        raise AssertionError("Rust distribution did not pass in both independent processes")
    OUTPUT.mkdir(parents=True, exist_ok=True)
    (OUTPUT / "rust_distribution_v11.json").write_text(first, encoding="utf-8")
    (OUTPUT / "rust_distribution_v11_process_evidence.json").write_text(
        json.dumps(
            {
                "canonical_bytes": "identical",
                "process_count": 2,
                "sha256": hashlib.sha256(first.encode()).hexdigest(),
                "status": "pass",
            },
            sort_keys=True,
            separators=(",", ":"),
        )
        + "\n",
        encoding="utf-8",
    )


def coverage() -> None:
    result = run(
        "cargo", "+nightly-2026-07-16", "llvm-cov", "--branch", "--workspace", "--all-targets",
        "--exclude", "nostr_automerge_xtask", "--locked", "--summary-only",
        capture=True,
    )
    OUTPUT.mkdir(parents=True, exist_ok=True)
    (OUTPUT / "rust_coverage.txt").write_text(result.stdout, encoding="utf-8")
    print(result.stdout, end="")


def supply_chain() -> None:
    run("cargo", "deny", "check")


def robustness() -> None:
    run("cargo", "+nightly-2026-07-16", "fuzz", "build")
    run("cargo", "test", "-p", "nostr_automerge", "--test", "properties", "--locked")


def resource() -> None:
    started = time.monotonic_ns()
    run("python3", "scripts/validate_target_work_accounting_v11.py", "--run-proofs")
    run("python3", "scripts/validate_persistent_ownership_v11.py", "--run-proofs")
    run("python3", "scripts/validate_resource_operation_inventory_v10.py", "--run-proofs")
    run("python3", "scripts/reproduce_resource_followup_v10.py", "--verify-open")
    run("python3", "scripts/reproduce_remediation_v11.py", "--verify-state")
    run("cargo", "bench", "-p", "nostr_automerge", "--bench", "resource_smoke", "--locked")
    run(
        "cargo", "test", "-p", "nostr_automerge", "--lib",
        "scaling", "--locked",
    )
    run(
        "cargo", "test", "-p", "nostr_automerge", "--test", "public_engine_api",
        "every_v3_work_counter_boundary", "--locked",
    )
    elapsed = time.monotonic_ns() - started
    maximum_resident_set_bytes = resource_usage.getrusage(
        resource_usage.RUSAGE_CHILDREN
    ).ru_maxrss
    OUTPUT.mkdir(parents=True, exist_ok=True)
    (OUTPUT / "rust_resource_smoke.json").write_text(
        json.dumps(
            {
                "elapsed_ns": str(elapsed),
                "maximum_resident_set_bytes": maximum_resident_set_bytes,
                "status": "pass",
            },
            separators=(",", ":"),
        ) + "\n",
        encoding="utf-8",
    )


def release_evidence() -> None:
    run("cargo", "package", "-p", "nostr_automerge", "--locked", "--allow-dirty")
    evidence = run("cargo", "run", "-p", "nostr_automerge_xtask", "--locked", "--", "sbom", capture=True).stdout
    OUTPUT.mkdir(parents=True, exist_ok=True)
    (OUTPUT / "nostr_automerge.cdx.json").write_text(evidence, encoding="utf-8")


JOBS = {
    "remediation": remediation,
    "policy": policy,
    "standard": standard,
    "conformance": conformance,
    "coverage": coverage,
    "supply_chain": supply_chain,
    "robustness": robustness,
    "resource": resource,
    "release_evidence": release_evidence,
}


def main() -> int:
    """Dispatch exactly one closed local gate."""

    if len(sys.argv) != 2 or sys.argv[1] not in JOBS:
        print("usage: local_gate.py <job>", file=sys.stderr)
        return 2
    JOBS[sys.argv[1]]()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
