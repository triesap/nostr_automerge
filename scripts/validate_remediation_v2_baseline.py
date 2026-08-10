#!/usr/bin/env python3
"""Validate the follow-up remediation baseline and RCLD continuity."""

from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
GIT_ID = re.compile(r"[0-9a-f]{40}\Z")
SHA256 = re.compile(r"[0-9a-f]{64}\Z")
RANGES = {
    15: (308, 317),
    16: (318, 336),
    17: (337, 355),
    18: (356, 381),
    19: (382, 398),
    20: (399, 409),
    21: (410, 429),
    22: (430, 443),
    23: (444, 459),
    24: (460, 481),
    25: (482, 493),
    26: (494, 506),
    27: (507, 519),
    28: (520, 533),
}


def digest(path: str) -> str:
    return hashlib.sha256((ROOT / path).read_bytes()).hexdigest()


def main() -> int:
    report = json.loads((ROOT / "reports/remediation_v2_baseline.json").read_text())
    if report["schema"] != "nostr_automerge.remediation_v2_baseline.v1":
        raise AssertionError("invalid follow-up baseline schema")
    if report["active_rcld"] != 15 or report["active_step"] != "step_308":
        raise AssertionError("invalid active follow-up checkpoint")
    if report["publication_authorized"] is not False:
        raise AssertionError("baseline cannot authorize publication")
    for section in ("rust", "typescript"):
        head = report[section]["head"]
        if not isinstance(head, str) or GIT_ID.fullmatch(head) is None:
            raise AssertionError(f"invalid {section} head")
    expected_hashes = {
        ("rust", "cargo_lock_sha256"): digest("Cargo.lock"),
        ("authority", "nip_draft_sha256"): digest("spec/NIP_DRAFT.md"),
        ("authority", "companion_spec_sha256"): digest(
            "spec/NOSTR_AUTOMERGE_V1_SPEC.md"
        ),
        ("authority", "requirements_sha256"): digest("spec/requirements.json"),
    }
    for (section, field), expected in expected_hashes.items():
        value = report[section][field]
        if not isinstance(value, str) or SHA256.fullmatch(value) is None or value != expected:
            raise AssertionError(f"stale baseline hash: {section}.{field}")
    previous = 307
    count = 0
    for number, (start, end) in RANGES.items():
        paths = list(
            (ROOT / "docs/execution/rcl").glob(
                f"nostr_automerge_v1_{number}_*_rcld.md"
            )
        )
        if len(paths) != 1 or start != previous + 1:
            raise AssertionError(f"invalid RCLD continuity: {number}")
        text = paths[0].read_text()
        if f"step_{start}" not in text or f"step_{end}" not in text:
            raise AssertionError(f"RCLD range not declared: {number}")
        previous = end
        count += end - start + 1
    if previous != 533 or count != 226:
        raise AssertionError("invalid follow-up step range")
    print("PASS: follow-up baseline and 226-step RCLD sequence are bound")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
