#!/usr/bin/env python3
"""Validate ordered follow-up remediation execution authority."""

from __future__ import annotations

import re
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
EXPECTED = [
    (15, 308, 317), (16, 318, 336), (17, 337, 355), (18, 356, 381),
    (19, 382, 398), (20, 399, 409), (21, 410, 429), (22, 430, 443),
    (23, 444, 459), (24, 460, 481), (25, 482, 493), (26, 494, 506),
    (27, 507, 519), (28, 520, 533),
]


def main() -> int:
    ledger = (ROOT / "docs/execution/remediation_v2/ledger.md").read_text()
    rows = [
        (int(number), int(start), int(end))
        for number, start, end in re.findall(
            r"\| (\d+) \| `step_(\d{3})`–`step_(\d{3})` \|", ledger
        )
    ]
    if rows != EXPECTED:
        raise AssertionError("follow-up RCLD ranges are missing or reordered")
    steps = [step for _, start, end in rows for step in range(start, end + 1)]
    if steps != list(range(308, 534)):
        raise AssertionError("follow-up checkpoints are not contiguous and unique")
    current = re.search(r"Current checkpoint: `step_(\d{3})`", ledger)
    if current is None or current.group(1) != "343":
        raise AssertionError("unexpected active follow-up checkpoint")
    active = [line for line in ledger.splitlines() if "| active |" in line]
    if len(active) != 1 or not active[0].startswith("| 17 |"):
        raise AssertionError("exactly RCLD 17 must be active")
    for phrase in ("Only one RCLD", "never", "cannot bypass", "Deviations"):
        if phrase.lower() not in ledger.lower():
            raise AssertionError(f"missing execution policy: {phrase}")
    print("PASS: follow-up ledger covers 226 ordered checkpoints with one active slice")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
