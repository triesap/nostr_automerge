#!/usr/bin/env python3
"""Publish the combined report from successful operator-supplied evidence."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--typescript-root", required=True, type=Path)
    parser.add_argument("--interop-evidence", required=True, type=Path)
    args = parser.parse_args()
    typescript = args.typescript_root.resolve(strict=True)
    interop_evidence = args.interop_evidence.resolve(strict=True)
    act_evidence = json.loads(interop_evidence.read_text())
    if act_evidence["status"] != "local_differential_pass" or act_evidence["mismatches"]:
        raise AssertionError("local interop evidence did not pass")
    rust_core = json.loads((ROOT / "reports/interop_rust_core.attestation.json").read_text())
    typescript_core = json.loads(
        (typescript / "reports/interop_typescript_core.attestation.json").read_text()
    )
    report = {
        **act_evidence,
        "dependency_locks": {
            "rust": rust_core["dependency_lock_sha256"],
            "typescript": typescript_core["dependency_lock_sha256"],
        },
        "non_claims": [
            "no hosted workflow was used or added",
            "no package was published and no release was authorized",
            "checkpoint evidence never authorizes or redefines document history",
        ],
        "profile_report_digests": {
            profile: json.loads(
                (ROOT / f"reports/interop_{profile}_agreement.json").read_text()
            )["report_sha256"]
            for profile in ("core", "checkpoint")
        },
        "schema": "nostr_automerge.local_interop.v2",
    }
    (ROOT / "reports/interop_combined.json").write_text(
        json.dumps(report, sort_keys=True, separators=(",", ":")) + "\n",
        encoding="utf-8",
    )
    print("PASS: combined local interoperability report")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
