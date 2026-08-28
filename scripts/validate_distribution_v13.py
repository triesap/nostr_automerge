#!/usr/bin/env python3
"""Validate the closed distribution-v13 authority and staged manifest."""

from __future__ import annotations

import copy
import hashlib
import json
from typing import Any

import generate_distribution_v13 as distribution


ROOT = distribution.ROOT
SCHEMA_KEYS = ("$schema", "$id", "title", "type", "additionalProperties", "required", "properties", "$defs")
GENERATOR_PATH = ROOT / "tools/nostr_automerge_conformance/src/fixture_generation.rs"
GENERATOR_SHA256 = "ac4b0354326de47c7bc9a40a9ffa57c68a60eacd5bc6c878fe1d99b7bfc77093"


def require(condition: bool, diagnostic: str) -> None:
    if not condition:
        raise distribution.DistributionError(diagnostic)


def validate_schema(schema: object) -> None:
    require(type(schema) is dict and tuple(schema) == SCHEMA_KEYS, "schema_shape")
    assert isinstance(schema, dict)
    require(schema["type"] == "object" and schema["additionalProperties"] is False, "schema_closed")
    require(set(schema["required"]) == set(schema["properties"]), "schema_required")
    properties = schema["properties"]
    require(properties["target_fixture_count"] == {"const": 204}, "schema_target")
    require(properties["fixture_count"] == {"enum": [198, 199, 200, 201, 202, 203, 204]}, "schema_counts")
    require(properties["preserved_v12_fixture_count"] == {"const": 198}, "schema_preserved")
    require(properties["files"]["maxItems"] == 659, "schema_files")
    require(set(schema["$defs"]) == {"sha256", "identifiers", "rebinding", "fixture", "file"}, "schema_defs")


def validate_manifest(manifest: object, state: dict[str, Any]) -> None:
    require(type(manifest) is dict, "manifest_object")
    expected = distribution.expected_manifest(state)
    require(manifest == expected, "manifest_exact")
    _, count = distribution.validate_state(state)
    planned = [row[0] for row in distribution.PLAN]
    require(manifest["appended_v13_fixtures"] == planned[:count], "manifest_prefix")
    require(manifest["missing_v13_fixtures"] == planned[count:], "manifest_suffix")
    require(manifest["fixture_count"] == 198 + count, "manifest_count")
    require(manifest["complete"] is (state["current_stage"] == "distribution_complete"), "manifest_complete")
    require(len(manifest["files"]) == 641 + 3 * count, "manifest_file_count")
    fixture_ids = [row["fixture_id"] for row in manifest["fixtures"]]
    file_paths = [row["path"] for row in manifest["files"]]
    require(fixture_ids == sorted(set(fixture_ids), key=str.encode), "fixture_order")
    require(file_paths == sorted(set(file_paths), key=str.encode), "file_order")
    for path in file_paths:
        parts = path.split("/")
        require(path and not path.startswith("/") and ".." not in parts and "." not in parts, "path_traversal")


def validate_generator_source(source: bytes) -> None:
    require(hashlib.sha256(source).hexdigest() == GENERATOR_SHA256, "generator_hash")
    text = source.decode()
    required = tuple(identifier for identifier, _ in distribution.PLAN)
    require(text.count('"epoch_semantics_v13" => generate_epoch_semantics_v13()') == 1, "generator_route")
    for identifier in required:
        require(text.count(f'let fixture_id = "{identifier}";') == 2, "generator_fixture:" + identifier)
        require(text.count(f"fn {identifier}()") == 1, "generator_test:" + identifier)


def mutation_self_test(state: dict[str, Any], manifest: dict[str, Any], schema: dict[str, Any], source: bytes) -> int:
    state_mutations = []
    for mutate in (
        lambda value: value.update(extra=False),
        lambda value: value["stage_order"].reverse(),
        lambda value: value["planned_v13_fixtures"].reverse(),
        lambda value: value["planned_v13_fixtures"].append(value["planned_v13_fixtures"][0]),
        lambda value: value.update(target_fixture_count=203),
        lambda value: value.update(base_manifest_sha256="0" * 64),
        lambda value: value.update(requirements_sha256="0" * 64),
    ):
        changed = copy.deepcopy(state)
        mutate(changed)
        state_mutations.append(changed)
    manifest_mutations = []
    for mutate in (
        lambda value: value.update(extra=False),
        lambda value: value.pop("missing_v13_fixtures"),
        lambda value: value["planned_v13_fixtures"].reverse(),
        lambda value: value["fixtures"].append(copy.deepcopy(value["fixtures"][0])),
        lambda value: value["files"].append({"path": chr(46) * 2 + "/escape", "sha256": "0" * 64}),
        lambda value: value["authorized_v12_source_rebindings"][0].update(v13_sha256="0" * 64),
        lambda value: value.update(fixture_count=value["fixture_count"] + 1),
        lambda value: value["files"].reverse(),
    ):
        changed = copy.deepcopy(manifest)
        mutate(changed)
        manifest_mutations.append(changed)
    schema_mutations = []
    for mutate in (
        lambda value: value.update(extra=False),
        lambda value: value.update(additionalProperties=True),
        lambda value: value["properties"]["fixture_count"].update(enum=[198, 204]),
        lambda value: value["$defs"].pop("rebinding"),
    ):
        changed = copy.deepcopy(schema)
        mutate(changed)
        schema_mutations.append(changed)
    for changed in state_mutations:
        try:
            distribution.validate_state(changed)
        except distribution.DistributionError:
            continue
        raise distribution.DistributionError("mutation:state")
    for changed in manifest_mutations:
        try:
            validate_manifest(changed, state)
        except distribution.DistributionError:
            continue
        raise distribution.DistributionError("mutation:manifest")
    for changed in schema_mutations:
        try:
            validate_schema(changed)
        except distribution.DistributionError:
            continue
        raise distribution.DistributionError("mutation:schema")
    companion_mutations = (
        (0, ("unapproved.fixture.json",)),
        (1, ()),
        (1, tuple(reversed(distribution.planned_companion_paths(1)))),
        (1, tuple(path.replace(".expected.json", chr(47) + chr(46) * 2 + "/stale.json") for path in distribution.planned_companion_paths(1))),
    )
    for count, paths in companion_mutations:
        try:
            distribution.validate_companion_inventory(count, paths)
        except distribution.DistributionError:
            continue
        raise distribution.DistributionError("mutation:companions")
    try:
        validate_generator_source(source + b"\n")
    except distribution.DistributionError:
        pass
    else:
        raise distribution.DistributionError("mutation:generator")
    return len(state_mutations) + len(manifest_mutations) + len(schema_mutations) + len(companion_mutations) + 1


def main() -> int:
    state = distribution.load(distribution.STATE_PATH)
    schema = distribution.load(distribution.SCHEMA_PATH)
    manifest = distribution.load(distribution.OUTPUT_PATH)
    validate_schema(schema)
    validate_manifest(manifest, state)
    source = GENERATOR_PATH.read_bytes()
    validate_generator_source(source)
    require((ROOT / distribution.OUTPUT_PATH).read_bytes() == distribution.canonical_bytes(state), "manifest_bytes")
    mutations = mutation_self_test(state, manifest, schema, source)
    print("PASS: distribution-v13 authority")
    print(f"- stage={state['current_stage']}")
    print(f"- fixtures={manifest['fixture_count']}/204")
    print(f"- planned={len(distribution.PLAN)}")
    print(f"- negative_mutations={mutations}")
    print(f"- manifest_sha256={hashlib.sha256((ROOT / distribution.OUTPUT_PATH).read_bytes()).hexdigest()}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
