#!/usr/bin/env python3
"""Validate required prior-art and rejected-alternative coverage."""

from __future__ import annotations

import re
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
REQUIRED_IDENTIFIERS = (
    "NIP-78", "#667", "#2192", "#1630", "#2123", "#929", "#400",
    "#569", "#419", "#1015", "#1670", "#2147",
)
REJECTED_HEADINGS = (
    "Generic CRDT envelope",
    "Relay-defined sequence or conflict winner",
    "Timestamp or last-writer-wins authorization",
    "Shared online signer",
    "Second logical clock",
    "Controller-endorsed missing-history recovery",
    "Normative Automerge save-byte digest",
    "Incremental evaluator as initial oracle",
)


def main() -> int:
    """Validate research identifiers, links, and rejected designs."""

    prior = (ROOT / "docs/research/prior_art.md").read_text(encoding="utf-8")
    rejected = (ROOT / "docs/research/rejected_alternatives.md").read_text(
        encoding="utf-8"
    )
    for identifier in REQUIRED_IDENTIFIERS:
        if identifier not in prior:
            raise AssertionError(f"prior art missing identifier: {identifier}")
    links = re.findall(r"\(https://github\.com/nostr-protocol/nips/[^)]+\)", prior)
    if len(links) != 12:
        raise AssertionError(f"expected 12 primary source links, found {len(links)}")
    for heading in REJECTED_HEADINGS:
        if f"## {heading}" not in rejected:
            raise AssertionError(f"rejected alternatives missing: {heading}")

    print("PASS: prior art and rejected alternatives")
    print(f"- identifiers={len(REQUIRED_IDENTIFIERS)}")
    print(f"- primary_links={len(links)}")
    print(f"- rejected_categories={len(REJECTED_HEADINGS)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
