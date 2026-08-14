#!/usr/bin/env python3
"""Validate companion/portable-delta parity while preserving the NIP snapshot."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
NIP_SHA256 = "67019c8ea680714052c65226f620a8e1a60b9b10a8f158603063a835a7bbc7a3"
SECTIONS = (
    "Causal operation counters",
    "Coordinate-scoped evaluation",
    "Semantic ChangeHash claims",
    "Dependent change authorization",
    "Final claim precedence",
    "Complete dependency knowledge",
    "Control parent and frontier references",
    "Descriptor and chunk references",
    "Manifest attribution and replacement",
    "Dynamic event dispositions",
    "Resource completion and finalization",
    "Mandatory signed conformance",
    "Checkpoint trust boundary",
)


def main() -> int:
    companion = (ROOT / "spec/NOSTR_AUTOMERGE_V1_SPEC.md").read_text()
    proposal = (ROOT / "spec/NIP_V6_PATCH_PROPOSAL.md").read_text()
    missing = [section for section in SECTIONS if section not in companion or section not in proposal]
    if missing:
        raise AssertionError(f"companion/proposal section mismatch: {missing}")
    nip_hash = hashlib.sha256((ROOT / "spec/NIP_DRAFT.md").read_bytes()).hexdigest()
    if nip_hash != NIP_SHA256:
        raise AssertionError("read-only NIP snapshot changed")
    if "not submitted" not in proposal or "grants no submission" not in proposal:
        raise AssertionError("portable delta overclaims external authority")
    authority = json.loads((ROOT / "reports/remediation_v6_companion_authority.json").read_text())
    expected = {
        "requirements_sha256": hashlib.sha256((ROOT / "spec/requirements.json").read_bytes()).hexdigest(),
        "applicability_sha256": hashlib.sha256((ROOT / "spec/requirements_applicability.json").read_bytes()).hexdigest(),
    }
    if any(authority.get(key) != value for key, value in expected.items()):
        raise AssertionError("stale companion authority hash")
    if authority["companion"]["sha256"] != hashlib.sha256(companion.encode()).hexdigest():
        raise AssertionError("stale companion hash")
    if authority["portable_delta"]["sha256"] != hashlib.sha256(proposal.encode()).hexdigest():
        raise AssertionError("stale portable delta hash")
    print("PASS: remediation-v6 companion reconciliation")
    print(f"- synchronized_sections={len(SECTIONS)}")
    print(f"- nip_sha256={nip_hash}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
