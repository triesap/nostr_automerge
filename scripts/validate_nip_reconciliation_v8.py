#!/usr/bin/env python3
"""Validate the reconciled local NIP, exact authority anchors, and holds."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
NIP_SHA256 = "0dfa683aa0f4a1c7d3df010ec95901bf4ba4094ed3adaacc26e85d95aaa4ded1"
COMPANION_SHA256 = "58177c31eb06086d76297bbb0fc15343a8e34c15499d6e03636c63df7604bb10"
PRESERVED_FILES = {
    "spec/protocol_revision.json": "165902e6be66f3528ff3d4745544eeea6468fd01811cc7aab047c54ab2ab5aa1",
    "fixtures/schema/report.schema.json": "75b7f8f1c089ed39d94207dc91a1dca021bb54668df155aece5ffcc42eace378",
    "crates/nostr_automerge/src/profile/kinds.rs": "97189546ae9ca3c7f16c2411504c83064ee75df277fdfe7453bbefecf3163033",
    "crates/nostr_automerge/src/wire/tags.rs": "c18a50132583899c37a21a5bfd4a31006a9eae0050b00ab9123d80b3fb9f9e2e",
    "spec/WIRE_FORMAT.md": "42d1a51abf488532d28910c8e5bae951bdef289415a9e4926f6e6eeb75048e61",
}
NIP_ANCHORS = {
    "NCRDT-NIP-001": "Conformance",
    "NCRDT-NIP-002": "Conformance",
    "NCRDT-BRANCH-003": "Branch-local change outcomes",
    "NCRDT-BRANCH-004": "Branch-local change outcomes",
    "NCRDT-SCOPE-007": "Coordinate scope and deterministic interruption",
    "NCRDT-RESOURCE-011": "Coordinate scope and deterministic interruption",
    "NCRDT-RESOURCE-012": "Coordinate scope and deterministic interruption",
    "NCRDT-DISPOSITION-004": "Semantic ChangeHash and carrier outcomes",
    "NCRDT-DISPOSITION-005": "Semantic ChangeHash and carrier outcomes",
    "NCRDT-NIP-003": "Conformance",
}


def digest(relative: str) -> str:
    return hashlib.sha256((ROOT / relative).read_bytes()).hexdigest()


def main() -> int:
    nip = (ROOT / "spec/NIP_DRAFT.md").read_text(encoding="utf-8")
    companion = (ROOT / "spec/NOSTR_AUTOMERGE_V1_SPEC.md").read_text(encoding="utf-8")
    if digest("spec/NIP_DRAFT.md") != NIP_SHA256:
        raise AssertionError("reconciled local NIP identity changed")
    if digest("spec/NOSTR_AUTOMERGE_V1_SPEC.md") != COMPANION_SHA256:
        raise AssertionError("reconciled companion identity changed")

    required_nip = (
        "NIP-XX",
        "`draft` `optional`",
        "not been submitted",
        "## Branch-local change outcomes",
        "### Semantic ChangeHash and carrier outcomes",
        "### Coordinate scope and deterministic interruption",
        "exactly 180 scenarios",
        "all eight declared delivery permutations",
        "does not imply submission",
    )
    for clause in required_nip:
        if clause not in nip:
            raise AssertionError(f"local NIP clause missing: {clause}")
    for kind in ("1624", "1625", "1626", "1627", "31624"):
        if kind not in nip:
            raise AssertionError(f"provisional kind missing: {kind}")
    for clause in (
        "Remediation v8 reconciled authority",
        "Carrier and semantic report layers",
        "Pass-level interrupted settlement",
        "Signed conformance v9",
        "does not authorize NIP submission",
    ):
        if clause not in companion:
            raise AssertionError(f"companion clause missing: {clause}")

    registry = json.loads((ROOT / "spec/requirements.json").read_text(encoding="utf-8"))
    rows = {row["id"]: row for row in registry["requirements"]}
    applicability = json.loads(
        (ROOT / "spec/requirements_applicability.json").read_text(encoding="utf-8")
    )["classifications"]
    for identifier, section in NIP_ANCHORS.items():
        row = rows[identifier]
        if row["source"] != "spec/NIP_DRAFT.md" or row["section"] != section:
            raise AssertionError(f"stale local NIP authority anchor: {identifier}")
        if applicability[identifier] == "explicitly-deferred":
            raise AssertionError(f"reconciled authority remains deferred: {identifier}")

    for relative, expected in PRESERVED_FILES.items():
        if digest(relative) != expected:
            raise AssertionError(f"reconciliation changed preserved wire authority: {relative}")
    revision = json.loads((ROOT / "spec/protocol_revision.json").read_text())
    if revision.get("sealed") is not True or revision.get("revision") != "draft_2026_08":
        raise AssertionError("sealed protocol revision changed")

    adaptation = json.loads((ROOT / "docs/import_adaptation.json").read_text())
    targets = {row["path"]: row for row in adaptation["imported_files"]}
    if targets["spec/NIP_DRAFT.md"]["target_sha256"] != NIP_SHA256:
        raise AssertionError("local NIP adaptation hash is stale")
    if targets["spec/NOSTR_AUTOMERGE_V1_SPEC.md"]["target_sha256"] != COMPANION_SHA256:
        raise AssertionError("companion adaptation hash is stale")

    print("PASS: remediation-v8 local NIP reconciliation")
    print(f"- nip_sha256={NIP_SHA256}")
    print(f"- companion_sha256={COMPANION_SHA256}")
    print(f"- exact_nip_anchors={len(NIP_ANCHORS)}")
    print("- wire_and_publication_boundaries=preserved")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
