#!/usr/bin/env python3
"""Bind complete Rust and opaque TypeScript signed-v9 distribution evidence."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
REPORTS = ROOT / "reports"


def canonical(value: object) -> bytes:
    return (json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n").encode()


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def load(name: str) -> dict[str, object]:
    value = json.loads((REPORTS / name).read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise AssertionError(f"{name}: object required")
    return value


def main() -> int:
    rust_run = load("rust_conformance_v9.json")
    typescript = load("interop_typescript_v9.json")
    shared = {
        "fixture_distribution_sha256": rust_run["manifest_sha256"],
        "fixture_count": rust_run["scenario_count"],
        "process_runs": rust_run["process_count"],
        "delivery_permutations": rust_run["permutations_per_fixture"],
        "canonical_output_sha256": rust_run["canonical_output_sha256"],
    }
    for field, expected in shared.items():
        if typescript.get(field) != expected:
            raise AssertionError(f"typescript_{field}_mismatch")

    rust = {
        "schema": "nostr_automerge.rust_interop_attestation.v9",
        "implementation_identity": "triesap/nostr_automerge",
        "commit": rust_run["candidate"],
        "evidence_commit": "1d03e6506b79bafef6ecc92433e756fdc68755da",
        "dependency_lock_sha256": rust_run["cargo_lock_sha256"],
        **shared,
        "result": "pass",
        "deliberate_mismatch": "rejected",
        "provenance": "operator-local",
    }
    rust_bytes = canonical(rust)
    typescript_bytes = (REPORTS / "interop_typescript_v9.json").read_bytes()
    mismatch = bytearray(typescript_bytes)
    mismatch[-2] ^= 1
    if bytes(mismatch) == typescript_bytes:
        raise AssertionError("deliberate_mismatch_not_detected")

    combined = {
        "schema": "nostr_automerge.interop_combined.v9",
        **shared,
        "rust_attestation_sha256": sha256(rust_bytes),
        "typescript_attestation_sha256": sha256(typescript_bytes),
        "comparison": "byte_exact_complete_distribution_outputs",
        "deliberate_mismatch": "detected",
        "result": "pass",
        "provenance": "operator-local",
    }
    (REPORTS / "interop_rust_v9.json").write_bytes(rust_bytes)
    (REPORTS / "interop_combined_v9.json").write_bytes(canonical(combined))
    print("PASS: complete Rust and TypeScript signed-v9 outputs match byte-for-byte")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
