#!/usr/bin/env python3
"""Validate independent, fail-closed remediation claim levels."""

from __future__ import annotations

import copy
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
ORDER = ["implementation", "signed_conformance", "interop", "release", "publication"]
EXPECTED_CURRENT = {
    "implementation": "incomplete",
    "signed_conformance": "unproven",
    "interop": "unproven",
    "release": "held",
    "publication": "unauthorized",
}


def validate(data: dict) -> None:
    if data.get("schema") != "nostr_automerge.claim_levels.v1":
        raise AssertionError("unexpected claim-level schema")
    levels = data.get("levels", [])
    if [level.get("id") for level in levels] != ORDER:
        raise AssertionError("claim levels are missing or reordered")
    if any(not level.get("prerequisites") for level in levels):
        raise AssertionError("every claim level must have explicit prerequisites")
    current = data.get("current")
    if current != EXPECTED_CURRENT:
        raise AssertionError("current claims must remain fail-closed during remediation")
    publication = levels[-1]
    if publication["prerequisites"] != ["release.ready", "explicit_human_authority"]:
        raise AssertionError("publication must require release readiness and human authority")
    expected_non_implications = {
        f"{left}.{levels[index]['passing_state']}!={right}.{levels[index + 1]['passing_state']}"
        for index, (left, right) in enumerate(zip(ORDER, ORDER[1:]))
    }
    if set(data.get("non_implications", [])) != expected_non_implications:
        raise AssertionError("independent claim boundaries are incomplete")


def expect_rejected(data: dict, reason: str) -> None:
    try:
        validate(data)
    except AssertionError:
        return
    raise AssertionError(f"invalid claim model accepted: {reason}")


def main() -> int:
    data = json.loads((ROOT / "spec/claim_levels.json").read_text())
    validate(data)
    escalated = copy.deepcopy(data)
    escalated["current"]["release"] = "ready"
    expect_rejected(escalated, "release escalated while assurance gates are held")
    published = copy.deepcopy(data)
    published["current"]["publication"] = "authorized"
    expect_rejected(published, "publication escalated without separate authority")
    no_authority = copy.deepcopy(data)
    no_authority["levels"][-1]["prerequisites"].remove("explicit_human_authority")
    expect_rejected(no_authority, "publication authority prerequisite removed")
    implied = copy.deepcopy(data)
    implied["non_implications"].pop()
    expect_rejected(implied, "release incorrectly implies publication")
    print("PASS: five independent remediation claim levels fail closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
