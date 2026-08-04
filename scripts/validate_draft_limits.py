#!/usr/bin/env python3
"""Validate draft limits against the NIP and requirements registry."""

import json
import pathlib

ROOT = pathlib.Path(__file__).resolve().parents[1]
EXPECTED = {
    "manifest_content_bytes": 16384, "control_content_bytes": 32768,
    "control_members": 256, "control_heads": 64, "change_decoded_bytes": 32768,
    "change_operations": 16384, "change_dependencies": 256,
    "checkpoint_raw_bytes": 67108864, "checkpoint_chunks": 4096,
    "checkpoint_chunk_raw_bytes": 32768, "checkpoint_heads": 256,
    "checkpoint_changes": 1000000, "checkpoint_operations": 10000000,
    "checkpoint_dependency_edges": 20000000,
}


def validate(data: object) -> None:
    if not isinstance(data, dict) or set(data) != {"schema", "revision", "status", "limits"}:
        raise ValueError("registry shape")
    if data["schema"] != "nostr_automerge.draft_limits.v1" or data["revision"] != "draft_2026_08" or data["status"] != "normative_for_draft_provisional_for_production":
        raise ValueError("registry identity")
    requirement_ids = {item["id"] for item in json.loads((ROOT / "spec/requirements.json").read_text())["requirements"]}
    names = []
    actual = {}
    for limit in data["limits"]:
        if set(limit) != {"name", "value", "unit", "requirement", "scope"}:
            raise ValueError("limit shape")
        if limit["unit"] not in {"bytes", "items"} or limit["requirement"] not in requirement_ids or not isinstance(limit["value"], int) or limit["value"] <= 0:
            raise ValueError("limit semantics")
        names.append(limit["name"])
        actual[limit["name"]] = limit["value"]
    if len(names) != len(set(names)):
        raise ValueError("limits must be unique")
    if any(actual.get(name) != value for name, value in EXPECTED.items()):
        raise ValueError("NIP limit mismatch")


def main() -> None:
    data = json.loads((ROOT / "spec/draft_limits.json").read_text())
    validate(data)
    changed = json.loads(json.dumps(data)); changed["limits"][0]["value"] = 0
    try: validate(changed)
    except ValueError: pass
    else: raise AssertionError("invalid limit accepted")
    print("PASS: sealed draft limits")
    print(f"- limits={len(data['limits'])}")
    print("- production_status=provisional")


if __name__ == "__main__":
    main()
