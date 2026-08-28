#!/usr/bin/env python3
"""Generate or check the append-only signed distribution-v13 transition."""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import subprocess
from typing import Any


ROOT = pathlib.Path(__file__).resolve().parents[1]
STATE_PATH = "spec/distribution_v13_transition.json"
BASE_PATH = "fixtures/distribution/manifest_v12.json"
OUTPUT_PATH = "fixtures/distribution/manifest_v13.json"
SCHEMA_PATH = "tools/validation/distribution_v13.schema.json"
COMPANION_ROOT = ROOT / "fixtures/v13/scenarios/epoch_semantics"
BASE_CANDIDATE = "de716296d88b9908e350ec2eb7bc9406573a2a5d"
BASE_SHA256 = "29d1304aae027d33ff66b39b2cc499cca0e40fb24e5d4f5d749e33bf7dafd7c0"
OLD_REQUIREMENTS_SHA256 = "840822a1acf171c887b9a9aba79ddf159ffcd9c5d7a74bd74d7e0bac5c6161f4"
NEW_REQUIREMENTS_SHA256 = "a8926ae4610b4855294f769871e87a14dee73d05ed201419de35711a8a781974"
PLAN = (
    ("deep_actor_predecessor_exact_budget", ("NCRDT-RESOURCE-017", "NCRDT-RESOURCE-018")),
    ("many_actor_causal_next_op_exact_budget", ("NCRDT-RESOURCE-018",)),
    ("empty_merge_frontier_exact_budget", ("NCRDT-RESOURCE-017", "NCRDT-RESOURCE-018")),
    ("wide_epoch_ancestry_exact_budget", ("NCRDT-RESOURCE-019",)),
    ("epoch_writer_authorization_exact_budget", ("NCRDT-RESOURCE-017",)),
    ("post_epoch_semantic_stop_has_no_target_work", ("NCRDT-RESOURCE-017", "NCRDT-COMPLETION-001")),
)
STAGE_COUNTS = {
    "authority_defined": 0,
    "inventory_installed": 0,
    "deep_actor_fixture_added": 1,
    "many_actor_fixture_added": 2,
    "empty_frontier_fixture_added": 3,
    "wide_ancestry_fixture_added": 4,
    "writer_authorization_fixture_added": 5,
    "post_stop_fixture_added": 6,
    "distribution_complete": 6,
}
STAGE_ORDER = tuple(STAGE_COUNTS)
STATE_KEYS = (
    "schema", "current_stage", "stage_order", "base_manifest",
    "base_manifest_sha256", "preserved_fixture_count", "target_fixture_count",
    "planned_v13_fixtures", "requirements_sha256", "result",
)


class DistributionError(ValueError):
    pass


def require(condition: bool, diagnostic: str) -> None:
    if not condition:
        raise DistributionError(diagnostic)


def digest(relative: str) -> str:
    return hashlib.sha256((ROOT / relative).read_bytes()).hexdigest()


def load(relative: str) -> dict[str, Any]:
    try:
        value = json.loads((ROOT / relative).read_text())
    except (OSError, json.JSONDecodeError) as error:
        raise DistributionError("json:" + relative) from error
    require(type(value) is dict, "object:" + relative)
    return value


def canonical_json(value: Any) -> bytes:
    return (json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n").encode()


def historical_base() -> tuple[dict[str, Any], bytes]:
    completed = subprocess.run(
        ("git", "show", f"{BASE_CANDIDATE}:{BASE_PATH}"),
        cwd=ROOT, capture_output=True, check=False,
    )
    require(completed.returncode == 0 and completed.stderr == b"", "base_candidate")
    require(hashlib.sha256(completed.stdout).hexdigest() == BASE_SHA256, "base_hash")
    require((ROOT / BASE_PATH).read_bytes() == completed.stdout, "base_drift")
    base = json.loads(completed.stdout)
    require(type(base) is dict and base.get("fixture_count") == 198, "base_identity")
    require(base.get("complete") is True, "base_complete")
    return base, completed.stdout


def ordered_projection(rows: list[dict[str, str]]) -> str:
    state = hashlib.sha256()
    for row in rows:
        path = row["path"].encode()
        value = row["sha256"].encode()
        state.update(len(path).to_bytes(8, "big") + path)
        state.update(len(value).to_bytes(8, "big") + value)
    return state.hexdigest()


def validate_state(state: object) -> tuple[str, int]:
    require(type(state) is dict and tuple(state) == STATE_KEYS, "state_shape")
    assert isinstance(state, dict)
    require(state["schema"] == "nostr_automerge.distribution_v13_transition.v1", "state_schema")
    stage = state["current_stage"]
    require(type(stage) is str and stage in STAGE_COUNTS, "state_stage")
    require(tuple(state["stage_order"]) == STAGE_ORDER, "state_order")
    require(state["base_manifest"] == BASE_PATH and state["base_manifest_sha256"] == BASE_SHA256, "state_base")
    require(state["preserved_fixture_count"] == 198 and state["target_fixture_count"] == 204, "state_counts")
    require(tuple(state["planned_v13_fixtures"]) == tuple(row[0] for row in PLAN), "state_plan")
    require(state["requirements_sha256"] == NEW_REQUIREMENTS_SHA256, "state_requirements")
    require(state["result"] == "pass", "state_result")
    return stage, STAGE_COUNTS[stage]


def fixture_entry(identifier: str, requirements: tuple[str, ...]) -> dict[str, Any]:
    root = f"fixtures/v13/scenarios/epoch_semantics/{identifier}"
    metadata_path = root + ".fixture.json"
    metadata = load(metadata_path)
    require(tuple(metadata) == ("expected", "fixture_id", "inputs", "profile", "provenance", "requirements", "schema"), "metadata_shape:" + identifier)
    require(metadata["fixture_id"] == identifier and metadata["requirements"] == list(requirements), "metadata_identity:" + identifier)
    require(metadata["profile"] == "resource", "metadata_profile:" + identifier)
    inputs = metadata["inputs"]
    expected = metadata["expected"]
    require(type(inputs) is list and len(inputs) == 1 and type(inputs[0]) is dict, "metadata_input:" + identifier)
    require(type(expected) is dict, "metadata_expected:" + identifier)
    input_path = f"fixtures/v13/scenarios/epoch_semantics/{inputs[0].get('path')}"
    expected_path = f"fixtures/v13/scenarios/epoch_semantics/{expected.get('report_path')}"
    require(digest(input_path) == inputs[0].get("sha256"), "input_hash:" + identifier)
    require(digest(expected_path) == expected.get("sha256"), "expected_hash:" + identifier)
    return {"expected_path": expected_path, "fixture_id": identifier, "input_paths": [input_path], "metadata_path": metadata_path, "profile": "resource", "requirements": list(requirements)}


def planned_companion_paths(count: int) -> tuple[str, ...]:
    paths: list[str] = []
    for identifier, _ in PLAN[:count]:
        root = f"fixtures/v13/scenarios/epoch_semantics/{identifier}"
        paths.extend((root + ".expected.json", root + ".fixture.json", root + ".input.json"))
    return tuple(sorted(paths, key=str.encode))


def validate_companion_inventory(count: int, actual: tuple[str, ...] | None = None) -> None:
    expected = planned_companion_paths(count)
    if actual is None:
        actual = tuple(sorted((path.relative_to(ROOT).as_posix() for path in COMPANION_ROOT.glob("*.json")), key=str.encode)) if COMPANION_ROOT.exists() else ()
    require(actual == expected, "companion_inventory")
    for path in actual:
        candidate = pathlib.PurePosixPath(path)
        require(not candidate.is_absolute() and ".." not in candidate.parts, "companion_traversal")


def expected_manifest(state: dict[str, Any]) -> dict[str, Any]:
    stage, count = validate_state(state)
    base, _ = historical_base()
    fixtures = list(base["fixtures"])
    profiles = {key: list(value) for key, value in base["profiles"].items()}
    files = [dict(row) for row in base["files"]]
    original_files_projection = ordered_projection(files)
    requirement_rows = [row for row in files if row["path"] == "spec/requirements.json"]
    require(requirement_rows == [{"path": "spec/requirements.json", "sha256": OLD_REQUIREMENTS_SHA256}], "requirements_base_row")
    requirement_rows[0]["sha256"] = NEW_REQUIREMENTS_SHA256
    require(digest("spec/requirements.json") == NEW_REQUIREMENTS_SHA256, "requirements_current")
    validate_companion_inventory(count)
    for identifier, requirements in PLAN[:count]:
        entry = fixture_entry(identifier, requirements)
        fixtures.append(entry)
        profiles["resource"].append(identifier)
        for path in (*entry["input_paths"], entry["expected_path"], entry["metadata_path"]):
            files.append({"path": path, "sha256": digest(path)})
    fixtures.sort(key=lambda row: row["fixture_id"].encode())
    for values in profiles.values():
        values.sort(key=str.encode)
    files.sort(key=lambda row: row["path"].encode())
    identifiers = [row[0] for row in PLAN]
    return {
        "appended_v13_fixtures": identifiers[:count],
        "authorized_v12_source_rebindings": [{"path": "spec/requirements.json", "v12_sha256": OLD_REQUIREMENTS_SHA256, "v13_sha256": NEW_REQUIREMENTS_SHA256}],
        "base_manifest_sha256": BASE_SHA256,
        "complete": stage == "distribution_complete",
        "distribution_id": "draft_2026_08_signed_neutral_13",
        "distribution_schema": "nostr_automerge.fixture_distribution.v13",
        "files": files,
        "fixture_count": 198 + count,
        "fixtures": fixtures,
        "missing_v13_fixtures": identifiers[count:],
        "planned_v13_fixtures": identifiers,
        "preserved_v12_file_count": 641,
        "preserved_v12_files_sha256": original_files_projection,
        "preserved_v12_fixture_count": 198,
        "profiles": profiles,
        "protocol_revision": "draft_2026_08",
        "requirements_sha256": NEW_REQUIREMENTS_SHA256,
        "status": "canonical_signed_neutral_corpus" if stage == "distribution_complete" else "locked_transition",
        "supersedes": BASE_PATH,
        "target_fixture_count": 204,
        "transition_stage": stage,
    }


def canonical_bytes(state: dict[str, Any]) -> bytes:
    return canonical_json(expected_manifest(state))


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    state = load(STATE_PATH)
    expected = canonical_bytes(state)
    output = ROOT / OUTPUT_PATH
    if args.write:
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_bytes(expected)
    else:
        require(output.is_file() and output.read_bytes() == expected, "manifest_bytes")
    _, count = validate_state(state)
    print("PASS: generated distribution-v13 transition")
    print(f"- stage={state['current_stage']} fixtures={198 + count}/204")
    print(f"- manifest_sha256={hashlib.sha256(expected).hexdigest()}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
