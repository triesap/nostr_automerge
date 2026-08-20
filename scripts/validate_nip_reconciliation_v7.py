#!/usr/bin/env python3
"""Validate the v7 companion delta while preserving the external NIP snapshot."""

from __future__ import annotations

import hashlib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
NIP_SHA256 = "67019c8ea680714052c65226f620a8e1a60b9b10a8f158603063a835a7bbc7a3"
SECTIONS = (
    "Branch-local control evaluation",
    "Coordinate-qualified dependent indexes",
    "Deterministic parent-state propagation",
    "Explicit finalization settlement",
    "Signed conformance v8",
)


def main() -> int:
    companion = (ROOT / "spec/NOSTR_AUTOMERGE_V1_SPEC.md").read_text()
    proposal = (ROOT / "spec/NIP_V7_PATCH_PROPOSAL.md").read_text()
    missing = [section for section in SECTIONS if section not in companion or section not in proposal]
    if missing:
        raise AssertionError(f"companion/proposal section mismatch: {missing}")
    nip_hash = hashlib.sha256((ROOT / "spec/NIP_DRAFT.md").read_bytes()).hexdigest()
    if nip_hash != NIP_SHA256:
        raise AssertionError("read-only NIP snapshot changed")
    required_boundaries = (
        "not submitted",
        "grants no submission",
        "does not modify",
        "explicit external hold",
    )
    if any(boundary not in proposal for boundary in required_boundaries):
        raise AssertionError("portable delta overclaims external authority")
    if "exactly\n171 scenarios" not in companion or "171-scenario" not in proposal:
        raise AssertionError("signed-v8 scenario count is not exact")
    print("PASS: remediation-v7 companion reconciliation")
    print(f"- synchronized_sections={len(SECTIONS)}")
    print(f"- nip_sha256={nip_hash}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
