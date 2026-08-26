#!/usr/bin/env python3
"""Generate or check the append-only resource follow-up distribution v11."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
BASE_PATH = "fixtures/distribution/manifest_v10.json"
OUTPUT_PATH = "fixtures/distribution/manifest_v11.json"
SCHEMA_PATH = "tools/validation/distribution_v11.schema.json"
OVERRIDE_IDS = (
    "foreign_claim_flood_exact_budget",
    "interrupted_after_checkpoint_resolution_returns_no_progress",
    "parent_propagation_exact_budget",
    "target_preparation_exact_budget",
    "target_raw_memo_exact_budget",
    "unrelated_control_flood_exact_budget",
    "unrelated_valid_checkpoints_exact_budget",
)
OVERRIDE_PROFILES = {
    "foreign_claim_flood_exact_budget": "core",
    "interrupted_after_checkpoint_resolution_returns_no_progress": "core",
    "parent_propagation_exact_budget": "core",
    "target_preparation_exact_budget": "resource",
    "target_raw_memo_exact_budget": "resource",
    "unrelated_control_flood_exact_budget": "resource",
    "unrelated_valid_checkpoints_exact_budget": "core",
}
APPENDED_ID = "checkpoint_lower_sequence_sibling_not_historical"
FOLLOWUP_ROOT = "fixtures/v11/scenarios/resource_followup"
ENTRY_KEYS = (
    "expected_path",
    "fixture_id",
    "input_paths",
    "metadata_path",
    "profile",
    "requirements",
)


class DistributionError(ValueError):
    """The append-only distribution contract was violated."""


def require(condition: bool, diagnostic: str) -> None:
    if not condition:
        raise DistributionError(diagnostic)


def digest(relative: str) -> str:
    return hashlib.sha256((ROOT / relative).read_bytes()).hexdigest()


def load(relative: str) -> dict[str, Any]:
    value = json.loads((ROOT / relative).read_text(encoding="utf-8"))
    require(isinstance(value, dict), f"object:{relative}")
    return value


def fixture_entry(fixture_id: str, profile: str) -> dict[str, Any]:
    prefix = f"{FOLLOWUP_ROOT}/{fixture_id}"
    metadata = load(f"{prefix}.fixture.json")
    require(metadata.get("fixture_id") == fixture_id, f"metadata:{fixture_id}")
    inputs = metadata.get("inputs")
    expected = metadata.get("expected")
    requirements = metadata.get("requirements")
    require(isinstance(inputs, list) and len(inputs) == 1, f"inputs:{fixture_id}")
    require(isinstance(expected, dict), f"expected:{fixture_id}")
    require(
        isinstance(requirements, list)
        and requirements
        and all(type(item) is str for item in requirements),
        f"requirements:{fixture_id}",
    )
    return {
        "expected_path": f"{FOLLOWUP_ROOT}/{expected['report_path']}",
        "fixture_id": fixture_id,
        "input_paths": [f"{FOLLOWUP_ROOT}/{inputs[0]['path']}"],
        "metadata_path": f"{prefix}.fixture.json",
        "profile": profile,
        "requirements": requirements,
    }


def preserved_projection(base: dict[str, Any]) -> str:
    projection = hashlib.sha256()
    files = base.get("files")
    require(isinstance(files, list), "base_files")
    for entry in files:
        require(
            isinstance(entry, dict)
            and tuple(entry) == ("path", "sha256")
            and type(entry.get("path")) is str
            and type(entry.get("sha256")) is str,
            "base_file",
        )
        path = entry["path"]
        expected = entry["sha256"]
        require(digest(path) == expected, f"preserved:{path}")
        path_bytes = path.encode()
        hash_bytes = expected.encode()
        projection.update(len(path_bytes).to_bytes(8, "big"))
        projection.update(path_bytes)
        projection.update(len(hash_bytes).to_bytes(8, "big"))
        projection.update(hash_bytes)
    return projection.hexdigest()


def expected_manifest() -> dict[str, Any]:
    base = load(BASE_PATH)
    require(base.get("fixture_count") == 192, "base_count")
    require(base.get("complete") is True, "base_complete")
    require(
        base.get("distribution_schema") == "nostr_automerge.fixture_distribution.v10",
        "base_schema",
    )
    replacements = {
        fixture_id: fixture_entry(fixture_id, OVERRIDE_PROFILES[fixture_id])
        for fixture_id in OVERRIDE_IDS
    }
    appended = fixture_entry(APPENDED_ID, "checkpoint")
    fixtures = []
    replaced = 0
    for entry in base["fixtures"]:
        require(isinstance(entry, dict) and tuple(entry) == ENTRY_KEYS, "base_entry")
        if entry["fixture_id"] in replacements:
            fixtures.append(replacements[entry["fixture_id"]])
            replaced += 1
        else:
            fixtures.append(entry)
    require(replaced == len(OVERRIDE_IDS), "override_cardinality")
    fixtures.append(appended)
    fixtures.sort(key=lambda entry: entry["fixture_id"].encode())
    require(len(fixtures) == 193, "fixture_count")
    require(
        all(
            fixtures[index - 1]["fixture_id"] < fixtures[index]["fixture_id"]
            for index in range(1, len(fixtures))
        ),
        "fixture_order",
    )
    profiles = {name: [] for name in ("checkpoint", "core", "malformed", "property", "resource")}
    for entry in fixtures:
        profiles[entry["profile"]].append(entry["fixture_id"])
    paths = {entry["path"] for entry in base["files"]}
    paths.add(SCHEMA_PATH)
    for entry in (*replacements.values(), appended):
        paths.add(entry["metadata_path"])
        paths.add(entry["expected_path"])
        paths.update(entry["input_paths"])
    files = [{"path": path, "sha256": digest(path)} for path in sorted(paths, key=str.encode)]
    return {
        "distribution_schema": "nostr_automerge.fixture_distribution.v11",
        "distribution_id": "draft_2026_08_signed_neutral_11",
        "protocol_revision": "draft_2026_08",
        "status": "canonical_signed_neutral_corpus",
        "target_fixture_count": 193,
        "fixture_count": 193,
        "complete": True,
        "base_manifest_sha256": digest(BASE_PATH),
        "preserved_v10_fixture_count": 192,
        "preserved_v10_file_count": len(base["files"]),
        "preserved_v10_files_sha256": preserved_projection(base),
        "intentional_v10_fixture_replacements": list(OVERRIDE_IDS),
        "appended_v11_fixtures": [APPENDED_ID],
        "requirements_sha256": digest("spec/requirements.json"),
        "opaque_private_assurance_identity": "d40e2f7424b04716f5da798da093907234492c43fa629cdca95c5434cb70a9c2",
        "supersedes": BASE_PATH,
        "profiles": profiles,
        "fixtures": fixtures,
        "files": files,
    }


def canonical_bytes() -> bytes:
    return (
        json.dumps(expected_manifest(), ensure_ascii=False, sort_keys=True, separators=(",", ":"))
        + "\n"
    ).encode()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    expected = canonical_bytes()
    output = ROOT / OUTPUT_PATH
    if args.write:
        output.write_bytes(expected)
        print(f"WROTE: {OUTPUT_PATH}")
    elif not output.is_file() or output.read_bytes() != expected:
        raise DistributionError("stale_manifest")
    else:
        print("PASS: signed distribution-v11 manifest is deterministic")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
