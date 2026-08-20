#!/usr/bin/env python3
"""Validate the v7 companion delta while preserving the external NIP snapshot."""

from __future__ import annotations

import hashlib
import json
import re
import subprocess
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
HEX40 = re.compile(r"^[0-9a-f]{40}$")


def digest(relative: str) -> str:
    return hashlib.sha256((ROOT / relative).read_bytes()).hexdigest()


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
    authority = json.loads(
        (ROOT / "reports/remediation_v7_companion_authority.json").read_text()
    )
    expected = {
        "requirements_sha256": digest("spec/requirements.json"),
        "applicability_sha256": digest("spec/requirements_applicability.json"),
        "fixture_distribution_sha256": digest("fixtures/distribution/manifest_v8.json"),
    }
    if any(authority.get(key) != value for key, value in expected.items()):
        raise AssertionError("stale remediation-v7 authority hash")
    if authority.get("companion", {}).get("sha256") != digest(
        "spec/NOSTR_AUTOMERGE_V1_SPEC.md"
    ):
        raise AssertionError("stale companion authority hash")
    if authority.get("portable_delta", {}).get("sha256") != digest(
        "spec/NIP_V7_PATCH_PROPOSAL.md"
    ):
        raise AssertionError("stale portable delta hash")
    if authority.get("protocol_revision", {}).get("sha256") != digest(
        "spec/protocol_revision.json"
    ):
        raise AssertionError("stale protocol revision hash")
    candidate = authority.get("source_candidate", "")
    if not HEX40.fullmatch(candidate) or subprocess.run(
        ["git", "cat-file", "-e", f"{candidate}^{{commit}}"],
        cwd=ROOT,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    ).returncode:
        raise AssertionError("invalid source candidate")
    if authority.get("publication_authorized") is not False:
        raise AssertionError("authority overclaims publication")
    print("PASS: remediation-v7 companion reconciliation")
    print(f"- synchronized_sections={len(SECTIONS)}")
    print(f"- nip_sha256={nip_hash}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
