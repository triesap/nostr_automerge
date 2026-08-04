#!/usr/bin/env python3
"""Validate the closed stable diagnostic-code registry."""

import json
import pathlib
import re

ROOT = pathlib.Path(__file__).resolve().parents[1]
CATEGORIES = {"raw_event", "nip01", "carrier", "automerge", "control", "graph", "checkpoint", "budget", "cancellation"}
PATTERN = re.compile(r"^[a-z][a-z0-9_]*\.[a-z][a-z0-9_]*$")


def validate(data: object) -> None:
    if not isinstance(data, dict) or set(data) != {"schema", "revision", "codes"}:
        raise ValueError("registry shape")
    if data["schema"] != "nostr_automerge.diagnostic_codes.v1" or data["revision"] != "draft_2026_08":
        raise ValueError("registry identity")
    seen = set()
    covered = set()
    for entry in data["codes"]:
        if set(entry) != {"code", "category", "meaning"} or not PATTERN.fullmatch(entry["code"]):
            raise ValueError("entry shape")
        if entry["category"] not in CATEGORIES or entry["code"] in seen or not entry["meaning"]:
            raise ValueError("entry semantics")
        seen.add(entry["code"])
        covered.add(entry["category"])
    if covered != CATEGORIES:
        raise ValueError("category coverage")


def main() -> None:
    data = json.loads((ROOT / "spec/diagnostic_codes.json").read_text())
    validate(data)
    for mutation in (dict(data, extra=True), dict(data, revision="future"), dict(data, codes=data["codes"] + [data["codes"][0]])):
        try: validate(mutation)
        except ValueError: continue
        raise AssertionError("negative diagnostic fixture accepted")
    print("PASS: stable diagnostic registry")
    print(f"- codes={len(data['codes'])}")
    print(f"- categories={len(CATEGORIES)}")


if __name__ == "__main__":
    main()
