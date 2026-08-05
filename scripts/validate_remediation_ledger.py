#!/usr/bin/env python3
"""Validate ordered remediation execution authority."""

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

def main() -> int:
    ledger = (ROOT / "docs/execution/remediation/nostr_automerge_v1_remediation.md").read_text()
    rcld = (ROOT / "docs/execution/rcl/nostr_automerge_v1_14_engine_remediation_rcld.md").read_text()
    ranges = [(int(a), int(b)) for a, b in re.findall(r"`step_(\d{3})`–`step_(\d{3})`", ledger)]
    expected = [(193, 200), (201, 217), (218, 234), (235, 244), (245, 252), (253, 269), (270, 287), (288, 307)]
    if ranges != expected:
        raise AssertionError("remediation phase ranges are missing or reordered")
    flattened = [number for start, end in ranges for number in range(start, end + 1)]
    if flattened != list(range(193, 308)):
        raise AssertionError("remediation checkpoints are not complete and unique")
    current = re.search(r"Current checkpoint: `step_(\d{3})`", ledger)
    if current is None or f"Current checkpoint: step_{current.group(1)}" not in rcld:
        raise AssertionError("ledger and governing RCLD current checkpoint disagree")
    for phrase in ("Only one checkpoint is active", "cannot bypass", "deviation"):
        if phrase.lower() not in ledger.lower():
            raise AssertionError(f"missing ledger policy: {phrase}")
    print("PASS: remediation ledger covers 115 ordered checkpoints")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
