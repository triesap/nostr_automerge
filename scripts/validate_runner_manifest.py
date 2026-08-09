#!/usr/bin/env python3
"""Validate the portable gate contract for external operator orchestration."""

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
    "cargo_llvm_cov": "0.8.7",
    "cargo_deny": "0.19.8",
    "cargo_fuzz": "0.13.2",
}


def main() -> int:
    """Validate the manifest's closed job and toolchain sets."""

    manifest = json.loads((ROOT / "local_runner_manifest.json").read_text())
    if manifest.get("schema") != "nostr_automerge.local_runner_manifest.v2":
        raise AssertionError("unsupported local runner manifest schema")
    if manifest.get("orchestration") != "external_operator":
        raise AssertionError("orchestration must remain outside the public repository")
    if manifest.get("jobs") != EXPECTED_JOBS:
        raise AssertionError("local runner job set or order differs from policy")
    if manifest.get("toolchain") != EXPECTED_TOOLS:
        raise AssertionError("local runner toolchain differs from policy")
    if manifest.get("entrypoint") != "python3 scripts/local_gate.py":
        raise AssertionError("unexpected local runner entrypoint")
    if manifest.get("output") != {
        "environment": "NOSTR_AUTOMERGE_OUTPUT_ROOT",
        "standalone_default": ".local/evidence",
    }:
        raise AssertionError("unexpected portable evidence output contract")
    print("PASS: Rust local runner manifest")
    print(f"- jobs={len(EXPECTED_JOBS)}")
    print(f"- pinned_tools={len(EXPECTED_TOOLS)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
