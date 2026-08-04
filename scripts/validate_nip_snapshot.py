#!/usr/bin/env python3
"""Validate the exact normative NIP snapshot."""

from __future__ import annotations

import hashlib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
NIP_PATH = ROOT / "spec/NIP_DRAFT.md"
CHECKSUM_PATH = ROOT / "spec/NIP_DRAFT.sha256"


def main() -> int:
    """Recompute and validate the committed NIP checksum."""

    fields = CHECKSUM_PATH.read_text(encoding="utf-8").strip().split()
    if len(fields) != 2 or fields[1] != "NIP_DRAFT.md":
        raise AssertionError("NIP_DRAFT.sha256 must contain one canonical entry")

    actual = hashlib.sha256(NIP_PATH.read_bytes()).hexdigest()
    if fields[0] != actual:
        raise AssertionError("normative NIP snapshot checksum mismatch")

    nip = NIP_PATH.read_text(encoding="utf-8")
    for required in (
        "Automerge CRDT Documents over Nostr",
        "nostr-crdt/automerge/actor/v1",
        "Verified-history checkpoints",
        "unsupported_revision",
        "history digest",
    ):
        if required not in nip:
            raise AssertionError(f"normative NIP missing required contract: {required}")

    print("PASS: normative NIP snapshot")
    print(f"- sha256={actual}")
    print("- required_contracts=5")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
