#!/usr/bin/env python3
"""Execute one tracked Rust local-runner gate."""

from __future__ import annotations

import json
import subprocess
import sys
import time
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / ".act/output"


def run(*command: str, capture: bool = False) -> subprocess.CompletedProcess[str]:
    """Run one command from the repository root and fail closed."""

    return subprocess.run(
        command, cwd=ROOT, check=True, text=True, capture_output=capture
    )


def policy() -> None:
    run("python3", "scripts/validate_repository_policy.py")
    run("python3", "scripts/validate_runner_manifest.py")


def remediation() -> None:
    run("python3", "scripts/validate_remediation.py")


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
        "--locked", "--", "run_corpus", "fixtures",
    )
    first = run(*command, capture=True).stdout
    second = run(*command, capture=True).stdout
    if first != second:
        raise AssertionError("Rust corpus output changed between local runs")
    OUTPUT.mkdir(parents=True, exist_ok=True)
    (OUTPUT / "rust_corpus.json").write_text(first, encoding="utf-8")


def coverage() -> None:
    run("cargo", "llvm-cov", "--workspace", "--all-targets", "--locked", "--summary-only")


def supply_chain() -> None:
    run("cargo", "deny", "check")


def robustness() -> None:
    run("cargo", "+nightly-2026-07-16", "fuzz", "build")
    run("cargo", "test", "-p", "nostr_automerge", "--test", "properties", "--locked")


def resource() -> None:
    started = time.monotonic_ns()
    run("cargo", "bench", "-p", "nostr_automerge", "--bench", "resource_smoke", "--locked")
    elapsed = time.monotonic_ns() - started
    OUTPUT.mkdir(parents=True, exist_ok=True)
    (OUTPUT / "rust_resource_smoke.json").write_text(
        json.dumps({"elapsed_ns": str(elapsed), "status": "pass"}, separators=(",", ":")) + "\n",
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
