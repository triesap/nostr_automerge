#!/usr/bin/env python3
"""Validate the closed distribution-v12 authority and staged manifest."""

from __future__ import annotations

import copy
import hashlib
import json
from typing import Any

import generate_distribution_v12 as distribution


ROOT = distribution.ROOT
SCHEMA_KEYS = (
    "$schema",
    "$id",
    "title",
    "type",
    "additionalProperties",
    "required",
    "properties",
    "$defs",
)


def require(condition: bool, diagnostic: str) -> None:
    if not condition:
        raise distribution.DistributionError(diagnostic)


def load(relative: str) -> dict[str, Any]:
    return distribution.load(relative)


def validate_schema(schema: dict[str, Any]) -> None:
    require(tuple(schema) == SCHEMA_KEYS, "schema_keys")
    require(schema["type"] == "object" and schema["additionalProperties"] is False, "schema_closed")
    required = schema["required"]
    properties = schema["properties"]
    require(type(required) is list and set(required) == set(properties), "schema_required")
    require(properties["target_fixture_count"] == {"const": 198}, "schema_target")
    require(properties["fixture_count"] == {"enum": [193, 196, 197, 198]}, "schema_counts")
    require(properties["preserved_v11_fixture_count"] == {"const": 193}, "schema_preserved")
    require(properties["authorized_v11_source_rebindings"]["minItems"] == 2, "schema_rebindings")
    require(set(schema["$defs"]) == {"sha256", "identifiers", "rebinding", "fixture", "file"}, "schema_defs")


def validate_manifest(manifest: dict[str, Any], state: dict[str, Any]) -> None:
    expected = distribution.expected_manifest(state)
    require(manifest == expected, "manifest_exact")
    count = distribution.STAGE_COUNTS[state["current_stage"]]
    planned = [row[0] for row in distribution.PLAN]
    require(manifest["appended_v12_fixtures"] == planned[:count], "manifest_prefix")
    require(manifest["missing_v12_fixtures"] == planned[count:], "manifest_suffix")
    require(manifest["fixture_count"] == 193 + count, "manifest_count")
    require(manifest["complete"] is (count == 5), "manifest_complete")
    require(len(manifest["files"]) == 623 + 3 * count, "manifest_file_count")
    target_identifiers = sorted(
        [
            *(row["fixture_id"] for row in distribution.base_manifest()["fixtures"]),
            *planned,
        ],
        key=str.encode,
    )
    require(
        len(target_identifiers) == 198 and len(set(target_identifiers)) == 198,
        "target_inventory",
    )
    require(
        manifest["appended_v12_fixtures"] + manifest["missing_v12_fixtures"] == planned,
        "planned_partition",
    )


def mutation_self_test(
    state: dict[str, Any], manifest: dict[str, Any], schema: dict[str, Any]
) -> int:
    state_mutations = []
    for mutate in (
        lambda value: value.update(extra=False),
        lambda value: value["stage_order"].reverse(),
        lambda value: value["planned_v12_fixtures"].reverse(),
        lambda value: value.update(target_fixture_count=197),
        lambda value: value.update(base_manifest_sha256="0" * 64),
        lambda value: value.update(requirements_sha256="0" * 64),
    ):
        candidate = copy.deepcopy(state)
        mutate(candidate)
        state_mutations.append(candidate)
    manifest_mutations = []
    for mutate in (
        lambda value: value.update(extra=False),
        lambda value: value.pop("missing_v12_fixtures"),
        lambda value: value["planned_v12_fixtures"].reverse(),
        lambda value: value.update(base_manifest_sha256="0" * 64),
        lambda value: value["authorized_v11_source_rebindings"].reverse(),
        lambda value: value.update(fixture_count=198),
    ):
        candidate = copy.deepcopy(manifest)
        mutate(candidate)
        manifest_mutations.append(candidate)
    schema_mutations = []
    for mutate in (
        lambda value: value.update(extra=False),
        lambda value: value.update(additionalProperties=True),
        lambda value: value["properties"]["fixture_count"].update(enum=[193, 198]),
        lambda value: value["$defs"].pop("rebinding"),
    ):
        candidate = copy.deepcopy(schema)
        mutate(candidate)
        schema_mutations.append(candidate)
    for candidate in state_mutations:
        try:
            distribution.validate_state(candidate)
        except distribution.DistributionError:
            continue
        raise distribution.DistributionError("mutation:state")
    for candidate in manifest_mutations:
        try:
            validate_manifest(candidate, state)
        except distribution.DistributionError:
            continue
        raise distribution.DistributionError("mutation:manifest")
    for candidate in schema_mutations:
        try:
            validate_schema(candidate)
        except distribution.DistributionError:
            continue
        raise distribution.DistributionError("mutation:schema")
    companion_mutations = (
        (0, ("fixtures/v12/scenarios/resource_followup/unapproved.fixture.json",)),
        (1, ()),
        (1, tuple(reversed(distribution.planned_companion_paths(1)))),
        (
            1,
            tuple(
                path.replace(".expected.json", ".stale.json")
                for path in distribution.planned_companion_paths(1)
            ),
        ),
    )
    for count, paths in companion_mutations:
        try:
            distribution.validate_companion_inventory(count, paths)
        except distribution.DistributionError:
            continue
        raise distribution.DistributionError("mutation:companions")
    return (
        len(state_mutations)
        + len(manifest_mutations)
        + len(schema_mutations)
        + len(companion_mutations)
    )


def main() -> int:
    state = load(distribution.STATE_PATH)
    schema = load(distribution.SCHEMA_PATH)
    manifest = load(distribution.OUTPUT_PATH)
    validate_schema(schema)
    validate_manifest(manifest, state)
    require((ROOT / distribution.OUTPUT_PATH).read_bytes() == distribution.canonical_bytes(state), "manifest_bytes")
    mutations = mutation_self_test(state, manifest, schema)
    print("PASS: distribution-v12 authority")
    print(f"- stage={state['current_stage']}")
    print(f"- fixtures={manifest['fixture_count']}/198")
    print(f"- planned={len(distribution.PLAN)}")
    print(f"- negative_mutations={mutations}")
    print(f"- manifest_sha256={hashlib.sha256((ROOT / distribution.OUTPUT_PATH).read_bytes()).hexdigest()}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
