#!/usr/bin/env python3
"""Compare independent canonical profile reports and detect a deliberate mismatch."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def canonical(value: object) -> bytes:
    return (json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n").encode()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("profile", choices=("core", "checkpoint"))
    parser.add_argument("--typescript-root", required=True, type=Path)
    args = parser.parse_args()
    typescript = args.typescript_root.resolve(strict=True)
    rust_report = json.loads((ROOT / f"reports/interop_rust_{args.profile}.canonical.json").read_text())
    typescript_report = json.loads(
        (typescript / f"reports/interop_typescript_{args.profile}.canonical.json").read_text()
    )
    rust_attestation = json.loads(
        (ROOT / f"reports/interop_rust_{args.profile}.attestation.json").read_text()
    )
    typescript_attestation = json.loads(
        (typescript / f"reports/interop_typescript_{args.profile}.attestation.json").read_text()
    )
    rust_bytes = canonical(rust_report)
    typescript_bytes = canonical(typescript_report)
    if rust_bytes != typescript_bytes:
        raise AssertionError(f"{args.profile} canonical report mismatch")
    mismatch = bytearray(typescript_bytes)
    mismatch[-2] ^= 1
    if rust_bytes == bytes(mismatch):
        raise AssertionError("deliberate mismatch was not detected")
    if rust_attestation["fixture_manifest_sha256"] != typescript_attestation["fixture_manifest_sha256"]:
        raise AssertionError("attestations bind different fixture manifests")
    result = {
        "canonical_report_bytes": "identical",
        "deliberate_mismatch": "detected",
        "fixture_count": len(rust_report),
        "fixture_manifest_sha256": rust_attestation["fixture_manifest_sha256"],
        "profile": args.profile,
        "report_sha256": hashlib.sha256(rust_bytes).hexdigest(),
        "rust_commit": rust_attestation["commit"],
        "schema": "nostr_automerge.interop_profile_agreement.v1",
        "status": "pass",
        "typescript_commit": typescript_attestation["commit"],
    }
    (ROOT / f"reports/interop_{args.profile}_agreement.json").write_text(
        json.dumps(result, sort_keys=True, separators=(",", ":")) + "\n",
        encoding="utf-8",
    )
    print(f"PASS: {args.profile} profile canonical report bytes are identical")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
