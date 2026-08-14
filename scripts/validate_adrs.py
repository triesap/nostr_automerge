#!/usr/bin/env python3
"""Validate architecture decision numbering, status, and index links."""

from __future__ import annotations

import re
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
ADR_ROOT = ROOT / "docs/adr"


def main() -> int:
    """Validate the complete approved ADR set."""

    index = (ADR_ROOT / "README.md").read_text(encoding="utf-8")
    paths = sorted(ADR_ROOT.glob("adr_[0-9][0-9][0-9][0-9]_*.md"))
    if len(paths) != 58:
        raise AssertionError(f"expected 58 ADRs, found {len(paths)}")

    for number, path in enumerate(paths, 1):
        expected_prefix = f"adr_{number:04d}_"
        if not path.name.startswith(expected_prefix):
            raise AssertionError(f"ADR numbering gap at {path.name}")
        text = path.read_text(encoding="utf-8")
        if re.search(rf"^# ADR {number:04d}:", text, re.MULTILINE) is None:
            raise AssertionError(f"ADR title mismatch: {path.name}")
        if f"[{number:04d}]({path.name}) | Approved |" not in index:
            raise AssertionError(f"ADR missing approved index entry: {path.name}")

    requirement_ids = set(re.findall(r"`(NCRDT-[A-Z0-9-]+)`", index))
    registry = (ROOT / "spec/requirements.json").read_text(encoding="utf-8")
    for identifier in requirement_ids:
        if f'"id": "{identifier}"' not in registry:
            raise AssertionError(f"ADR index references unknown requirement: {identifier}")

    print("PASS: architecture decision records")
    print(f"- decisions={len(paths)}")
    print("- statuses=approved")
    print(f"- mapped_requirements={len(requirement_ids)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
