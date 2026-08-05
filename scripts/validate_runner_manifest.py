#!/usr/bin/env python3
"""Validate the tracked contract for ignored local Act workflows."""

from __future__ import annotations

import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
EXPECTED_JOBS = [
    "remediation", "policy", "standard", "conformance", "coverage", "supply_chain",
    "robustness", "resource", "release_evidence",
]
EXPECTED_TOOLS = {
    "rust": "1.97.1",
    "rust_nightly": "nightly-2026-07-16",
    "act": "0.2.89",
    "cargo_llvm_cov": "0.8.7",
    "cargo_deny": "0.19.8",
    "cargo_fuzz": "0.13.2",
}


def main() -> int:
    """Validate the manifest's closed job and toolchain sets."""

    manifest = json.loads((ROOT / "local_runner_manifest.json").read_text())
    if manifest.get("schema") != "nostr_automerge.local_runner_manifest.v1":
        raise AssertionError("unsupported local runner manifest schema")
    if manifest.get("workflow") != ".act/workflows/local_suite.yml":
        raise AssertionError("unexpected local workflow path")
    if manifest.get("remediation_workflow") != ".act/workflows/remediation.yml":
        raise AssertionError("unexpected remediation workflow path")
    if manifest.get("jobs") != EXPECTED_JOBS:
        raise AssertionError("local runner job set or order differs from policy")
    if manifest.get("toolchain") != EXPECTED_TOOLS:
        raise AssertionError("local runner toolchain differs from policy")
    if manifest.get("entrypoint") != "python3 scripts/local_gate.py":
        raise AssertionError("unexpected local runner entrypoint")
    if manifest.get("output_root") != ".act/output":
        raise AssertionError("unexpected local runner output root")
    print("PASS: Rust local runner manifest")
    print(f"- jobs={len(EXPECTED_JOBS)}")
    print(f"- pinned_tools={len(EXPECTED_TOOLS)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
