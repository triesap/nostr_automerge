#!/usr/bin/env python3
"""Validate the closed distribution-v14 budget-rebinding authority."""

from __future__ import annotations

import copy
import hashlib
import json
import subprocess

import generate_distribution_v14 as distribution

ROOT = distribution.ROOT
MANIFEST = ROOT / distribution.OUTPUT_PATH
STATE = ROOT / distribution.STATE_PATH
SCHEMA = ROOT / "tools/validation/distribution_v14.schema.json"
LOCK = ROOT / "fixtures/distribution/manifest_v14.lock.json"
LOCK_SCHEMA = ROOT / "tools/validation/distribution_v14_lock.schema.json"
LOCK_SHA256 = "0fc414a0e49b4e87bb0cf1f21bea3cf0cd70af904720b93a95fae00f079e7304"
SCHEMA_KEYS = (
    "$schema", "$id", "title", "type", "additionalProperties", "required", "properties"
)
LOCK_KEYS = (
    "schema", "status", "source_candidate", "manifest_sha256", "scenario_count",
    "file_count", "fixture_ids_sha256", "files_sha256",
    "fixture_rebindings_sha256", "profiles_sha256", "result_identity_sha256",
)
LOCK_EXPECTED = {
    "schema": "nostr_automerge.fixture_distribution_lock.v14.v1",
    "status": "locked",
    "source_candidate": "54537099a48f79150e46a7d6ebbdab55044a4e42",
    "manifest_sha256": "c76cd24bc91308b0e615bd837d69b72fe145b7713a544fb325f7f054275c485d",
    "scenario_count": 204,
    "file_count": 698,
    "fixture_ids_sha256": "523a1c6203080aefc91107f203bb305e9405a800f6c3182de5d4bd73730bf200",
    "files_sha256": "0955046f5394da48990b3b35b833d65f0c764bb849bc3b5f9d097f5b9b8d6148",
    "fixture_rebindings_sha256": "1d5fda4c3bdde8c8fca688e4f5bc03995bfa017364606325057077c5d4dbfe60",
    "profiles_sha256": "84f80b3a819b70ea943c861dd6636d22c8c66d489c68a422cf372b045e727134",
    "result_identity_sha256": "2e26916e0dfd9c315b4e07d5c15b69181c5ec1f059053c74a55cbe079f92a8b0",
}


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
    else:
        expected_inventory = tuple(
            distribution.REBINDING_ROOT / f"{identifier}.{suffix}.json"
            for identifier, _, _ in distribution.AFFECTED
            for suffix in ("expected", "fixture", "input")
        )
        actual_inventory = tuple(sorted(distribution.REBINDING_ROOT.glob("*.json"), key=lambda path: path.name.encode()))
        require(actual_inventory == tuple(sorted(expected_inventory, key=lambda path: path.name.encode())), "complete_rebinding_inventory")


def canonical(value: object) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()


def validate_lock_schema(schema: object) -> None:
    require(type(schema) is dict and tuple(schema) == SCHEMA_KEYS, "lock_schema_shape")
    assert isinstance(schema, dict)
    require(schema["type"] == "object" and schema["additionalProperties"] is False, "lock_schema_closed")
    require(schema["required"] == list(LOCK_KEYS), "lock_schema_required")
    require(tuple(schema["properties"]) == LOCK_KEYS, "lock_schema_properties")
    require(schema["properties"]["scenario_count"] == {"const": 204}, "lock_schema_count")
    require(schema["properties"]["file_count"] == {"const": 698}, "lock_schema_files")


def validate_lock(lock: object, manifest: dict[str, object]) -> None:
    require(type(lock) is dict and tuple(lock) == LOCK_KEYS, "lock_shape")
    assert isinstance(lock, dict)
    require(lock == LOCK_EXPECTED, "lock_binding")
    projection = {key: lock[key] for key in LOCK_KEYS[:-1]}
    require(hashlib.sha256(canonical(projection)).hexdigest() == lock["result_identity_sha256"], "lock_identity")
    require(hashlib.sha256(MANIFEST.read_bytes()).hexdigest() == lock["manifest_sha256"], "lock_manifest")
    require(hashlib.sha256(canonical([row["fixture_id"] for row in manifest["fixtures"]])).hexdigest() == lock["fixture_ids_sha256"], "lock_ids")
    require(hashlib.sha256(canonical(manifest["files"])).hexdigest() == lock["files_sha256"], "lock_files")
    require(hashlib.sha256(canonical(manifest["authorized_v13_fixture_rebindings"])).hexdigest() == lock["fixture_rebindings_sha256"], "lock_rebindings")
    require(hashlib.sha256(canonical(manifest["profiles"])).hexdigest() == lock["profiles_sha256"], "lock_profiles")
    require(hashlib.sha256(LOCK.read_bytes()).hexdigest() == LOCK_SHA256, "lock_hash")
    candidate = subprocess.run(
        ("git", "rev-parse", lock["source_candidate"] + "^{commit}"),
        cwd=ROOT, capture_output=True, check=False, text=True,
    )
    require(candidate.returncode == 0 and candidate.stdout.strip() == lock["source_candidate"], "lock_candidate")


def self_test(
    manifest: dict[str, object], state: dict[str, object], schema: dict[str, object],
    lock: dict[str, object], lock_schema: dict[str, object],
) -> int:
    cases = []
    for target, mutate in (
        ("manifest", lambda value: value["planned_v14_rebindings"].reverse()),
        ("manifest", lambda value: value["missing_v14_rebindings"].append("unexpected")),
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
    lock_cases = []
    for mutate in (
        lambda value: value.update(extra=False),
        lambda value: value.update(source_candidate="0" * 40),
        lambda value: value.update(manifest_sha256="0" * 64),
        lambda value: value.update(file_count=697),
        lambda value: value.update(files_sha256="0" * 64),
        lambda value: value.update(result_identity_sha256="0" * 64),
        lambda value: (value.update(files_sha256="1" * 64), value.update(result_identity_sha256=hashlib.sha256(canonical({key: value[key] for key in LOCK_KEYS[:-1]})).hexdigest())),
    ):
        changed = copy.deepcopy(lock)
        mutate(changed)
        lock_cases.append(changed)
    for changed in lock_cases:
        try:
            validate_lock(changed, manifest)
        except distribution.DistributionError:
            continue
        raise distribution.DistributionError("mutation_survived:lock")
    schema_cases = []
    for mutate in (
        lambda value: value.update(additionalProperties=True),
        lambda value: value["required"].pop(),
        lambda value: value["properties"].pop("file_count"),
        lambda value: value["properties"]["file_count"].update(const=697),
    ):
        changed = copy.deepcopy(lock_schema)
        mutate(changed)
        schema_cases.append(changed)
    for changed in schema_cases:
        try:
            validate_lock_schema(changed)
        except distribution.DistributionError:
            continue
        raise distribution.DistributionError("mutation_survived:lock_schema")
    return len(cases) + len(lock_cases) + len(schema_cases)


def main() -> int:
    state = json.loads(STATE.read_text(encoding="utf-8"))
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    schema = json.loads(SCHEMA.read_text(encoding="utf-8"))
    lock = json.loads(LOCK.read_text(encoding="utf-8"))
    lock_schema = json.loads(LOCK_SCHEMA.read_text(encoding="utf-8"))
    validate_manifest(manifest, state)
    validate_schema(schema, manifest)
    validate_lock(lock, manifest)
    validate_lock_schema(lock_schema)
    mutations = self_test(manifest, state, schema, lock, lock_schema)
    print(
        "PASS: distribution-v14 transition "
        f"stage={state['current_stage']} scenarios=204 affected=9 mutations={mutations}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
