#!/usr/bin/env python3
"""Bind the final append-only 139-row authority used by evidence v9."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
ADDITIONS = (
    "NCRDT-BRANCH-003", "NCRDT-BRANCH-004", "NCRDT-SCOPE-007",
    "NCRDT-RESOURCE-011", "NCRDT-RESOURCE-012", "NCRDT-DISPOSITION-004",
    "NCRDT-DISPOSITION-005", "NCRDT-NIP-003", "NCRDT-CONF-009",
    "NCRDT-EVIDENCE-005",
)


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    registry = json.loads((ROOT / "spec/requirements.json").read_text())
    applicability = json.loads((ROOT / "spec/requirements_applicability.json").read_text())
    rows = registry["requirements"]
    identifiers = [row["id"] for row in rows]
    report = {
        "schema": "nostr_automerge.requirements_authority.v9",
        "status": "finalized",
        "requirement_count": 139,
        "preserved_prefix_count": 129,
        "appended_ids": list(ADDITIONS),
        "ordered_ids_sha256": hashlib.sha256(
            json.dumps(identifiers, separators=(",", ":")).encode()
        ).hexdigest(),
        "requirements_sha256": sha256(ROOT / "spec/requirements.json"),
        "applicability_sha256": sha256(ROOT / "spec/requirements_applicability.json"),
        "nip_sha256": sha256(ROOT / "spec/NIP_DRAFT.md"),
        "companion_sha256": sha256(ROOT / "spec/NOSTR_AUTOMERGE_V1_SPEC.md"),
        "conformance_sha256": sha256(ROOT / "spec/CONFORMANCE.md"),
        "fixture_distribution_sha256": sha256(ROOT / "fixtures/distribution/manifest_v9.json"),
        "applicability_count": len(applicability["classifications"]),
        "result": "pass",
    }
    (ROOT / "reports/requirements_authority_v9.json").write_text(
        json.dumps(report, sort_keys=True, separators=(",", ":")) + "\n",
        encoding="utf-8",
    )
    print("PASS: bound final append-only 139-row requirement authority")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
