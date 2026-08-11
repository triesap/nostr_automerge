#!/usr/bin/env python3
"""Write the signed-distribution handoff to ignored operator storage."""

from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "fixtures" / "distribution" / "manifest_v3.json"


def main() -> int:
    manifest_bytes = MANIFEST.read_bytes()
    manifest = json.loads(manifest_bytes)
    output_root = Path(os.environ.get("NOSTR_AUTOMERGE_OUTPUT_ROOT", ".local/evidence"))
    output_root.mkdir(parents=True, exist_ok=True)
    handoff = {
        "distribution_id": manifest["distribution_id"],
        "fixture_count": len(manifest["fixtures"]),
        "manifest_sha256": hashlib.sha256(manifest_bytes).hexdigest(),
        "profiles": {
            profile: len(fixtures) for profile, fixtures in sorted(manifest["profiles"].items())
        },
        "provenance": "operator-local",
        "revision": manifest["protocol_revision"],
        "schema": "nostr_automerge.interop_handoff.v2",
    }
    target = output_root / "interop_handoff_v2.json"
    target.write_text(
        json.dumps(handoff, sort_keys=True, separators=(",", ":")) + "\n",
        encoding="utf-8",
    )
    print("PASS: operator-only signed fixture handoff")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
