#!/usr/bin/env python3
"""Validate the closed distribution-v15 budget rebinding authority."""

from __future__ import annotations

import copy
import hashlib
import json
import subprocess
import sys

import generate_distribution_v15 as distribution

sys.dont_write_bytecode = True
ROOT=distribution.ROOT
MANIFEST=ROOT/distribution.OUTPUT_PATH
LOCK=ROOT/distribution.LOCK_PATH
STATE=ROOT/distribution.STATE_PATH
SCHEMA=ROOT/"tools/validation/distribution_v15.schema.json"
LOCK_SCHEMA=ROOT/"tools/validation/distribution_v15_lock.schema.json"
MANIFEST_SHA256="862d0c1ad6ae14cd54b75f88742fa3b584c6c3981195bfeb988818403bee689c"
LOCK_SHA256="a511c18a540aaa5de5a7ef23cf6b360108a74e0e178c1e1025907ae880d78da7"
EXPECTED_LOCK={
    "file_count":725,"files_sha256":"6838a551f33fe9e025b6158708005921fee90befb7793ddd9fb40747312ceedd",
    "fixture_ids_sha256":"523a1c6203080aefc91107f203bb305e9405a800f6c3182de5d4bd73730bf200",
    "fixture_rebindings_sha256":"71dfcbbb865ea7776983838972a8cc32628e33cc14c300680bbbeea8e232b09c",
    "manifest_sha256":MANIFEST_SHA256,"profiles_sha256":"84f80b3a819b70ea943c861dd6636d22c8c66d489c68a422cf372b045e727134",
    "result_identity_sha256":"be61110e2e1c3eb2dc7f30244e07a9efd6d0f4f1beae9693e77441506a35ac92",
    "scenario_count":204,"schema":"nostr_automerge.fixture_distribution_lock.v15.v1",
    "source_candidate":distribution.SOURCE_CANDIDATE,"status":"locked",
}


def require(condition: bool, label: str) -> None:
    if not condition: raise distribution.DistributionError(label)


def canonical(value: object) -> bytes:
    return json.dumps(value,ensure_ascii=False,sort_keys=True,separators=(",", ":")).encode()


def validate(manifest: object, lock: object, state: object, schema: object, lock_schema: object) -> None:
    require(type(state) is dict and distribution.validate_state(state),"state")
    expected=distribution.expected_manifest(state); require(manifest == expected,"manifest:exact")
    require(hashlib.sha256(MANIFEST.read_bytes()).hexdigest() == MANIFEST_SHA256,"manifest:sha")
    require(type(manifest) is dict and len(manifest["fixtures"]) == 204 and len(manifest["files"]) == 725,"manifest:inventory")
    require([row["fixture_id"] for row in manifest["fixtures"]] == sorted({row["fixture_id"] for row in manifest["fixtures"]},key=str.encode),"manifest:fixture_order")
    require([row["path"] for row in manifest["files"]] == sorted({row["path"] for row in manifest["files"]},key=str.encode),"manifest:file_order")
    base=distribution.historical_base(); affected=set(manifest["planned_v15_rebindings"]); base_by={row["fixture_id"]:row for row in base["fixtures"]}; current_by={row["fixture_id"]:row for row in manifest["fixtures"]}
    require(all(current_by[key] == row for key,row in base_by.items() if key not in affected),"manifest:unaffected")
    require(type(lock) is dict and lock == EXPECTED_LOCK and hashlib.sha256(LOCK.read_bytes()).hexdigest() == LOCK_SHA256,"lock:exact")
    require(lock == distribution.expected_lock(MANIFEST.read_bytes(),manifest),"lock:derived")
    resolved=subprocess.run(["git","rev-parse","--verify",lock["source_candidate"]+"^{commit}"],cwd=ROOT,capture_output=True,text=True,check=False)
    require(resolved.returncode == 0 and resolved.stdout.strip() == lock["source_candidate"],"lock:candidate")
    required=list(manifest); require(type(schema) is dict and schema.get("additionalProperties") is False and schema.get("required") == required and list(schema.get("properties",{})) == required,"schema:closed")
    require(type(lock_schema) is dict and lock_schema.get("additionalProperties") is False and lock_schema.get("required") == list(distribution.LOCK_KEYS),"lock_schema:closed")


def self_test(manifest: dict, lock: dict, state: dict, schema: dict, lock_schema: dict) -> int:
    cases=[
        ("manifest",lambda value:value["fixtures"].pop()),("manifest",lambda value:value["files"].reverse()),
        ("manifest",lambda value:value["planned_v15_rebindings"].reverse()),("manifest",lambda value:value["authorized_v14_fixture_rebindings"][0].update(required_max_items=0)),
        ("manifest",lambda value:value.update(extra=False)),("state",lambda value:value["affected_fixture_ids"].reverse()),
        ("state",lambda value:value.update(v14_files_preserved=False)),("lock",lambda value:value.update(manifest_sha256="0"*64)),
        ("lock",lambda value:value.update(result_identity_sha256="0"*64)),("schema",lambda value:value.update(additionalProperties=True)),
        ("lock_schema",lambda value:value.update(additionalProperties=True)),
    ]
    caught=0
    for target,mutate in cases:
        values={"manifest":copy.deepcopy(manifest),"lock":copy.deepcopy(lock),"state":copy.deepcopy(state),"schema":copy.deepcopy(schema),"lock_schema":copy.deepcopy(lock_schema)}; mutate(values[target])
        try: validate(**values)
        except distribution.DistributionError: caught+=1; continue
        raise distribution.DistributionError("mutation_survived:"+target)
    return caught


def main() -> int:
    manifest=json.loads(MANIFEST.read_text()); lock=json.loads(LOCK.read_text()); state=json.loads(STATE.read_text()); schema=json.loads(SCHEMA.read_text()); lock_schema=json.loads(LOCK_SCHEMA.read_text())
    validate(manifest,lock,state,schema,lock_schema); mutations=self_test(manifest,lock,state,schema,lock_schema)
    print(f"PASS: distribution-v15 transition scenarios=204 affected=9 mutations={mutations}")
    return 0


if __name__ == "__main__": raise SystemExit(main())
