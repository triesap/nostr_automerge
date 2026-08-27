#!/usr/bin/env python3
"""Generate or check the append-only signed distribution-v12 transition."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
STATE_PATH = "spec/distribution_v12_transition.json"
BASE_PATH = "fixtures/distribution/manifest_v11.json"
OUTPUT_PATH = "fixtures/distribution/manifest_v12.json"
SCHEMA_PATH = "tools/validation/distribution_v12.schema.json"
COMPANION_ROOT = ROOT / "fixtures/v12/scenarios/resource_followup"
COMPATIBILITY_REBINDING = "canonical_derivation_exact_budget"
COMPATIBILITY_SOURCE_ROOT = "fixtures/v1_draft/scenarios/resource"
COMPATIBILITY_TARGET_ROOT = "fixtures/v12/scenarios/resource_followup"
COMPATIBILITY_MAX_ITEMS = 371
BASE_CANDIDATE = "6f561e7ff4b12734e908dff6c98bc8139473052c"
BASE_SHA256 = "db247fa3e6891e850f32ed9b00fb08cfd78d30c9eb88ea36a00bd22dabb63f5a"
PLAN = (
    ("deep_delta_root_lookup_exact_budget", ("NCRDT-RESOURCE-015",), "resource"),
    ("deep_delta_absent_lookup_exact_budget", ("NCRDT-RESOURCE-015",), "resource"),
    ("deep_delta_extend_exact_budget", ("NCRDT-RESOURCE-015",), "resource"),
    (
        "post_branch_stop_has_no_target_work",
        ("NCRDT-COMPLETION-001", "NCRDT-RESOURCE-016"),
        "resource",
    ),
    ("unsupported_change_event_has_no_semantic_hash", ("NCRDT-VERSION-003",), "malformed"),
)
STAGE_COUNTS = {
    "authority_defined": 0,
    "inventory_installed": 0,
    "lookup_fixtures_added": 3,
    "stop_fixtures_added": 4,
    "distribution_complete": 5,
}
REBINDINGS = (
    (
        "spec/NIP_DRAFT.md",
        "0dfa683aa0f4a1c7d3df010ec95901bf4ba4094ed3adaacc26e85d95aaa4ded1",
        "8262bf32cb70b7c0e46210441120652e52504fb73839641ac19dddfed840acf8",
    ),
    (
        "spec/requirements.json",
        "f6e6070de7a5fc707f8488ced3a031f7dfc36d11c7477d800c3d3c33d532e6ba",
        "840822a1acf171c887b9a9aba79ddf159ffcd9c5d7a74bd74d7e0bac5c6161f4",
    ),
)
ENTRY_KEYS = (
    "expected_path",
    "fixture_id",
    "input_paths",
    "metadata_path",
    "profile",
    "requirements",
)


class DistributionError(ValueError):
    """The distribution-v12 transition contract was violated."""


def require(condition: bool, diagnostic: str) -> None:
    if not condition:
        raise DistributionError(diagnostic)


def digest(relative: str) -> str:
    return hashlib.sha256((ROOT / relative).read_bytes()).hexdigest()


def load(relative: str) -> dict[str, Any]:
    try:
        value = json.loads((ROOT / relative).read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise DistributionError(f"json:{relative}") from error
    require(type(value) is dict, f"object:{relative}")
    return value


def historical_bytes(relative: str) -> bytes:
    completed = subprocess.run(
        ("git", "show", f"{BASE_CANDIDATE}:{relative}"),
        cwd=ROOT,
        check=False,
        capture_output=True,
    )
    require(completed.returncode == 0 and completed.stderr == b"", f"historical:{relative}")
    return completed.stdout


def base_manifest() -> dict[str, Any]:
    historical = historical_bytes(BASE_PATH)
    require(hashlib.sha256(historical).hexdigest() == BASE_SHA256, "base_candidate_hash")
    require((ROOT / BASE_PATH).read_bytes() == historical, "base_manifest_drift")
    value = json.loads(historical)
    require(type(value) is dict, "base_manifest_object")
    require(value.get("fixture_count") == 193 and value.get("complete") is True, "base_identity")
    return value


def ordered_projection(rows: list[dict[str, str]]) -> str:
    state = hashlib.sha256()
    for row in rows:
        path = row["path"].encode()
        value = row["sha256"].encode()
        state.update(len(path).to_bytes(8, "big"))
        state.update(path)
        state.update(len(value).to_bytes(8, "big"))
        state.update(value)
    return state.hexdigest()


def validate_base_files(base: dict[str, Any]) -> list[dict[str, str]]:
    rows = base.get("files")
    require(type(rows) is list and len(rows) == 622, "base_files")
    expected_rebindings = {path: (old, new) for path, old, new in REBINDINGS}
    paths: list[str] = []
    result: list[dict[str, str]] = []
    for row in rows:
        require(type(row) is dict and tuple(row) == ("path", "sha256"), "base_file_shape")
        path = row["path"]
        old = row["sha256"]
        require(type(path) is str and type(old) is str, "base_file_type")
        paths.append(path)
        if path in expected_rebindings:
            expected_old, expected_new = expected_rebindings[path]
            require(old == expected_old and digest(path) == expected_new, f"source_rebinding:{path}")
            result.append({"path": path, "sha256": expected_new})
        else:
            require(digest(path) == old, f"preserved_v11:{path}")
            result.append({"path": path, "sha256": old})
    require(paths == sorted(set(paths), key=str.encode), "base_file_order")
    require(set(expected_rebindings).issubset(paths), "source_rebinding_inventory")
    return result


def fixture_entry(
    identifier: str,
    requirements: tuple[str, ...],
    profile: str,
    root: str | None = None,
) -> dict[str, Any]:
    root = root or f"fixtures/v12/scenarios/resource_followup/{identifier}"
    metadata_path = f"{root}.fixture.json"
    metadata = load(metadata_path)
    require(metadata.get("fixture_id") == identifier, f"metadata_id:{identifier}")
    require(metadata.get("requirements") == list(requirements), f"metadata_requirements:{identifier}")
    inputs = metadata.get("inputs")
    expected = metadata.get("expected")
    require(type(inputs) is list and len(inputs) == 1 and type(inputs[0]) is dict, f"metadata_input:{identifier}")
    require(type(expected) is dict, f"metadata_expected:{identifier}")
    directory = root.rsplit("/", 1)[0]
    input_path = f"{directory}/{inputs[0].get('path')}"
    expected_path = f"{directory}/{expected.get('report_path')}"
    require(digest(input_path) == inputs[0].get("sha256"), f"input_hash:{identifier}")
    require(digest(expected_path) == expected.get("sha256"), f"expected_hash:{identifier}")
    return {
        "expected_path": expected_path,
        "fixture_id": identifier,
        "input_paths": [input_path],
        "metadata_path": metadata_path,
        "profile": profile,
        "requirements": list(requirements),
    }


def planned_companion_paths(count: int) -> tuple[str, ...]:
    compatibility_root = f"{COMPATIBILITY_TARGET_ROOT}/{COMPATIBILITY_REBINDING}"
    paths = [
        f"{compatibility_root}.expected.json",
        f"{compatibility_root}.fixture.json",
        f"{compatibility_root}.input.json",
    ]
    for identifier, _, _ in PLAN[:count]:
        root = f"fixtures/v12/scenarios/resource_followup/{identifier}"
        paths.extend((f"{root}.expected.json", f"{root}.fixture.json", f"{root}.input.json"))
    return tuple(sorted(paths, key=str.encode))


def canonical_json(value: Any) -> bytes:
    return (
        json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
        + "\n"
    ).encode()


def compatibility_companions() -> dict[str, bytes]:
    identifier = COMPATIBILITY_REBINDING
    source_root = f"{COMPATIBILITY_SOURCE_ROOT}/{identifier}"
    target_root = f"{COMPATIBILITY_TARGET_ROOT}/{identifier}"
    expected = historical_bytes(f"{source_root}.expected.json")
    require(
        (ROOT / f"{source_root}.expected.json").read_bytes() == expected,
        "compatibility_expected_drift",
    )
    scenario = load(f"{source_root}.input.json")
    require(
        scenario.get("budget") == {"max_bytes": 1_000_000, "max_items": 335},
        "compatibility_source_budget",
    )
    scenario["budget"] = {
        "max_bytes": 1_000_000,
        "max_items": COMPATIBILITY_MAX_ITEMS,
    }
    input_bytes = canonical_json(scenario)
    metadata = load(f"{source_root}.fixture.json")
    require(metadata.get("fixture_id") == identifier, "compatibility_metadata")
    metadata["inputs"][0]["sha256"] = hashlib.sha256(input_bytes).hexdigest()
    metadata["expected"]["sha256"] = hashlib.sha256(expected).hexdigest()
    metadata["provenance"] = {
        "created_at": "2026-08-10",
        "generator": "nostr_automerge_conformance distribution_v12 compatibility rebinding",
        "generator_revision": "signed_scenario_v2",
        "source_versions": {"nostr_automerge": "0.1.0-alpha.0"},
    }
    return {
        f"{target_root}.expected.json": expected,
        f"{target_root}.fixture.json": canonical_json(metadata),
        f"{target_root}.input.json": input_bytes,
    }


def validate_compatibility_companions(write: bool) -> None:
    for relative, expected in compatibility_companions().items():
        path = ROOT / relative
        if write:
            path.write_bytes(expected)
        else:
            require(path.is_file() and path.read_bytes() == expected, f"compatibility:{relative}")


def validate_companion_inventory(count: int, actual: tuple[str, ...] | None = None) -> None:
    expected = planned_companion_paths(count)
    if actual is None:
        actual = tuple(
            sorted(
                (path.relative_to(ROOT).as_posix() for path in COMPANION_ROOT.glob("*.json")),
                key=str.encode,
            )
        )
    require(actual == expected, "companion_inventory")


def validate_state(state: dict[str, Any]) -> tuple[str, int]:
    keys = (
        "schema",
        "current_stage",
        "stage_order",
        "base_manifest",
        "base_manifest_sha256",
        "preserved_fixture_count",
        "target_fixture_count",
        "planned_v12_fixtures",
        "requirements_sha256",
        "result",
    )
    require(tuple(state) == keys, "state_keys")
    require(state["schema"] == "nostr_automerge.distribution_v12_transition.v1", "state_schema")
    require(tuple(state["stage_order"]) == tuple(STAGE_COUNTS), "state_stages")
    stage = state["current_stage"]
    require(type(stage) is str and stage in STAGE_COUNTS, "state_stage")
    require(state["base_manifest"] == BASE_PATH and state["base_manifest_sha256"] == BASE_SHA256, "state_base")
    require(state["preserved_fixture_count"] == 193 and state["target_fixture_count"] == 198, "state_counts")
    require(tuple(state["planned_v12_fixtures"]) == tuple(row[0] for row in PLAN), "state_plan")
    require(state["requirements_sha256"] == digest("spec/requirements.json"), "state_requirements")
    require(state["result"] == "pass", "state_result")
    return stage, STAGE_COUNTS[stage]


def expected_manifest(state: dict[str, Any] | None = None) -> dict[str, Any]:
    state = load(STATE_PATH) if state is None else state
    stage, count = validate_state(state)
    base = base_manifest()
    rebound_files = validate_base_files(base)
    validate_companion_inventory(count)
    target_identifiers = sorted(
        [*(row["fixture_id"] for row in base["fixtures"]), *(row[0] for row in PLAN)],
        key=str.encode,
    )
    require(
        len(target_identifiers) == 198 and len(set(target_identifiers)) == 198,
        "target_inventory",
    )
    appended = [fixture_entry(*row) for row in PLAN[:count]]
    replacement_root = f"{COMPATIBILITY_TARGET_ROOT}/{COMPATIBILITY_REBINDING}"
    replacement = fixture_entry(
        COMPATIBILITY_REBINDING,
        ("NCRDT-CONF-010", "NCRDT-RESOURCE-014"),
        "resource",
        replacement_root,
    )
    fixtures = [
        *(row for row in base["fixtures"] if row["fixture_id"] != COMPATIBILITY_REBINDING),
        replacement,
        *appended,
    ]
    fixtures.sort(key=lambda row: row["fixture_id"].encode())
    identifiers = [row["fixture_id"] for row in fixtures]
    require(identifiers == sorted(set(identifiers), key=str.encode), "fixture_order")
    profiles = {name: list(base["profiles"][name]) for name in ("checkpoint", "core", "malformed", "property", "resource")}
    for entry in appended:
        profiles[entry["profile"]].append(entry["fixture_id"])
    for values in profiles.values():
        values.sort(key=str.encode)
    file_paths = {row["path"]: row["sha256"] for row in rebound_files}
    file_paths[SCHEMA_PATH] = digest(SCHEMA_PATH)
    for entry in [replacement, *appended]:
        for path in (entry["metadata_path"], entry["expected_path"], *entry["input_paths"]):
            file_paths[path] = digest(path)
    files = [{"path": path, "sha256": file_paths[path]} for path in sorted(file_paths, key=str.encode)]
    complete = stage == "distribution_complete"
    planned = [row[0] for row in PLAN]
    return {
        "distribution_schema": "nostr_automerge.fixture_distribution.v12",
        "distribution_id": "draft_2026_08_signed_neutral_12",
        "protocol_revision": "draft_2026_08",
        "transition_stage": stage,
        "status": "canonical_signed_neutral_corpus" if complete else "locked_transition",
        "target_fixture_count": 198,
        "fixture_count": 193 + count,
        "complete": complete,
        "base_manifest_sha256": BASE_SHA256,
        "preserved_v11_fixture_count": 193,
        "preserved_v11_file_count": 622,
        "preserved_v11_files_sha256": ordered_projection(base["files"]),
        "authorized_v11_source_rebindings": [
            {"path": path, "v11_sha256": old, "v12_sha256": new}
            for path, old, new in REBINDINGS
        ],
        "planned_v12_fixtures": planned,
        "appended_v12_fixtures": planned[:count],
        "missing_v12_fixtures": planned[count:],
        "requirements_sha256": digest("spec/requirements.json"),
        "supersedes": BASE_PATH,
        "profiles": profiles,
        "fixtures": fixtures,
        "files": files,
    }


def canonical_bytes(state: dict[str, Any] | None = None) -> bytes:
    return (
        json.dumps(expected_manifest(state), ensure_ascii=False, sort_keys=True, separators=(",", ":"))
        + "\n"
    ).encode()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    validate_compatibility_companions(args.write)
    expected = canonical_bytes()
    output = ROOT / OUTPUT_PATH
    if args.write:
        output.write_bytes(expected)
        print(f"WROTE: {OUTPUT_PATH}")
    elif not output.is_file() or output.read_bytes() != expected:
        raise DistributionError("stale_manifest")
    else:
        print("PASS: signed distribution-v12 transition is deterministic")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
