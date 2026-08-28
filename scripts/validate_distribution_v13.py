#!/usr/bin/env python3
"""Validate the closed distribution-v13 authority and staged manifest."""

from __future__ import annotations

import copy
import hashlib
import json
import subprocess
from typing import Any

import generate_distribution_v13 as distribution


ROOT = distribution.ROOT
SCHEMA_KEYS = ("$schema", "$id", "title", "type", "additionalProperties", "required", "properties", "$defs")
GENERATOR_PATH = ROOT / "tools/nostr_automerge_conformance/src/fixture_generation.rs"
GENERATOR_SHA256 = "afe76f150b03852fd3aa5659b59e480c846de96ccfced42cdc6dbd5cf0606b51"
LOCK_PATH = "fixtures/distribution/manifest_v13.lock.json"
LOCK_SHA256 = "c8145bbbd84d5d149b7ae7712f2ee4d16e3c8f5f367348c7119b41ccac40333f"
LOCK_KEYS = (
    "schema", "status", "source_candidate", "manifest_sha256", "scenario_count",
    "file_count", "fixture_ids_sha256", "files_sha256",
    "fixture_rebindings_sha256", "profiles_sha256", "result_identity_sha256",
)
LOCK_EXPECTED = {
    "schema": "nostr_automerge.fixture_distribution_lock.v13.v1",
    "status": "locked",
    "source_candidate": "378f15e7af474e34884b9b25a19960d37b0c02f6",
    "manifest_sha256": "12aa1b1f806ce810463768d566cc63d2ceba6126014d4da9fe5688df518bef3f",
    "scenario_count": 204,
    "file_count": 671,
    "fixture_ids_sha256": "523a1c6203080aefc91107f203bb305e9405a800f6c3182de5d4bd73730bf200",
    "files_sha256": "cbd60e1b5ec560b5f9d7bd3f146d072ca1a2e6c030e1cbf3041ba03503481fe6",
    "fixture_rebindings_sha256": "2f557a3a0567b4453b7329ebc54a96cd8d745dd5f3751cac8081bd0b035733be",
    "profiles_sha256": "84f80b3a819b70ea943c861dd6636d22c8c66d489c68a422cf372b045e727134",
    "result_identity_sha256": "f06fe6c3dbe3674cf2c138a60846262f5a89b7cb8bec4fafd94401d47c73d293",
}


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
    require(properties["files"]["maxItems"] == 671, "schema_files")
    require(properties["authorized_v12_fixture_rebindings"]["maxItems"] == 4, "schema_fixture_rebindings")
    require(set(schema["$defs"]) == {"sha256", "identifiers", "rebinding", "fixture_rebinding", "fixture", "file"}, "schema_defs")


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
    complete = state["current_stage"] == "distribution_complete"
    require(len(manifest["files"]) == 641 + 3 * count + 12 * int(complete), "manifest_file_count")
    require(len(manifest["authorized_v12_fixture_rebindings"]) == 4 * int(complete), "manifest_fixture_rebinding")
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


def canonical(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()


def validate_lock(lock: object, manifest: dict[str, Any]) -> None:
    require(type(lock) is dict and tuple(lock) == LOCK_KEYS, "lock_shape")
    assert isinstance(lock, dict)
    require(lock == LOCK_EXPECTED, "lock_binding")
    projection = {key: lock[key] for key in LOCK_KEYS[:-1]}
    require(hashlib.sha256(canonical(projection)).hexdigest() == lock["result_identity_sha256"], "lock_identity")
    require(hashlib.sha256((ROOT / distribution.OUTPUT_PATH).read_bytes()).hexdigest() == lock["manifest_sha256"], "lock_manifest")
    require(hashlib.sha256(canonical([row["fixture_id"] for row in manifest["fixtures"]])).hexdigest() == lock["fixture_ids_sha256"], "lock_fixture_ids")
    require(hashlib.sha256(canonical(manifest["files"])).hexdigest() == lock["files_sha256"], "lock_files")
    require(hashlib.sha256(canonical(manifest["authorized_v12_fixture_rebindings"])).hexdigest() == lock["fixture_rebindings_sha256"], "lock_rebindings")
    require(hashlib.sha256(canonical(manifest["profiles"])).hexdigest() == lock["profiles_sha256"], "lock_profiles")
    require(hashlib.sha256((ROOT / LOCK_PATH).read_bytes()).hexdigest() == LOCK_SHA256, "lock_hash")
    candidate = subprocess.run(
        ("git", "rev-parse", lock["source_candidate"] + "^{commit}"),
        cwd=ROOT, capture_output=True, check=False, text=True,
    )
    require(candidate.returncode == 0 and candidate.stdout.strip() == lock["source_candidate"], "lock_candidate")


def mutation_self_test(state: dict[str, Any], manifest: dict[str, Any], schema: dict[str, Any], source: bytes, lock: dict[str, Any]) -> int:
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
    for enabled, paths in (
        (True, ()),
        (True, tuple(reversed(distribution.rebinding_paths()))),
        (True, distribution.rebinding_paths() + (distribution.rebinding_paths()[0],)),
        (False, distribution.rebinding_paths()),
    ):
        try:
            distribution.validate_rebinding_inventory(enabled, paths)
        except distribution.DistributionError:
            continue
        raise distribution.DistributionError("mutation:rebinding_inventory")
    try:
        validate_generator_source(source + b"\n")
    except distribution.DistributionError:
        pass
    else:
        raise distribution.DistributionError("mutation:generator")
    lock_mutations = []
    for mutate in (
        lambda value: value.update(extra=False),
        lambda value: value.update(manifest_sha256="0" * 64),
        lambda value: value.update(scenario_count=203),
        lambda value: value.update(files_sha256="0" * 64),
        lambda value: value.update(result_identity_sha256="0" * 64),
    ):
        changed = copy.deepcopy(lock)
        mutate(changed)
        lock_mutations.append(changed)
    for changed in lock_mutations:
        try:
            validate_lock(changed, manifest)
        except distribution.DistributionError:
            continue
        raise distribution.DistributionError("mutation:lock")
    return len(state_mutations) + len(manifest_mutations) + len(schema_mutations) + len(companion_mutations) + len(lock_mutations) + 5


def main() -> int:
    state = distribution.load(distribution.STATE_PATH)
    schema = distribution.load(distribution.SCHEMA_PATH)
    manifest = distribution.load(distribution.OUTPUT_PATH)
    lock = distribution.load(LOCK_PATH)
    validate_schema(schema)
    validate_manifest(manifest, state)
    source = GENERATOR_PATH.read_bytes()
    validate_generator_source(source)
    validate_lock(lock, manifest)
    require((ROOT / distribution.OUTPUT_PATH).read_bytes() == distribution.canonical_bytes(state), "manifest_bytes")
    mutations = mutation_self_test(state, manifest, schema, source, lock)
    print("PASS: distribution-v13 authority")
    print(f"- stage={state['current_stage']}")
    print(f"- fixtures={manifest['fixture_count']}/204")
    print(f"- planned={len(distribution.PLAN)}")
    print(f"- negative_mutations={mutations}")
    print(f"- manifest_sha256={hashlib.sha256((ROOT / distribution.OUTPUT_PATH).read_bytes()).hexdigest()}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
