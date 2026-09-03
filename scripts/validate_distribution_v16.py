#!/usr/bin/env python3
"""Validate the closed, source-derived distribution-v16 authority."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import subprocess
import sys

import generate_distribution_v16 as distribution

sys.dont_write_bytecode = True
ROOT = distribution.ROOT
MANIFEST = ROOT / distribution.OUTPUT_PATH
LOCK = ROOT / distribution.LOCK_PATH
STATE = ROOT / distribution.STATE_PATH
STATE_SCHEMA = ROOT / distribution.STATE_SCHEMA_PATH
SCHEMA = ROOT / "tools/validation/distribution_v16.schema.json"
LOCK_SCHEMA = ROOT / "tools/validation/distribution_v16_lock.schema.json"
MANIFEST_SHA256 = "7890fe2532da48ca84e54f5b1b883a38fd1a60ff58bb2999a056025335a4b5d3"
LOCK_SHA256 = "9e09dfd2de706d320c3bcd7cfe45b2f9a7560d5e9354809d2a41e5f52a2fba90"
EXPECTED_LOCK = {
    "file_count": 749,
    "files_sha256": "c4074f2573f1d01ef69e0c9f33c3d36eb2800146b2b4349983be77ba5f0c23c7",
    "fixture_ids_sha256": "523a1c6203080aefc91107f203bb305e9405a800f6c3182de5d4bd73730bf200",
    "fixture_rebindings_sha256": "7e37d4aacef2cd68fde96f7842f755eb30b99d10539326fb2d57708e76acd695",
    "manifest_sha256": MANIFEST_SHA256,
    "measurement_sha256": distribution.MEASUREMENT_SHA256,
    "profiles_sha256": "84f80b3a819b70ea943c861dd6636d22c8c66d489c68a422cf372b045e727134",
    "result_identity_sha256": "3b360ceab9d24c2baacf9cd4fd594ebf65a62717b1799a080db6b0c0b4e81318",
    "scenario_count": 204,
    "schema": "nostr_automerge.fixture_distribution_lock.v16.v1",
    "signed_event_count": 771,
    "source_candidate": distribution.SOURCE_CANDIDATE,
    "status": "locked",
}


def require(condition: bool, label: str) -> None:
    if not condition:
        raise distribution.DistributionError(label)


def load(path: object) -> object:
    return json.loads(path.read_text())


def validate(manifest: object, lock: object, state: object, state_schema: object,
             schema: object, lock_schema: object) -> None:
    require(distribution.validate_state(state, state_schema), "state")
    assert isinstance(state, dict)
    expected = distribution.expected_manifest(state, state_schema)
    require(manifest == expected, "manifest:exact")
    require(hashlib.sha256(MANIFEST.read_bytes()).hexdigest() == MANIFEST_SHA256, "manifest:sha")
    require(type(manifest) is dict and len(manifest["fixtures"]) == 204 and len(manifest["files"]) == 749, "manifest:inventory")
    require([row["fixture_id"] for row in manifest["fixtures"]] == sorted({row["fixture_id"] for row in manifest["fixtures"]}, key=str.encode), "manifest:fixture_order")
    require([row["path"] for row in manifest["files"]] == sorted({row["path"] for row in manifest["files"]}, key=str.encode), "manifest:file_order")
    require(all(hashlib.sha256((ROOT / row["path"]).read_bytes()).hexdigest() == row["sha256"] for row in manifest["files"]), "manifest:file_hash")
    base = distribution.historical_base()
    affected = set(manifest["planned_v16_rebindings"])
    base_by = {row["fixture_id"]: row for row in base["fixtures"]}
    current_by = {row["fixture_id"]: row for row in manifest["fixtures"]}
    require(all(current_by[key] == row for key, row in base_by.items() if key not in affected), "manifest:unaffected")
    require(type(lock) is dict and lock == EXPECTED_LOCK and hashlib.sha256(LOCK.read_bytes()).hexdigest() == LOCK_SHA256, "lock:exact")
    require(lock == distribution.expected_lock(MANIFEST.read_bytes(), manifest), "lock:derived")
    resolved = subprocess.run(["git", "rev-parse", "--verify", lock["source_candidate"] + "^{commit}"], cwd=ROOT, capture_output=True, text=True, check=False)
    require(resolved.returncode == 0 and resolved.stdout.strip() == lock["source_candidate"], "lock:candidate")
    required = list(manifest)
    require(type(schema) is dict and schema.get("additionalProperties") is False and schema.get("required") == required and list(schema.get("properties", {})) == required, "schema:closed")
    require(type(lock_schema) is dict and lock_schema.get("additionalProperties") is False and lock_schema.get("required") == list(distribution.LOCK_KEYS) and list(lock_schema.get("properties", {})) == list(distribution.LOCK_KEYS), "lock_schema:closed")


def derive(state: dict[str, object]) -> None:
    completed = subprocess.run(
        ["cargo", "run", "--quiet", "-p", "nostr_automerge_conformance", "--locked", "--", "derive_distribution_items", distribution.BASE_PATH],
        cwd=ROOT, capture_output=True, check=True,
    )
    require(hashlib.sha256(completed.stdout).hexdigest() == distribution.MEASUREMENT_SHA256, "derive:sha")
    measured = json.loads(completed.stdout)
    require(measured["scenario_count"] == 204 and measured["signed_event_count"] == 771 and measured["delivery_order_count"] == 8, "derive:inventory")
    authority_ids = [row["fixture_id"] for row in distribution.historical_base()["authorized_v14_fixture_rebindings"]]
    by_id = {row["fixture_id"]: row for row in measured["measurements"]}
    rows = [{"fixture_id": key, "prior_max_items": by_id[key]["configured_max_items"], "required_max_items": by_id[key]["required_max_items"]} for key in authority_ids if by_id[key]["configured_max_items"] != by_id[key]["required_max_items"]]
    require(rows == state["changed_budget_rows"], "derive:rows")


def self_test(manifest: dict, lock: dict, state: dict, state_schema: dict,
              schema: dict, lock_schema: dict) -> int:
    cases = [
        ("manifest", lambda value: value["fixtures"].pop()),
        ("manifest", lambda value: value["files"].reverse()),
        ("manifest", lambda value: value["planned_v16_rebindings"].reverse()),
        ("manifest", lambda value: value["authorized_v15_fixture_rebindings"][0].update(required_max_items=0)),
        ("manifest", lambda value: value.update(extra=False)),
        ("state", lambda value: value["changed_budget_rows"].reverse()),
        ("state", lambda value: value["changed_budget_rows"][0].update(required_max_items=0)),
        ("state", lambda value: value.update(v15_files_preserved=False)),
        ("lock", lambda value: value.update(manifest_sha256="0" * 64)),
        ("lock", lambda value: value.update(measurement_sha256="0" * 64)),
        ("lock", lambda value: value.update(result_identity_sha256="0" * 64)),
        ("state_schema", lambda value: value.update(additionalProperties=True)),
        ("schema", lambda value: value.update(additionalProperties=True)),
        ("lock_schema", lambda value: value.update(additionalProperties=True)),
    ]
    originals = {"manifest": manifest, "lock": lock, "state": state, "state_schema": state_schema, "schema": schema, "lock_schema": lock_schema}
    caught = 0
    for target, mutate in cases:
        values = copy.deepcopy(originals)
        mutate(values[target])
        try:
            validate(**values)
        except distribution.DistributionError:
            caught += 1
            continue
        raise distribution.DistributionError("mutation_survived:" + target)
    return caught


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--run-derivation", action="store_true")
    args = parser.parse_args()
    values = {"manifest": load(MANIFEST), "lock": load(LOCK), "state": load(STATE), "state_schema": load(STATE_SCHEMA), "schema": load(SCHEMA), "lock_schema": load(LOCK_SCHEMA)}
    validate(**values)
    mutations = self_test(**values)
    if args.run_derivation:
        derive(values["state"])
    print(f"PASS: distribution-v16 scenarios=204 affected=8 mutations={mutations} derivation={1 if args.run_derivation else 0}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
