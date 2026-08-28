#!/usr/bin/env python3
"""Validate the closed distribution-v14 budget-rebinding authority."""

from __future__ import annotations

import copy
import json

import generate_distribution_v14 as distribution

ROOT = distribution.ROOT
MANIFEST = ROOT / distribution.OUTPUT_PATH
STATE = ROOT / distribution.STATE_PATH
SCHEMA = ROOT / "tools/validation/distribution_v14.schema.json"
SCHEMA_KEYS = (
    "$schema", "$id", "title", "type", "additionalProperties", "required", "properties"
)


def require(condition: bool, label: str) -> None:
    if not condition:
        raise distribution.DistributionError(label)


def validate_schema(schema: object, manifest: dict[str, object]) -> None:
    require(type(schema) is dict and tuple(schema) == SCHEMA_KEYS, "schema_shape")
    assert isinstance(schema, dict)
    require(schema["type"] == "object" and schema["additionalProperties"] is False, "schema_closed")
    require(schema["required"] == list(manifest), "schema_required")
    require(type(schema["properties"]) is dict and list(schema["properties"]) == list(manifest), "schema_properties")
    require(schema["properties"]["fixture_count"] == {"const": 204}, "schema_count")
    require(schema["properties"]["files"] == {"type": "array", "minItems": 671, "maxItems": 698}, "schema_files")
    require(schema["properties"]["authorized_v13_fixture_rebindings"] == {"type": "array", "maxItems": 9}, "schema_rebindings")


def validate_manifest(manifest: object, state: dict[str, object]) -> None:
    require(type(manifest) is dict, "manifest_object")
    assert isinstance(manifest, dict)
    expected = distribution.expected_manifest(state)
    require(manifest == expected, "manifest_exact")
    complete = distribution.validate_state(state)
    require(manifest["fixture_count"] == 204 and len(manifest["fixtures"]) == 204, "manifest_count")
    require(len(manifest["files"]) == 671 + 27 * int(complete), "manifest_files")
    require(len(manifest["authorized_v13_fixture_rebindings"]) == 9 * int(complete), "manifest_rebindings")
    require(len(manifest["missing_v14_rebindings"]) == 9 * int(not complete), "manifest_missing")
    fixture_ids = [row["fixture_id"] for row in manifest["fixtures"]]
    file_paths = [row["path"] for row in manifest["files"]]
    require(fixture_ids == sorted(set(fixture_ids), key=str.encode), "fixture_order")
    require(file_paths == sorted(set(file_paths), key=str.encode), "file_order")
    base = distribution.historical_base()
    affected = set(manifest["planned_v14_rebindings"])
    base_by_id = {row["fixture_id"]: row for row in base["fixtures"]}
    current_by_id = {row["fixture_id"]: row for row in manifest["fixtures"]}
    require(
        all(current_by_id[identifier] == row for identifier, row in base_by_id.items() if identifier not in affected),
        "unaffected_fixture",
    )
    if not complete:
        require(current_by_id == base_by_id, "staged_fixture_identity")
        actual = () if not distribution.REBINDING_ROOT.exists() else tuple(distribution.REBINDING_ROOT.glob("*.json"))
        require(actual == (), "staged_rebinding_inventory")


def self_test(manifest: dict[str, object], state: dict[str, object], schema: dict[str, object]) -> int:
    cases = []
    for target, mutate in (
        ("manifest", lambda value: value["planned_v14_rebindings"].reverse()),
        ("manifest", lambda value: value["missing_v14_rebindings"].pop()),
        ("manifest", lambda value: value.update(fixture_count=203)),
        ("manifest", lambda value: value["fixtures"].pop()),
        ("manifest", lambda value: value["files"].reverse()),
        ("manifest", lambda value: value.update(extra=False)),
        ("state", lambda value: value["affected_fixture_ids"].reverse()),
        ("state", lambda value: value.update(unaffected_fixture_count=194)),
        ("state", lambda value: value.update(signed_events_preserved=False)),
        ("state", lambda value: value.update(ample_work_reports_preserved=False)),
        ("state", lambda value: value.update(base_manifest_sha256="0" * 64)),
        ("schema", lambda value: value.update(additionalProperties=True)),
        ("schema", lambda value: value["properties"]["files"].update(maxItems=699)),
    ):
        changed_manifest = copy.deepcopy(manifest)
        changed_state = copy.deepcopy(state)
        changed_schema = copy.deepcopy(schema)
        mutate({"manifest": changed_manifest, "state": changed_state, "schema": changed_schema}[target])
        cases.append((changed_manifest, changed_state, changed_schema))
    for index, (changed_manifest, changed_state, changed_schema) in enumerate(cases):
        try:
            validate_manifest(changed_manifest, changed_state)
            validate_schema(changed_schema, changed_manifest)
        except distribution.DistributionError:
            continue
        raise distribution.DistributionError(f"mutation_survived:{index}")
    return len(cases)


def main() -> int:
    state = json.loads(STATE.read_text(encoding="utf-8"))
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    schema = json.loads(SCHEMA.read_text(encoding="utf-8"))
    validate_manifest(manifest, state)
    validate_schema(schema, manifest)
    mutations = self_test(manifest, state, schema)
    print(
        "PASS: distribution-v14 transition "
        f"stage={state['current_stage']} scenarios=204 affected=9 mutations={mutations}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
