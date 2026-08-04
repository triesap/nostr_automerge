#!/usr/bin/env python3
"""Validate required repository-agent policy."""

from __future__ import annotations

from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
AGENTS_PATH = ROOT / "AGENTS.md"
REQUIRED_HEADINGS = (
    "## Source Of Truth",
    "## Work Discipline",
    "## Naming",
    "## Architecture",
    "## Safety And Quality",
    "## Verification",
    "## Completion Report",
)
REQUIRED_PHRASES = (
    "triesap/nostr_automerge",
    "nostr_automerge_conformance",
    "nostr_automerge_xtask",
    "nostr-crdt/automerge/actor/v1",
    "one active checkpoint",
    "Record a deviation",
    "Do not push, publish, release, tag, deploy",
    "Next-step safety",
)


def main() -> int:
    """Validate the policy file and print deterministic results."""

    text = AGENTS_PATH.read_text(encoding="utf-8")
    for heading in REQUIRED_HEADINGS:
        if heading not in text:
            raise AssertionError(f"AGENTS.md missing heading: {heading}")
    for phrase in REQUIRED_PHRASES:
        if phrase not in text:
            raise AssertionError(f"AGENTS.md missing policy: {phrase}")

    print("PASS: repository agent policy")
    print(f"- required_headings={len(REQUIRED_HEADINGS)}")
    print(f"- required_policies={len(REQUIRED_PHRASES)}")
    print("- repository_identity=pass")
    print("- normative_actor_domain=pass")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
