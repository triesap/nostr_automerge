#!/usr/bin/env python3
"""Validate exact authority and append-only preservation for evidence v9."""

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
    prior = json.loads((ROOT / "reports/requirements_coverage_v8.json").read_text())
    report = json.loads((ROOT / "reports/requirements_authority_v9.json").read_text())
    rows = registry.get("requirements")
    if registry.get("schema") != "nostr_automerge.requirements.v6" or not isinstance(rows, list) or len(rows) != 139:
        raise AssertionError("registry_shape")
    prior_rows = prior.get("rows")
    if not isinstance(prior_rows, list) or len(prior_rows) != 129:
        raise AssertionError("prior_evidence_shape")
    reconciled_authority = {"NCRDT-NIP-001", "NCRDT-NIP-002"}
    for current, old in zip(rows[:129], prior_rows, strict=True):
        authority = old.get("authority", {})
        if current.get("id") != old.get("id"):
            raise AssertionError(f"altered_prefix_order:{current.get('id')}")
        if current.get("id") not in reconciled_authority and (
            current.get("source") != authority.get("source")
            or current.get("section") != authority.get("section")
            or hashlib.sha256(str(current.get("text", "")).encode()).hexdigest() != authority.get("text_sha256")
        ):
            raise AssertionError(f"altered_prefix:{current.get('id')}")
    identifiers = [row.get("id") for row in rows]
    if tuple(identifiers[-10:]) != ADDITIONS or len(set(identifiers)) != 139:
        raise AssertionError("append_order")
    classifications = applicability.get("classifications")
    if not isinstance(classifications, dict) or list(classifications) != identifiers:
        raise AssertionError("applicability_order")
    expected = {
        "schema": "nostr_automerge.requirements_authority.v9",
        "status": "finalized",
        "requirement_count": 139,
        "preserved_prefix_count": 129,
        "appended_ids": list(ADDITIONS),
        "ordered_ids_sha256": hashlib.sha256(json.dumps(identifiers, separators=(",", ":")).encode()).hexdigest(),
        "requirements_sha256": sha256(ROOT / "spec/requirements.json"),
        "applicability_sha256": sha256(ROOT / "spec/requirements_applicability.json"),
        "nip_sha256": sha256(ROOT / "spec/NIP_DRAFT.md"),
        "companion_sha256": sha256(ROOT / "spec/NOSTR_AUTOMERGE_V1_SPEC.md"),
        "conformance_sha256": sha256(ROOT / "spec/CONFORMANCE.md"),
        "fixture_distribution_sha256": sha256(ROOT / "fixtures/distribution/manifest_v9.json"),
        "applicability_count": 139,
        "result": "pass",
    }
    if report != expected:
        raise AssertionError("authority_binding")
    print("PASS: 129-row order is preserved, NIP anchors reconciled, and 10 rows appended")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
