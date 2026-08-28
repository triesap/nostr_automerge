#!/usr/bin/env python3
"""Generate or check the budget-only signed distribution-v14 transition."""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import subprocess
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parents[1]
STATE_PATH = "spec/distribution_v14_transition.json"
BASE_PATH = "fixtures/distribution/manifest_v13.json"
OUTPUT_PATH = "fixtures/distribution/manifest_v14.json"
REBINDING_ROOT = ROOT / "fixtures/v14/rebindings/causal_projection"
BASE_CANDIDATE = "73ce3be33ddd1beba6528fb9f61a533e5d571cc6"
BASE_SHA256 = "12aa1b1f806ce810463768d566cc63d2ceba6126014d4da9fe5688df518bef3f"
AFFECTED = (
    ("canonical_derivation_exact_budget", 453, 455),
    ("deep_actor_predecessor_exact_budget", 1966, 2104),
    ("deep_delta_absent_lookup_exact_budget", 9081, 10162),
    ("deep_delta_extend_exact_budget", 9300, 10381),
    ("deep_delta_root_lookup_exact_budget", 9913, 10994),
    ("empty_merge_frontier_exact_budget", 1894, 2019),
    ("epoch_writer_authorization_exact_budget", 38096, 38156),
    ("many_actor_causal_next_op_exact_budget", 4744, 5328),
    ("wide_epoch_ancestry_exact_budget", 14377, 15230),
)
STATE_KEYS = (
    "schema", "current_stage", "stage_order", "base_manifest",
    "base_manifest_sha256", "scenario_count", "affected_fixture_ids",
    "unaffected_fixture_count", "signed_events_preserved",
    "ample_work_reports_preserved", "result",
)


class DistributionError(ValueError):
    pass


def require(condition: bool, label: str) -> None:
    if not condition:
        raise DistributionError(label)


def canonical_json(value: Any) -> bytes:
    return (json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n").encode()


def digest(path: str) -> str:
    return hashlib.sha256((ROOT / path).read_bytes()).hexdigest()


def load(path: str) -> dict[str, Any]:
    value = json.loads((ROOT / path).read_text(encoding="utf-8"))
    require(type(value) is dict, "object:" + path)
    return value


def historical_base() -> dict[str, Any]:
    completed = subprocess.run(
        ("git", "show", f"{BASE_CANDIDATE}:{BASE_PATH}"),
        cwd=ROOT,
        capture_output=True,
        check=False,
    )
    require(completed.returncode == 0 and completed.stderr == b"", "base_candidate")
    require(hashlib.sha256(completed.stdout).hexdigest() == BASE_SHA256, "base_hash")
    require((ROOT / BASE_PATH).read_bytes() == completed.stdout, "base_drift")
    value = json.loads(completed.stdout)
    require(type(value) is dict and value.get("fixture_count") == 204, "base_identity")
    return value


def validate_state(state: object) -> bool:
    require(type(state) is dict and tuple(state) == STATE_KEYS, "state_shape")
    assert isinstance(state, dict)
    require(state["schema"] == "nostr_automerge.distribution_v14_transition.v1", "state_schema")
    require(state["current_stage"] in ("authority_defined", "distribution_complete"), "state_stage")
    require(state["stage_order"] == ["authority_defined", "distribution_complete"], "state_order")
    require(state["base_manifest"] == BASE_PATH and state["base_manifest_sha256"] == BASE_SHA256, "state_base")
    require(state["scenario_count"] == 204, "state_count")
    require(state["affected_fixture_ids"] == [row[0] for row in AFFECTED], "state_affected")
    require(state["unaffected_fixture_count"] == 195, "state_unaffected")
    require(state["signed_events_preserved"] is True, "state_events")
    require(state["ample_work_reports_preserved"] is True, "state_reports")
    require(state["result"] == "pass", "state_result")
    return state["current_stage"] == "distribution_complete"


def ordered_projection(rows: list[dict[str, str]]) -> str:
    state = hashlib.sha256()
    for row in rows:
        for key in ("path", "sha256"):
            value = row[key].encode()
            state.update(len(value).to_bytes(8, "big") + value)
    return state.hexdigest()


def rebindings() -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    entries: list[dict[str, Any]] = []
    records: list[dict[str, Any]] = []
    base = historical_base()
    by_id = {row["fixture_id"]: row for row in base["fixtures"]}
    for fixture_id, old_budget, new_budget in AFFECTED:
        prior = by_id[fixture_id]
        root = f"fixtures/v14/rebindings/causal_projection/{fixture_id}"
        metadata = load(root + ".fixture.json")
        input_value = load(root + ".input.json")
        prior_input = load(prior["input_paths"][0])
        projected = json.loads(json.dumps(prior_input))
        require(projected["budget"]["max_items"] == old_budget, "prior_budget:" + fixture_id)
        projected["budget"]["max_items"] = new_budget
        require(input_value == projected, "budget_only:" + fixture_id)
        require((ROOT / (root + ".expected.json")).read_bytes() == (ROOT / prior["expected_path"]).read_bytes(), "report_bytes:" + fixture_id)
        require(input_value["raw_events"] == prior_input["raw_events"], "event_bytes:" + fixture_id)
        require(input_value.get("delivery_orders") == prior_input.get("delivery_orders"), "delivery_orders:" + fixture_id)
        require(metadata["fixture_id"] == fixture_id, "metadata_id:" + fixture_id)
        require(metadata["inputs"][0]["sha256"] == digest(root + ".input.json"), "metadata_input:" + fixture_id)
        require(metadata["expected"]["sha256"] == digest(root + ".expected.json"), "metadata_report:" + fixture_id)
        entry = dict(prior)
        entry["metadata_path"] = root + ".fixture.json"
        entry["input_paths"] = [root + ".input.json"]
        entry["expected_path"] = root + ".expected.json"
        entries.append(entry)
        records.append({
            "fixture_id": fixture_id,
            "prior_metadata_path": prior["metadata_path"],
            "current_metadata_path": root + ".fixture.json",
            "prior_max_items": old_budget,
            "required_max_items": new_budget,
            "raw_events_preserved": True,
            "ample_work_report_preserved": True,
            "delivery_orders_identical": True,
        })
    return entries, records


def materialize_rebindings() -> None:
    base = historical_base()
    by_id = {row["fixture_id"]: row for row in base["fixtures"]}
    REBINDING_ROOT.mkdir(parents=True, exist_ok=True)
    for fixture_id, old_budget, new_budget in AFFECTED:
        prior = by_id[fixture_id]
        prior_input = load(prior["input_paths"][0])
        require(prior_input["budget"]["max_items"] == old_budget, "materialize_prior:" + fixture_id)
        current_input = json.loads(json.dumps(prior_input))
        current_input["budget"]["max_items"] = new_budget
        input_bytes = canonical_json(current_input)
        expected_bytes = (ROOT / prior["expected_path"]).read_bytes()
        prior_metadata = load(prior["metadata_path"])
        metadata = json.loads(json.dumps(prior_metadata))
        metadata["inputs"][0]["path"] = fixture_id + ".input.json"
        metadata["inputs"][0]["sha256"] = hashlib.sha256(input_bytes).hexdigest()
        metadata["expected"]["report_path"] = fixture_id + ".expected.json"
        metadata["expected"]["sha256"] = hashlib.sha256(expected_bytes).hexdigest()
        metadata["provenance"]["generator"] = "nostr_automerge distribution-v14 budget rebinding"
        (REBINDING_ROOT / (fixture_id + ".input.json")).write_bytes(input_bytes)
        (REBINDING_ROOT / (fixture_id + ".expected.json")).write_bytes(expected_bytes)
        (REBINDING_ROOT / (fixture_id + ".fixture.json")).write_bytes(canonical_json(metadata))


def expected_manifest(state: dict[str, Any]) -> dict[str, Any]:
    complete = validate_state(state)
    base = historical_base()
    fixtures = [dict(row) for row in base["fixtures"]]
    files = [dict(row) for row in base["files"]]
    profiles = {key: list(value) for key, value in base["profiles"].items()}
    records: list[dict[str, Any]] = []
    if complete:
        rebound, records = rebindings()
        by_id = {row["fixture_id"]: row for row in rebound}
        fixtures = [by_id.get(row["fixture_id"], row) for row in fixtures]
        for row in rebound:
            for path in (*row["input_paths"], row["expected_path"], row["metadata_path"]):
                files.append({"path": path, "sha256": digest(path)})
    fixtures.sort(key=lambda row: row["fixture_id"].encode())
    files.sort(key=lambda row: row["path"].encode())
    missing = [] if complete else [row[0] for row in AFFECTED]
    return {
        "authorized_v13_fixture_rebindings": records,
        "base_manifest_sha256": BASE_SHA256,
        "complete": complete,
        "distribution_id": "draft_2026_08_signed_neutral_14",
        "distribution_schema": "nostr_automerge.fixture_distribution.v14",
        "files": files,
        "fixture_count": 204,
        "fixtures": fixtures,
        "missing_v14_rebindings": missing,
        "planned_v14_rebindings": [row[0] for row in AFFECTED],
        "preserved_v13_file_count": 671,
        "preserved_v13_files_sha256": ordered_projection(base["files"]),
        "preserved_v13_fixture_count": 204,
        "profiles": profiles,
        "protocol_revision": "draft_2026_08",
        "requirements_sha256": base["requirements_sha256"],
        "status": "canonical_signed_neutral_corpus" if complete else "locked_transition",
        "supersedes": BASE_PATH,
        "target_fixture_count": 204,
        "transition_stage": state["current_stage"],
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write", action="store_true")
    parser.add_argument("--materialize", action="store_true")
    args = parser.parse_args()
    state = load(STATE_PATH)
    if args.materialize:
        require(validate_state(state), "materialize_stage")
        materialize_rebindings()
    expected = canonical_json(expected_manifest(state))
    output = ROOT / OUTPUT_PATH
    if args.write:
        output.write_bytes(expected)
    else:
        require(output.is_file() and output.read_bytes() == expected, "manifest_bytes")
    print("PASS: generated distribution-v14 transition")
    print(f"- stage={state['current_stage']} fixtures=204 affected={len(AFFECTED)}")
    print(f"- manifest_sha256={hashlib.sha256(expected).hexdigest()}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
