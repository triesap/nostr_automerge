#!/usr/bin/env python3
"""Generate the immutable source-derived signed distribution-v16 transition."""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import subprocess
import sys
from typing import Any

sys.dont_write_bytecode = True
ROOT = pathlib.Path(__file__).resolve().parents[1]
STATE_PATH = "spec/distribution_v16_transition.json"
STATE_SCHEMA_PATH = "tools/validation/distribution_v16_transition.schema.json"
BASE_PATH = "fixtures/distribution/manifest_v15.json"
OUTPUT_PATH = "fixtures/distribution/manifest_v16.json"
LOCK_PATH = "fixtures/distribution/manifest_v16.lock.json"
REBINDING_ROOT = ROOT / "fixtures/v16/rebindings/causal_projection"
SOURCE_CANDIDATE = "d2653edc718b002b7fe13b18d01bfe09df1fa02d"
BASE_SHA256 = "862d0c1ad6ae14cd54b75f88742fa3b584c6c3981195bfeb988818403bee689c"
MEASUREMENT_SHA256 = "d8b16aa333f870d8a500da99e414d990077310be16b333669805cfa000f80e1f"
STATE_KEYS = (
    "schema","current_stage","stage_order","base_manifest","base_manifest_sha256",
    "source_candidate","measurement_manifest","measurement_count","measurement_sha256",
    "scenario_count","signed_event_count","delivery_order_count","changed_budget_rows",
    "affected_fixture_count","unaffected_fixture_count","signed_events_preserved",
    "ample_work_reports_preserved","v15_files_preserved","result",
)
LOCK_KEYS = (
    "schema","status","source_candidate","manifest_sha256","scenario_count",
    "signed_event_count","file_count","fixture_ids_sha256","files_sha256",
    "fixture_rebindings_sha256","measurement_sha256","profiles_sha256",
    "result_identity_sha256",
)


class DistributionError(ValueError):
    pass


def require(condition: bool, label: str) -> None:
    if not condition:
        raise DistributionError(label)


def canonical_json(value: Any) -> bytes:
    return (json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n").encode()


def canonical(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()


def digest(path: str) -> str:
    return hashlib.sha256((ROOT / path).read_bytes()).hexdigest()


def load(path: str) -> dict[str, Any]:
    value = json.loads((ROOT / path).read_text())
    require(type(value) is dict, "object:" + path)
    return value


def historical_base() -> dict[str, Any]:
    completed = subprocess.run(
        ("git", "show", f"{SOURCE_CANDIDATE}:{BASE_PATH}"),
        cwd=ROOT,
        capture_output=True,
        check=False,
    )
    require(
        completed.returncode == 0
        and hashlib.sha256(completed.stdout).hexdigest() == BASE_SHA256,
        "base_candidate",
    )
    require((ROOT / BASE_PATH).read_bytes() == completed.stdout, "base_drift")
    value = json.loads(completed.stdout)
    require(type(value) is dict and value.get("fixture_count") == 204, "base_identity")
    return value


def changed_rows(state: dict[str, Any]) -> list[dict[str, Any]]:
    rows = state["changed_budget_rows"]
    require(
        type(rows) is list
        and len(rows) == 8
        and [row["fixture_id"] for row in rows]
        == sorted({row["fixture_id"] for row in rows}, key=str.encode),
        "state_changed_rows",
    )
    for row in rows:
        require(
            type(row) is dict
            and list(row) == ["fixture_id", "prior_max_items", "required_max_items"]
            and type(row["prior_max_items"]) is int
            and type(row["required_max_items"]) is int
            and row["prior_max_items"] != row["required_max_items"],
            "state_changed_row",
        )
    return rows


def validate_state(state: object, schema: object) -> bool:
    require(type(state) is dict and tuple(state) == STATE_KEYS, "state_shape")
    assert isinstance(state, dict)
    require(state["schema"] == "nostr_automerge.distribution_v16_transition.v1", "state_schema")
    require(
        state["current_stage"] in ("authority_defined", "distribution_complete")
        and state["stage_order"] == ["authority_defined", "distribution_complete"],
        "state_stage",
    )
    require(
        state["base_manifest"] == BASE_PATH
        and state["base_manifest_sha256"] == BASE_SHA256
        and state["source_candidate"] == SOURCE_CANDIDATE,
        "state_base",
    )
    require(
        state["measurement_manifest"] == BASE_PATH
        and state["measurement_count"] == 204
        and state["measurement_sha256"] == MEASUREMENT_SHA256,
        "state_measurement",
    )
    rows = changed_rows(state)
    require(
        state["scenario_count"] == 204
        and state["signed_event_count"] == 771
        and state["delivery_order_count"] == 8
        and state["affected_fixture_count"] == len(rows)
        and state["unaffected_fixture_count"] == 204 - len(rows),
        "state_inventory",
    )
    require(
        state["signed_events_preserved"] is True
        and state["ample_work_reports_preserved"] is True
        and state["v15_files_preserved"] is True
        and state["result"] == "pass",
        "state_preservation",
    )
    require(
        type(schema) is dict
        and schema.get("additionalProperties") is False
        and schema.get("required") == list(STATE_KEYS)
        and list(schema.get("properties", {})) == list(STATE_KEYS)
        and schema["$defs"]["rebinding"].get("additionalProperties") is False,
        "state_schema_closed",
    )
    return state["current_stage"] == "distribution_complete"


def ordered_projection(rows: list[dict[str, str]]) -> str:
    value = hashlib.sha256()
    for row in rows:
        for key in ("path", "sha256"):
            item = row[key].encode()
            value.update(len(item).to_bytes(8, "big") + item)
    return value.hexdigest()


def materialize_rebindings(state: dict[str, Any]) -> None:
    base = historical_base()
    by_id = {row["fixture_id"]: row for row in base["fixtures"]}
    REBINDING_ROOT.mkdir(parents=True, exist_ok=True)
    for row in changed_rows(state):
        fixture_id = row["fixture_id"]
        prior = by_id[fixture_id]
        prior_input = load(prior["input_paths"][0])
        require(prior_input["budget"]["max_items"] == row["prior_max_items"], "prior_budget:" + fixture_id)
        current = json.loads(json.dumps(prior_input))
        current["budget"]["max_items"] = row["required_max_items"]
        input_bytes = canonical_json(current)
        expected_bytes = (ROOT / prior["expected_path"]).read_bytes()
        metadata = json.loads(json.dumps(load(prior["metadata_path"])))
        metadata["inputs"][0]["path"] = fixture_id + ".input.json"
        metadata["inputs"][0]["sha256"] = hashlib.sha256(input_bytes).hexdigest()
        metadata["expected"]["report_path"] = fixture_id + ".expected.json"
        metadata["expected"]["sha256"] = hashlib.sha256(expected_bytes).hexdigest()
        metadata["provenance"]["generator"] = "nostr_automerge distribution-v16 source-derived budget rebinding"
        (REBINDING_ROOT / (fixture_id + ".input.json")).write_bytes(input_bytes)
        (REBINDING_ROOT / (fixture_id + ".expected.json")).write_bytes(expected_bytes)
        (REBINDING_ROOT / (fixture_id + ".fixture.json")).write_bytes(canonical_json(metadata))


def rebindings(state: dict[str, Any]) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    base = historical_base()
    by_id = {row["fixture_id"]: row for row in base["fixtures"]}
    entries: list[dict[str, Any]] = []
    records: list[dict[str, Any]] = []
    for changed in changed_rows(state):
        fixture_id = changed["fixture_id"]
        prior = by_id[fixture_id]
        root = f"fixtures/v16/rebindings/causal_projection/{fixture_id}"
        metadata = load(root + ".fixture.json")
        current = load(root + ".input.json")
        prior_input = load(prior["input_paths"][0])
        projected = json.loads(json.dumps(prior_input))
        projected["budget"]["max_items"] = changed["required_max_items"]
        require(prior_input["budget"]["max_items"] == changed["prior_max_items"] and current == projected, "budget_only:" + fixture_id)
        require((ROOT / (root + ".expected.json")).read_bytes() == (ROOT / prior["expected_path"]).read_bytes(), "report_bytes:" + fixture_id)
        require(current["raw_events"] == prior_input["raw_events"] and current.get("delivery_orders") == prior_input.get("delivery_orders"), "input_identity:" + fixture_id)
        require(metadata["inputs"][0]["sha256"] == digest(root + ".input.json") and metadata["expected"]["sha256"] == digest(root + ".expected.json"), "metadata:" + fixture_id)
        entry = dict(prior)
        entry["metadata_path"] = root + ".fixture.json"
        entry["input_paths"] = [root + ".input.json"]
        entry["expected_path"] = root + ".expected.json"
        entries.append(entry)
        records.append({
            "fixture_id": fixture_id,
            "prior_metadata_path": prior["metadata_path"],
            "current_metadata_path": root + ".fixture.json",
            "prior_max_items": changed["prior_max_items"],
            "required_max_items": changed["required_max_items"],
            "raw_events_preserved": True,
            "ample_work_report_preserved": True,
            "delivery_orders_identical": True,
        })
    return entries, records


def expected_manifest(state: dict[str, Any], schema: dict[str, Any]) -> dict[str, Any]:
    complete = validate_state(state, schema)
    base = historical_base()
    fixtures = [dict(row) for row in base["fixtures"]]
    files = [dict(row) for row in base["files"]]
    records: list[dict[str, Any]] = []
    if complete:
        rebound, records = rebindings(state)
        by_id = {row["fixture_id"]: row for row in rebound}
        fixtures = [by_id.get(row["fixture_id"], row) for row in fixtures]
        for row in rebound:
            for path in (*row["input_paths"], row["expected_path"], row["metadata_path"]):
                files.append({"path": path, "sha256": digest(path)})
    fixtures.sort(key=lambda row: row["fixture_id"].encode())
    files.sort(key=lambda row: row["path"].encode())
    affected = [row["fixture_id"] for row in changed_rows(state)]
    return {
        "authorized_v15_fixture_rebindings": records,
        "base_manifest_sha256": BASE_SHA256,
        "complete": complete,
        "derivation_measurement_sha256": MEASUREMENT_SHA256,
        "distribution_id": "draft_2026_08_signed_neutral_16",
        "distribution_schema": "nostr_automerge.fixture_distribution.v16",
        "files": files,
        "fixture_count": 204,
        "fixtures": fixtures,
        "missing_v16_rebindings": [] if complete else affected,
        "planned_v16_rebindings": affected,
        "preserved_v15_file_count": 725,
        "preserved_v15_files_sha256": ordered_projection(base["files"]),
        "preserved_v15_fixture_count": 204,
        "profiles": {key: list(value) for key, value in base["profiles"].items()},
        "protocol_revision": "draft_2026_08",
        "requirements_sha256": base["requirements_sha256"],
        "status": "canonical_signed_neutral_corpus" if complete else "locked_transition",
        "supersedes": BASE_PATH,
        "target_fixture_count": 204,
        "transition_stage": state["current_stage"],
    }


def expected_lock(manifest_bytes: bytes, manifest: dict[str, Any]) -> dict[str, Any]:
    value = {
        "schema":"nostr_automerge.fixture_distribution_lock.v16.v1",
        "status":"locked",
        "source_candidate":SOURCE_CANDIDATE,
        "manifest_sha256":hashlib.sha256(manifest_bytes).hexdigest(),
        "scenario_count":204,
        "signed_event_count":771,
        "file_count":749,
        "fixture_ids_sha256":hashlib.sha256(canonical([row["fixture_id"] for row in manifest["fixtures"]])).hexdigest(),
        "files_sha256":hashlib.sha256(canonical(manifest["files"])).hexdigest(),
        "fixture_rebindings_sha256":hashlib.sha256(canonical(manifest["authorized_v15_fixture_rebindings"])).hexdigest(),
        "measurement_sha256":MEASUREMENT_SHA256,
        "profiles_sha256":hashlib.sha256(canonical(manifest["profiles"])).hexdigest(),
        "result_identity_sha256":"",
    }
    value["result_identity_sha256"] = hashlib.sha256(canonical({key: value[key] for key in LOCK_KEYS[:-1]})).hexdigest()
    return value


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write", action="store_true")
    parser.add_argument("--materialize", action="store_true")
    args = parser.parse_args()
    state = load(STATE_PATH)
    schema = load(STATE_SCHEMA_PATH)
    if args.materialize:
        require(validate_state(state, schema), "materialize_stage")
        materialize_rebindings(state)
    manifest = expected_manifest(state, schema)
    manifest_bytes = canonical_json(manifest)
    if args.write:
        (ROOT / OUTPUT_PATH).write_bytes(manifest_bytes)
        (ROOT / LOCK_PATH).write_bytes(canonical_json(expected_lock(manifest_bytes, manifest)))
    else:
        require((ROOT / OUTPUT_PATH).read_bytes() == manifest_bytes, "manifest_bytes")
        require(json.loads((ROOT / LOCK_PATH).read_text()) == expected_lock(manifest_bytes, manifest), "lock_bytes")
    print(f"PASS: generated distribution-v16 scenarios=204 affected={len(changed_rows(state))} manifest_sha256={hashlib.sha256(manifest_bytes).hexdigest()}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
